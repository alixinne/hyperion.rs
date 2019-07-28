//! Definition of the Method type

use std::time::Instant;

use crate::config::ColorFormat;
use crate::filters::ColorFilter;
use crate::runtime::{IdleTracker, LedInstance};

/// A method for communicating with a device
pub trait Method {
    /// Write the current LED status to the target device
    ///
    /// # Parameters
    ///
    /// * `time`: instant at which the filtered LED values should be evaluated
    /// * `filter`: filter to interpolate LED values
    /// * `leds`: reference to the LED state
    /// * `idle_tracker`: idle state tracker
    /// * `format`: device color format
    fn write(
        &mut self,
        time: Instant,
        filter: &ColorFilter,
        leds: &mut [LedInstance],
        idle_tracker: &mut IdleTracker,
        format: &ColorFormat,
    );
}
