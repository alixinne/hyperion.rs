//! Definition of the ValueStore type

use super::Sample;

use circular_queue::CircularQueue;

/// A struct for storing timestamped values for filtering
#[derive(Debug)]
pub struct ValueStore<T> {
    /// Circular buffer of samples
    samples: CircularQueue<Sample<T>>,
}

impl<T: std::fmt::Debug + PartialEq> ValueStore<T> {
    /// Create a new value store for `capacity` samples
    ///
    /// # Parameters
    ///
    /// * `capacity`: number of samples the store should hold
    pub fn with_capacity(capacity: usize) -> Self {
        // capacity = 0 makes no sense
        assert!(capacity > 0);

        Self {
            samples: CircularQueue::with_capacity(capacity),
        }
    }

    /// Sample a new value
    ///
    /// # Parameters
    ///
    /// * `sample`: sample
    pub fn push_sample(&mut self, sample: Sample<T>) {
        if sample.filtered {
            // Filtered value, replace last filtered value if it was
            // also filtered
            if let Some(last_sample) = self.samples.iter_mut().next() {
                if last_sample.filtered {
                    *last_sample = sample;
                    return;
                }
            }
        }

        // Target value sample, push it
        self.samples.push(sample);
    }

    /// Iterate values
    pub fn iter(&self) -> circular_queue::Iter<Sample<T>> {
        self.samples.iter()
    }

    /// Reset the stored values
    pub fn clear(&mut self) {
        self.samples.clear();
    }
}
