//! Definition of the Sample type

use std::time::Instant;

/// A sample value associated with its time
#[derive(Copy, Clone, Debug)]
pub struct Sample<T: Copy> {
    /// Time of the sample
    pub instant: Instant,
    /// Value of the sample
    pub value: T,
}

impl<T: Copy> Sample<T> {
    /// Create a new sample
    ///
    /// # Parameters
    ///
    /// `instant`: time of the sample
    /// `value`: value of the sample
    pub fn new(instant: Instant, value: T) -> Self {
        Self { instant, value }
    }
}
