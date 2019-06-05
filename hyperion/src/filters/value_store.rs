//! Definition of the ValueStore type

use super::Sample;

use circular_queue::CircularQueue;

/// A struct for storing timestamped values for filtering
#[derive(Debug)]
pub struct ValueStore<T> {
    /// Circular buffer of samples
    samples: CircularQueue<Sample<T>>,
    /// Circular buffer of filtered samples
    filtered_samples: CircularQueue<Sample<T>>,
}

impl<T: std::fmt::Debug> ValueStore<T> {
    /// Create a new value store for `capacity` samples
    ///
    /// # Parameters
    ///
    /// * `capacity`: number of samples the store should hold
    /// * `filtered_capacity`: number of filtered samples the store should hold
    pub fn with_capacity((capacity, filtered_capacity): (usize, usize)) -> Self {
        // capacity = 0 makes no sense
        assert!(capacity > 0);
        assert!(filtered_capacity > 0);

        Self {
            samples: CircularQueue::with_capacity(capacity),
            filtered_samples: CircularQueue::with_capacity(filtered_capacity),
        }
    }

    /// Sample a new value
    ///
    /// # Parameters
    ///
    /// * `sample`: sample
    /// * `filtered`: true if this is a filtered sample
    pub fn push_sample(&mut self, sample: Sample<T>, filtered: bool) {
        if filtered {
            self.filtered_samples.push(sample);
        } else {
            self.samples.push(sample);
        }
    }

    /// Iterate samples
    pub fn iter(&self) -> circular_queue::Iter<Sample<T>> {
        self.samples.iter()
    }

    /// Iterate filtered samples
    pub fn iter_filtered(&self) -> circular_queue::Iter<Sample<T>> {
        self.filtered_samples.iter()
    }

    /// Reset the stored values
    pub fn clear(&mut self) {
        self.samples.clear();
        self.filtered_samples.clear();
    }
}
