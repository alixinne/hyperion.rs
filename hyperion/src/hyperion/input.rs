//! Input type definition

use std::convert::TryInto;
use std::time::Duration;

use super::StateUpdate;

/// Hyperion input information
#[derive(Debug, Clone)]
pub enum Input {
    /// One shot command, applied immediately once
    OneShot(StateUpdate),
    /// Priority-only command
    Priority {
        /// Command to execute
        update: StateUpdate,
        /// Priority of the input
        priority: i32,
    },
    /// Full command, applied for a duration if its priority is high enough
    Full {
        /// Command to execute
        update: StateUpdate,
        /// Priority of the input
        priority: i32,
        /// Duration to apply the input for
        duration: Duration,
    },
}

impl Input {
    /// Get the duration of an input
    ///
    /// A very large value is returned if the input has no duration.
    pub fn get_duration(&self) -> Duration {
        match self {
            Input::Full { duration, .. } => *duration,
            _ => Duration::from_millis(std::u32::MAX as u64),
        }
    }

    /// Get the priority of an input
    ///
    /// Items without priority will return the highest priority (apply instantly)
    pub fn get_priority(&self) -> i32 {
        match self {
            Input::Full { priority, .. } | Input::Priority { priority, .. } => *priority,
            _ => std::i32::MAX,
        }
    }

    /// Convert the input into its associated state update
    pub fn into_update(self) -> StateUpdate {
        match self {
            Input::OneShot(update) => update,
            Input::Priority { update, .. } => update,
            Input::Full { update, .. } => update,
        }
    }

    /// Create a new input
    ///
    /// # Parameters
    ///
    /// * `update`: update contents
    pub fn new(update: StateUpdate) -> Self {
        Input::OneShot(update)
    }

    /// Create an input from priority value
    ///
    /// # Parameters
    ///
    /// * `update`: update contents
    /// * `priority`: priority of the update
    pub fn from_priority(update: StateUpdate, priority: i32) -> Self {
        Input::Priority { update, priority }
    }

    /// Create an input from full input details
    ///
    /// # Parameters
    ///
    /// * `update`: update contents
    /// * `priority`: priority of the update
    /// * `duration`: duration of the update
    pub fn from_full(update: StateUpdate, priority: i32, duration: Option<i32>) -> Self {
        match duration {
            Some(duration) if duration > 0 => Input::Full { update, priority, duration: Duration::from_millis(duration.try_into().unwrap()) },
            _ => Input::Priority { update, priority }
        }
    }
}
