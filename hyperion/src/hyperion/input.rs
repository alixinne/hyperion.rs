//! Input type definition

use std::convert::TryInto;
use std::time::Duration;

use super::{ServiceCommand, StateUpdate};

use crate::servers::json::Effect;

/// Hyperion input information
#[derive(Debug, Clone)]
pub enum Input {
    /// Command coming from outside sources
    UserInput {
        /// Command to execute
        update: StateUpdate,
        /// Priority of the input
        priority: Option<i32>,
        /// Duration to apply the input for
        duration: Option<Duration>,
    },
    /// Effect command
    Effect {
        /// Effect to run
        effect: Effect,
        /// Priority of the input
        priority: Option<i32>,
        /// Duration to apply the effect for
        duration: Option<Duration>,
    },
    /// State change issued by an effect
    ///
    /// The priority is stored by the running effect engine,
    /// and compared to when cancelling effects.
    EffectInput {
        /// Command to execute
        update: StateUpdate,
    },
    /// Internal command, not a direct user input
    Internal(ServiceCommand),
}

impl Input {
    /// Get the duration of an input
    ///
    /// A very large value is returned if the input has no duration.
    pub fn get_duration(&self) -> Option<Duration> {
        match self {
            Input::UserInput { duration, .. } | Input::Effect { duration, .. } => duration.clone(),
            _ => None,
        }
    }

    /// Get the priority of an input
    ///
    /// Items without priority will return the highest priority (apply instantly)
    pub fn get_priority(&self) -> Option<i32> {
        match self {
            Input::UserInput { priority, .. } | Input::Effect { priority, .. } => priority.clone(),
            _ => None,
        }
    }

    /// Create a new user input
    ///
    /// # Parameters
    ///
    /// * `update`: update contents
    /// * `priority`: update priority
    /// * `duration`: update duration, in milliseconds
    pub fn user_input(update: StateUpdate, priority: i32, duration: Option<i32>) -> Self {
        Input::UserInput {
            update,
            priority: if priority >= 0 { Some(priority) } else { None },
            duration: duration.and_then(|d| d.try_into().ok().map(|d| Duration::from_millis(d))),
        }
    }

    /// Create a new effect input
    ///
    /// # Parameters
    ///
    /// * `update`: update contents
    pub fn effect_input(update: StateUpdate) -> Self {
        Input::EffectInput { update }
    }

    /// Create an input from effect input details
    ///
    /// # Parameters
    ///
    /// * `priority`: priority of the update
    /// * `duration`: duration of the update
    /// * `effect`: effect to run
    pub fn effect(priority: i32, duration: i32, effect: Effect) -> Self {
        Input::Effect {
            effect,
            priority: Some(priority),
            duration: if duration > 0 {
                Some(Duration::from_millis(duration.try_into().unwrap()))
            } else {
                None
            },
        }
    }
}

impl From<ServiceCommand> for Input {
    fn from(command: ServiceCommand) -> Self {
        Input::Internal(command)
    }
}
