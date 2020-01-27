//! Definition of the IdleTracker type

use std::fmt;
use std::time::{Duration, Instant};

use crate::color;
use crate::config::IdleSettings;
use crate::methods::{WriteError, WriteResult};

/// Idle pass statistics
pub struct IdlePass<'t> {
    /// Referenced idle tracker
    tracker: &'t mut IdleTracker,
    /// Device name for logging
    device_name: &'t str,
    /// Total change in all color components in the current pass
    total_change: f64,
    /// Number of LEDs with non-zero color value
    nonzero_color_count: usize,
    /// Write completion state
    completion_state: Option<WriteResult>,
}

impl<'t> IdlePass<'t> {
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

    /// Adds the device write completion state to this pass
    ///
    /// This allows scheduling the next write time appropriately
    ///
    /// # Parameters
    ///
    /// * `completion_state`: result from the method write call
    pub fn complete(&mut self, completion_state: WriteResult) {
        self.completion_state = Some(completion_state);
    }
}

impl<'t> Drop for IdlePass<'t> {
    fn drop(&mut self) {
        self.tracker.end_pass(
            self.total_change,
            self.nonzero_color_count,
            self.device_name,
            self.completion_state.take(),
        );
    }
}

/// RGB LED idle tracker
pub struct IdleTracker {
    /// Duration after which the device is considered idle
    idle_settings: IdleSettings,
    /// Update period (derived from device frequency)
    update_period: Duration,
    /// Number of write passes since the last change
    passes_since_last_change: u32,
    /// Current state of the tracker
    current_state: IdleState,
    /// Next write
    next_write: Option<Instant>,
}

/// Current state of the tracked device
#[derive(Clone, PartialEq)]
enum IdleState {
    /// The device is actively being updated
    Active,
    /// The device is idle and turned off
    IdleBlack,
    /// The device is idle but with a solid color
    IdleColor,
    /// The device encountered an error and will be woken up later
    Errored { error: String },
}

impl IdleState {
    pub fn is_errored(&self) -> bool {
        match self {
            IdleState::Errored { .. } => true,
            _ => false,
        }
    }
}

impl fmt::Display for IdleState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IdleState::Active => write!(f, "active"),
            IdleState::IdleColor => write!(f, "idle (active)"),
            IdleState::IdleBlack => write!(f, "idle (inactive)"),
            IdleState::Errored { error } => write!(f, "errored ({})", error),
        }
    }
}

impl IdleTracker {
    /// Create a new idle tracker
    ///
    /// # Parameters
    ///
    /// * `idle_settings`: settings for device idle modes
    /// * `frequency`: device update frequency in Hz
    pub fn new(idle_settings: IdleSettings, frequency: f64) -> Self {
        Self {
            idle_settings,
            update_period: Duration::from_nanos((1_000_000_000f64 / frequency) as u64),
            passes_since_last_change: 0,
            current_state: IdleState::Active,
            next_write: Some(Instant::now()),
        }
    }

    /// Notify of a recent state change
    ///
    /// This should be call when new input data is available, thus a potentially
    /// idle device needs to be updated as soon as possible
    ///
    /// # Parameters
    ///
    /// * `time`: instant at which the updated occurred
    ///
    /// # Returns
    ///
    /// `true` if the device had to change state because of this notification.
    pub fn update_write(&mut self, time: Instant) -> bool {
        let update_instant = match self.current_state {
            IdleState::Active | IdleState::Errored { .. } => None,
            IdleState::IdleBlack | IdleState::IdleColor => Some(time),
        };

        // Switch to active state
        if !self.current_state.is_errored() {
            self.current_state = IdleState::Active;
        }

        // Update next write
        if update_instant.is_some() {
            self.next_write = update_instant;
        }

        update_instant.is_some()
    }

    /// Starts a new pass for the given device
    ///
    /// The given object should be used to track changes to LED values in an update pass.
    pub fn start_pass<'t>(&'t mut self, device_name: &'t str) -> IdlePass<'t> {
        IdlePass {
            tracker: self,
            device_name,
            total_change: 0.,
            nonzero_color_count: 0,
            completion_state: None,
        }
    }

    /// Get the time at which the scheduler should write to the associated device again
    pub fn next_write(&self) -> Option<Instant> {
        self.next_write
    }

    /// Completes the current pass
    fn end_pass(
        &mut self,
        total_change: f64,
        nonzero_color_count: usize,
        device_name: &str,
        completion_state: Option<WriteResult>,
    ) {
        let (new_state, update_delay) =
            if completion_state.as_ref().map(Result::is_ok).unwrap_or(true) {
                // Write to device disabled OR write to device was a success

                // Update change values
                if total_change > 2.0f64.powf(-f64::from(self.idle_settings.resolution)) {
                    self.passes_since_last_change = 1;
                } else if self.passes_since_last_change < self.idle_settings.retries {
                    self.passes_since_last_change += 1;
                }

                if !self.idle_settings.enabled
                    || self.passes_since_last_change >= self.idle_settings.retries
                {
                    // Nothing changed recently
                    if nonzero_color_count > 0 {
                        // When a color is displayed, we only require an update every delay
                        // if the device needs periodic updates to stay on.
                        (
                            IdleState::IdleColor,
                            if self.idle_settings.holds {
                                None
                            } else {
                                Some(self.idle_settings.delay)
                            },
                        )
                    } else {
                        (IdleState::IdleBlack, None)
                    }
                } else {
                    (IdleState::Active, Some(self.update_period))
                }
            } else {
                // completion_state is Some(Err()) from above if
                let completion_state = completion_state.unwrap().unwrap_err();

                match completion_state {
                    WriteError::NotReady => {
                        // Temporary error, we should try again soon

                        (
                            if !self.idle_settings.enabled
                                || self.passes_since_last_change >= self.idle_settings.retries
                            {
                                // Nothing changed recently
                                if nonzero_color_count > 0 {
                                    IdleState::IdleColor
                                } else {
                                    IdleState::IdleBlack
                                }
                            } else {
                                IdleState::Active
                            },
                            Some(self.update_period),
                        )
                    }
                    WriteError::Errored { error } => {
                        // Permanent error, try again after some timeout

                        (IdleState::Errored { error }, Some(Duration::from_secs(60)))
                    }
                }
            };

        trace!(
            "end pass: total_change: {}, passes_since_last_change: {}",
            total_change,
            self.passes_since_last_change
        );

        let changed = new_state != self.current_state;
        self.current_state = new_state;

        // Notify log of state changes
        if changed {
            log!(
                if self.current_state.is_errored() {
                    log::Level::Error
                } else {
                    log::Level::Debug
                },
                "device '{}' is now {}{}",
                device_name,
                self.current_state,
                if let Some(duration) = update_delay {
                    format!(", next update in {}", humantime::Duration::from(duration))
                } else {
                    "".to_owned()
                }
            );
        }

        // Update next write deadline
        self.next_write = update_delay.map(|d| Instant::now() + d);
    }
}
