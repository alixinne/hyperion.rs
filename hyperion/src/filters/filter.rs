//! Definition of the Filter type

use std::time::Instant;

use super::ValueStore;

use crate::color;

/// A generic time-domain filter over values
pub trait Filter<T> {
    /// Computes the current value using this filter and the stored values
    ///
    /// # Parameters
    ///
    /// * `time`: time at which the value should evaluated
    /// * `value_store`: stored values to use for the evaluation
    fn current_value(&self, time: Instant, value_store: &ValueStore<T>) -> T;

    /// Returns the recommended size for the value store for this temporal filter
    ///
    /// This is a tuple containing the raw sample size and the filtered sample size.
    ///
    /// # Parameters
    ///
    /// * `frequency`: update frequency of the device this filter is to be used for
    fn capacity(&self, frequency: f32) -> (usize, usize);
}

/// Linear RGB color filter implementation
pub type ColorFilter = Box<dyn Filter<color::ColorPoint> + Send>;

/// Filter configuration
type FilterConfig = crate::config::Filter;

impl From<FilterConfig> for ColorFilter {
    fn from(config: FilterConfig) -> Self {
        use super::*;
        trace!("creating filter for {:?}", config);

        match config {
            crate::config::Filter::Nearest => Box::new(Nearest::default()),
            crate::config::Filter::Linear { frequency } => Box::new(Linear::new(frequency)),
        }
    }
}
