//! Definition of the Linear type

use std::ops::{Add, Mul, Sub};
use std::time::{Duration, Instant};

use super::{Filter, Sample, ValueStore};

pub struct Linear {
    frequency: f32,
}

impl Linear {
    pub fn new(frequency: f32) -> Self {
        Self { frequency }
    }
}

fn t(d: Duration) -> f32 {
    (d.as_secs() as u64 * 1_000_000u64 + u64::from(d.subsec_micros())) as f32 / 1_000_000f32
}

use std::fmt::Debug;

impl<
        T: PartialEq
            + Debug
            + Default
            + Clone
            + Add<T, Output = T>
            + Sub<T, Output = T>
            + Mul<f32, Output = T>,
    > Filter<T> for Linear
{
    fn current_value(&self, time: Instant, value_store: &ValueStore<T>) -> T {
        let period = 1.0 / self.frequency;

        if let Some(last_target_sample) = value_store.iter().find(|sample| !sample.filtered) {
            let default_sample = Sample::new(
                time - Duration::from_millis((1000f32 * period) as u64),
                Default::default(),
                true,
            );

            // We should target last_target_sample, linearly from the current value
            let current_sample = if let Some(last_filtered_sample) =
                value_store.iter().find(|sample| sample.filtered)
            {
                last_filtered_sample
            } else {
                warn!("no filtered value found, consider increasing the value store capacity");

                // Alas, we have no clue on the current value of the target, assume 0
                &default_sample
            };

            // The difference we still have to cover
            let value_diff = last_target_sample.value.clone() - current_sample.value.clone();
            // The time difference between the current time and the target point
            let time_diff = t(time - last_target_sample.instant);

            if time_diff >= period {
                // Linear filtering period over
                last_target_sample.value.clone()
            } else {
                // Linear filtering in effect
                current_sample.value.clone() + value_diff * (time_diff / period)
            }
        } else {
            // No target sample found
            if let Some(any_sample) = value_store.iter().next() {
                any_sample.value.clone()
            } else {
                // Initial case, no value at all
                Default::default()
            }
        }
    }

    fn capacity(&self, frequency: f32) -> usize {
        std::cmp::max(1, (frequency / self.frequency).ceil() as usize)
    }
}
