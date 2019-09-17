//! Definition of the Linear type

use std::ops::{Add, Mul};
use std::time::{Duration, Instant};

use super::{Filter, Sample, ValueStore};

/// Linear filter
///
/// Linearly interpolates values from the unfiltered samples over time.
pub struct Linear {
    /// Filter window, as a frequency in Hz
    frequency: f32,
}

impl Linear {
    /// Create a new linear filter
    ///
    /// # Parameters
    ///
    /// * `frequency`: filtering window size, as a frequency in Hz
    pub fn new(frequency: f32) -> Self {
        Self { frequency }
    }
}

/// Convert a duration to floating-point seconds
///
/// # Parameters
///
/// * `d`: duration to convert to seconds
fn t(d: Duration) -> f32 {
    (d.as_secs() as u64 * 1_000_000u64 + u64::from(d.subsec_micros())) as f32 / 1_000_000f32
}

use std::fmt::Debug;

impl<T: Debug + Default + Copy + Add<T, Output = T> + Mul<f32, Output = T>> Filter<T> for Linear {
    fn current_value(&self, time: Instant, value_store: &ValueStore<T>) -> T {
        let period = 1.0 / self.frequency;

        if let Some(last_target_sample) = value_store.iter().next() {
            let default_sample = Sample::new(
                time - Duration::from_millis((1000f32 * period) as u64),
                Default::default(),
            );

            // We should target last_target_sample, linearly from the current value
            let current_sample =
                if let Some(last_filtered_sample) = value_store.iter_filtered().next() {
                    last_filtered_sample
                } else {
                    warn!("no filtered value found, consider increasing the value store capacity");

                    // Alas, we have no clue on the current value of the target, assume 0
                    &default_sample
                };

            // The time difference between the current time and the target point
            let time_diff = t(time - last_target_sample.instant);

            if time_diff >= period {
                // Linear filtering period over
                last_target_sample.value
            } else {
                let time_factor = time_diff / period;

                // Linear filtering in effect
                current_sample.value
                    + last_target_sample.value * time_factor
                    + current_sample.value * (-time_factor)
            }
        } else {
            // No target sample found
            // Initial case, no value at all
            Default::default()
        }
    }

    fn capacity(&self, _frequency: f32) -> (usize, usize) {
        (1, 1)
    }
}
