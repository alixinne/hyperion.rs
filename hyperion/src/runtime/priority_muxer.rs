//! Input priority muxer

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::pin::Pin;

use std::mem::replace;
use std::time::Instant;

use futures::prelude::*;
use futures::task::Poll;

use crate::hyperion::{Input, ServiceCommand, StateUpdate};
use crate::servers::json::Effect;

/// Boxed stream of service inputs
pub type ServiceInputReceiver = Pin<Box<dyn Stream<Item = Input>>>;

/// Priority muxer
///
/// Type responsible for determining which update applies depending on durations and priorities.
pub struct PriorityMuxer {
    /// Input command receiver
    receiver: ServiceInputReceiver,
    /// Priority queue of inputs
    inputs: BinaryHeap<MuxerEntry>,
}

/// Result of service inputs muxed by priority
#[derive(Debug, Clone)]
pub enum MuxedInput {
    /// Lighting system state update
    StateUpdate {
        /// LED state update details
        update: StateUpdate,
        /// true if currently running effects should be cleared
        clear_effects: bool,
    },
    /// Effect launch request
    LaunchEffect {
        /// Details of the effect being launched
        effect: Effect,
        /// End time of the effect
        deadline: Option<Instant>,
    },
    /// Internal service update
    Internal(ServiceCommand),
}

/// Entry in the muxer queue
struct MuxerEntry {
    /// Input data, None when it was sent as a StateUpdate
    input: Option<Input>,
    /// Expiration date of the entry
    deadline: Option<Instant>,
    /// Priority of the entry
    priority: i32,
}

impl From<Input> for MuxerEntry {
    fn from(input: Input) -> Self {
        // Use duration or 24h from now as a default timeout
        let deadline = input.get_duration().map(|d| Instant::now() + d);

        // Default priority
        let priority = input.get_priority().unwrap_or(1000);

        Self {
            input: Some(input),
            deadline,
            priority,
        }
    }
}

impl Eq for MuxerEntry {}

impl PartialEq for MuxerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline && self.priority == other.priority
    }
}

impl Ord for MuxerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.deadline.cmp(&other.deadline))
    }
}

impl PartialOrd for MuxerEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PriorityMuxer {
    /// Create a new PriorityMuxer
    ///
    /// # Parameters
    ///
    /// * `receiver`: channel receiver for input commands
    pub fn new(receiver: ServiceInputReceiver) -> Self {
        Self {
            receiver,
            inputs: BinaryHeap::new(),
        }
    }
}

impl Stream for PriorityMuxer {
    type Item = MuxedInput;

    fn poll_next(
        mut self: Pin<&mut Self>,
        ctx: &mut std::task::Context,
    ) -> Poll<Option<Self::Item>> {
        // Receive incoming inputs
        while let Poll::Ready(value) = Stream::poll_next(Pin::new(&mut self.receiver), ctx) {
            if let Some(input) = value {
                // Forward internal commands directly
                if let Input::Internal(service_command) = input {
                    return Poll::Ready(Some(MuxedInput::Internal(service_command)));
                }

                // Push inputs into queue
                self.inputs.push(input.into());
            } else {
                return Poll::Ready(None);
            }
        }

        let now = Instant::now();
        let mut expired_entries = false;

        // Remove expired inputs
        while let Some(entry) = self.inputs.peek() {
            if entry.deadline.map(|d| d < now).unwrap_or(false) {
                trace!("input {:?} has expired", entry.input);
                self.inputs.pop();
                expired_entries = true;
            } else {
                break;
            }
        }

        // Should we pop the top entry
        let mut pop_top_entry = false;
        let mut result = None;

        // Send non-forwarded top input if any
        if let Some(mut entry) = self.inputs.peek_mut() {
            // Replace with None marks this as forwarded without cloning
            let input = replace(&mut entry.input, None);
            let deadline = entry.deadline;

            if let Some(input) = input {
                match input {
                    Input::UserInput { update, .. } => {
                        // User input cancels running effects (clear_effects: true)

                        // No duration => one shot
                        pop_top_entry = deadline.is_none();

                        // Forward input
                        trace!("forwarding state update: {:#?}", update);
                        result = Some(MuxedInput::StateUpdate {
                            update,
                            clear_effects: true,
                        });
                    }
                    Input::EffectInput { update } => {
                        // No duration => one shot
                        pop_top_entry = deadline.is_none();

                        // Effect input, forward directly
                        trace!("forwarding state update: {:#?}", update);
                        result = Some(MuxedInput::StateUpdate {
                            update,
                            clear_effects: false,
                        });
                    }
                    Input::Effect { effect, .. } => {
                        // Remove effect entry so we can process user inputs
                        pop_top_entry = true;

                        // Launch effect request
                        result = Some(MuxedInput::LaunchEffect { effect, deadline });
                    }
                    Input::Internal(_) => panic!("unexpected internal command in input processing"),
                }
            }
        } else if expired_entries {
            // We expired entries and now there are none, clear everything
            return Poll::Ready(Some(MuxedInput::StateUpdate {
                update: StateUpdate::Clear,
                clear_effects: false,
            }));
        }

        // Pop one-shot top entry
        if pop_top_entry {
            self.inputs.pop();
        }

        // Return actual result
        if let Some(result) = result {
            return Poll::Ready(Some(result));
        }

        // Not ready, no input
        Poll::Pending
    }
}
