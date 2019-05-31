//! Definition of the Nearest type

use std::time::Instant;

use super::{Filter, ValueStore};

#[derive(Default)]
pub struct Nearest;

impl<T: std::fmt::Debug + Default + Clone + PartialEq> Filter<T> for Nearest {
    fn current_value(&self, _time: Instant, value_store: &ValueStore<T>) -> T {
        value_store
            .iter()
            .map(|sample| sample.value.clone())
            .next()
            .unwrap_or_else(Default::default)
    }

    fn capacity(&self, _frequency: f32) -> usize {
        1
    }
}
