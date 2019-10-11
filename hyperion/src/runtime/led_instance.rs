//! Definition of the LedInstance type

use std::time::Instant;

use crate::color;
use crate::config::Led;
use crate::filters::{ColorFilter, Sample, ValueStore};
use crate::runtime::IdleTracker;

/// Instance of a LED at runtime
///
/// Combines the specification of the LED coverage of the screen area plus
/// its current state.
#[derive(Debug)]
pub struct LedInstance {
    /// LED parameters for this instance
    pub spec: Led,
    /// History of values for this LED
    values: ValueStore<color::ColorPoint>,
    /// Current (i.e. written) value of the LED
    current_color: color::ColorPoint,
}

impl LedInstance {
    /// Create a new LedInstance from a LED specification
    ///
    /// # Parameters
    ///
    /// * `led`: LED specification
    /// * `capacity`: filtering value store capacity
    pub fn new(led: Led, capacity: (usize, usize)) -> Self {
        Self {
            spec: led,
            values: ValueStore::with_capacity(capacity),
            current_color: Default::default(),
        }
    }

    /// Get the current color of this LED
    pub fn current_color(&self) -> color::ColorPoint {
        self.current_color
    }

    /// Reload an LED's configuration from a new instance
    ///
    /// This will preserve the current value store, and update it to the
    /// required capacity.
    ///
    /// # Parameters
    ///
    /// * `new_led`: new LED instance to pull settings from
    pub fn reload(&mut self, led: Led, capacity: (usize, usize)) {
        // Update spec
        self.spec = led;

        // Update capacity
        self.values.set_capacity(capacity);
    }

    /// Updates this LED's color
    ///
    /// # Parameters
    ///
    /// `time`: time of the update sample
    /// `new_color`: new LED color
    /// `immediate`: force instant update (breaks filtering continuity)
    pub fn update_color(&mut self, time: Instant, new_color: color::ColorPoint, immediate: bool) {
        // Clear buffered values for the filter if in immediate mode
        if immediate {
            self.values.clear();
        }

        // Update value
        self.values.push_sample(Sample::new(time, new_color), false);
    }

    /// Update the current value of the LED using the given filter
    ///
    /// # Parameters
    ///
    /// * `time`: instant to evaluate the color at
    /// * `filter`: filter to use for computing the value
    /// * `idle_tracker`: idle state tracker
    pub fn next_value(
        &mut self,
        time: Instant,
        filter: &ColorFilter,
        idle_tracker: &mut IdleTracker,
    ) {
        // Compute new value
        let new_value = filter.current_value(time, &self.values);

        // Notify value change
        idle_tracker.update_color(&self.current_color, &new_value);

        // Add the value to the store
        self.values.push_sample(Sample::new(time, new_value), true);

        // Update stored color
        self.current_color = new_value;
    }
}
