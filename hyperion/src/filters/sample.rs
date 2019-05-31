//! Definition of the Sample type

use std::time::Instant;

/// A sample value associated with its time
#[derive(Debug)]
pub struct Sample<T> {
    /// Time of the sample
    pub instant: Instant,
    /// Value of the sample
    pub value: T,
    /// Is it a filtered value
    pub filtered: bool,
}

impl<T> Sample<T> {
    /// Create a new sample
    ///
    /// # Parameters
    ///
    /// `instant`: time of the sample
    /// `value`: value of the sample
    /// `filtered`: true if this sample has been filtered
    pub fn new(instant: Instant, value: T, filtered: bool) -> Self {
        Self {
            instant,
            value,
            filtered,
        }
    }
}
