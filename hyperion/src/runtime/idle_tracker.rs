//! Definition of the IdleTracker type

use std::fmt;
use std::time::Instant;

use crate::color;
use crate::config::IdleSettings;

/// RGB LED idle tracker
pub struct IdleTracker {
    /// Duration after which the device is considered idle
    idle_settings: IdleSettings,
    /// Total change in all color components in the current pass
    total_change: f64,
    /// Number of LEDs with non-zero color value
    nonzero_color_count: usize,
    /// Instant of the last change in any LED value
    last_change: Instant,
    /// Number of write passes since the last change
    passes_since_last_change: u32,
    /// true if an update pass is running
    pass_started: bool,
    /// Current state of the tracker
    current_state: IdleState,
    /// Is an update pending?
    update_pending: bool,
}

/// Current state of the tracked device
#[derive(Clone)]
pub enum IdleState {
    /// The device is actively being updated
    Active,
    /// The device is idle and turned off
    IdleBlack,
    /// The device is idle but with a solid color
    IdleColor {
        /// true if the device should be updated to prevent it
        /// from turning off after its inactivity timeout.
        update_required: bool,
    },
}

impl IdleState {
    /// Returns true if the state requires updating the target device
    pub fn should_write(&self) -> bool {
        match self {
            IdleState::Active
            | IdleState::IdleColor {
                update_required: true,
            } => true,
            _ => false,
        }
    }

    /// Returns true if the two states are different variants
    ///
    /// # Parameters
    ///
    /// * `other`: state to compare this state to
    pub fn has_changed(&self, other: &IdleState) -> bool {
        match self {
            IdleState::Active => match other {
                IdleState::Active => false,
                _ => true,
            },
            IdleState::IdleBlack => match other {
                IdleState::IdleBlack => false,
                _ => true,
            },
            IdleState::IdleColor { .. } => match other {
                IdleState::IdleColor { .. } => false,
                _ => true,
            },
        }
    }
}

impl fmt::Display for IdleState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IdleState::Active => write!(f, "active"),
            IdleState::IdleColor { .. } => write!(f, "idle (active)"),
            IdleState::IdleBlack => write!(f, "idle (inactive)"),
        }
    }
}

impl From<IdleSettings> for IdleTracker {
    /// Create a new idle tracker
    ///
    /// # Parameters
    ///
    /// * `idle_settings`: settings for device idle modes
    fn from(idle_settings: IdleSettings) -> Self {
        Self {
            idle_settings,
            total_change: 0.0,
            nonzero_color_count: 0,
            last_change: Instant::now(),
            passes_since_last_change: 0,
            pass_started: false,
            current_state: IdleState::Active,
            update_pending: false,
        }
    }
}

impl IdleTracker {
    /// Starts a new pass
    ///
    /// This function should be called before updating LEDs in the device.
    pub fn start_pass(&mut self) {
        assert!(!self.pass_started);

        self.total_change = 0.0;
        self.nonzero_color_count = 0;

        self.pass_started = true;
    }

    /// Completes the current pass
    ///
    /// This function should be called after the LEDs have been updated.
    pub fn end_pass(&mut self) {
        assert!(self.pass_started);

        self.last_change = Instant::now();

        // Update change values
        if self.total_change > 2.0f64.powf(-f64::from(self.idle_settings.resolution)) {
            self.passes_since_last_change = 1;
        } else if self.passes_since_last_change < self.idle_settings.retries {
            self.passes_since_last_change += 1;
        }

        trace!(
            "end pass: total_change: {}, last_change: {:?}, passes_since_last_change: {}",
            self.total_change,
            self.last_change,
            self.passes_since_last_change
        );

        self.pass_started = false;
    }

    /// Notifies the tracker that LED state has changed and should be checked again
    pub fn notify_changed(&mut self) {
        self.update_pending = true;
    }

    /// Notifies of an update on an LED color
    ///
    /// This function should be called for every LED color update. Note that this only tracks
    /// changes, but does not update the actual color.
    ///
    /// # Parameters
    ///
    /// * `current_color`: current color of the LED being updated
    /// * `new_color`: new color value for the LED
    pub fn update_color(
        &mut self,
        current_color: &color::ColorPoint,
        new_color: &color::ColorPoint,
    ) {
        let diff = current_color.diff(new_color);

        // Add up total color difference
        if diff > 0.0 {
            self.total_change += f64::from(diff);
        }

        // Check if everything is black
        if !new_color.is_black() {
            self.nonzero_color_count += 1;
        }
    }

    /// Update the current state of this tracker
    ///
    /// Note that if this method returns a state that expects the device to be written to,
    /// the internal change tracker state will be updated assuming the caller does actually
    /// write to the device.
    ///
    /// # Returns
    ///
    /// * `(changed, state)`: `changed` is true if the state changed to its current value `state`.
    /// The `changed` flag does not take into account the state of `update_required` on
    /// IdleColor.
    pub fn update_state(&mut self) -> (bool, IdleState) {
        let now = Instant::now();

        let new_state =
            // Only consider idle stats if idling is enabled, and we are not waiting on a oneshot
            // update
            if !self.update_pending && self.idle_settings.enabled && self.passes_since_last_change >= self.idle_settings.retries {
                if self.nonzero_color_count > 0 {
                    // When a color is displayed, we only require an update every delay
                    // if the device needs periodic updates to stay on.
                    IdleState::IdleColor {
                        update_required: !self.idle_settings.holds
                            && (now - self.last_change) > self.idle_settings.delay,
                    }
                } else {
                    IdleState::IdleBlack
                }
            } else {
                IdleState::Active
            };

        // Acknowledge update notifications
        self.update_pending = false;

        let changed = new_state.has_changed(&self.current_state);
        self.current_state = new_state;

        (changed, self.current_state.clone())
    }
}
