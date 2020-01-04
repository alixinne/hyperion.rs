//! Input priority muxer

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt;
use std::pin::Pin;

use std::time::Instant;

use futures::prelude::*;
use futures::task::Poll;

use crate::hyperion::{Input, InputDuration, ServiceCommand, StateUpdate};
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
        /// Time at which the update was requested
        update_time: Instant,
    },
    /// Effect launch request
    LaunchEffect {
        /// Details of the effect being launched
        effect: Effect,
        /// Duration of the effect
        duration: InputDuration,
    },
    /// Internal service update
    Internal(ServiceCommand),
}

impl fmt::Display for MuxedInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MuxedInput::StateUpdate {
                update,
                clear_effects,
                ..
            } => write!(
                f,
                "state update{} {:?}",
                if *clear_effects {
                    " (clearing effects)"
                } else {
                    ""
                },
                update
            ),
            MuxedInput::LaunchEffect { effect, .. } => write!(
                f,
                "effect launch {}",
                serde_json::to_string(effect).unwrap()
            ),
            MuxedInput::Internal(command) => write!(f, "internal {:?}", command),
        }
    }
}

/// Entry in the muxer queue
#[derive(Debug)]
struct MuxerEntry {
    /// Input data, None when it was sent as a StateUpdate
    input: Option<Input>,
    /// Expiration date of the entry
    duration: InputDuration,
    /// Priority of the entry
    priority: i32,
}

impl From<Input> for MuxerEntry {
    fn from(input: Input) -> Self {
        let duration = input
            .get_duration()
            .unwrap_or(InputDuration::from((Instant::now(), None)));

        // Default priority
        let priority = input.get_priority().unwrap_or(1000);

        Self {
            input: Some(input),
            duration,
            priority,
        }
    }
}

impl Eq for MuxerEntry {}

impl PartialEq for MuxerEntry {
    fn eq(&self, other: &Self) -> bool {
        self.duration == other.duration && self.priority == other.priority
    }
}

impl Ord for MuxerEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.duration.cmp(&other.duration))
            .reverse()
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
        let mut pushed_new_entries = false;

        // Receive incoming inputs
        while let Poll::Ready(value) = Stream::poll_next(Pin::new(&mut self.receiver), ctx) {
            if let Some(input) = value {
                // Forward internal commands directly
                if let Input::Internal(service_command) = input {
                    return Poll::Ready(Some(MuxedInput::Internal(service_command)));
                }

                // Push inputs into queue
                pushed_new_entries = true;
                self.inputs.push(input.into());
            } else {
                return Poll::Ready(None);
            }
        }

        if (pushed_new_entries) {
            trace!("inputs: {:#?}", self.inputs);
        }

        // Number of entries before processing
        let entries_before = self.inputs.len();

        let now = Instant::now();

        // Pop the top entry
        while let Some(mut entry) = self.inputs.pop() {
            // Check that the entry is not expired
            if entry.duration.is_expired(now) {
                // Go look at the next entry
                trace!("entry expired: {:?}", entry);
                continue;
            }

            // Pull the input from the entry
            if let Some(input) = entry.input.take() {
                trace!("new entry: {:?}", entry);

                let result = match input {
                    Input::UserInput { update, .. } => {
                        // User input, forward
                        MuxedInput::StateUpdate {
                            update,
                            update_time: entry.duration.start(),
                            clear_effects: true, // User input cancels running effects
                        }
                    }
                    Input::EffectInput { update } => {
                        // Effect input, forward
                        MuxedInput::StateUpdate {
                            update,
                            update_time: entry.duration.start(),
                            clear_effects: false, // Don't clear effects, this is an input from an effect
                        }
                    }
                    Input::Effect { effect, .. } => {
                        // Launch effect request
                        MuxedInput::LaunchEffect {
                            effect,
                            duration: entry.duration,
                        }
                    }
                    Input::Internal(_) => panic!("unexpected internal command in input processing"),
                };

                // If it's a clear, empty the whole input heap
                if let MuxedInput::StateUpdate {
                    update: StateUpdate::Clear,
                    ..
                } = &result
                {
                    trace!("clearing all entries");
                    self.inputs.clear();
                }

                if !entry.duration.is_oneshot() {
                    // Try to pop the following entries if we'll never see them
                    // i.e., if this entry will outlast them

                    if let Some(deadline) = entry.duration.deadline() {
                        // The current entry will expire
                        while self
                            .inputs
                            .peek()
                            .map(|item| {
                                item.priority == entry.priority
                                    || item.duration.is_expired(deadline)
                            })
                            .unwrap_or(false)
                        {
                            let entry = self.inputs.pop();
                            trace!("removed invisible entry: {:?}", entry);
                        }
                    } else {
                        // The current entry will never expire, so clear everything of the same
                        // priority
                        while self
                            .inputs
                            .peek()
                            .map(|item| item.priority == entry.priority)
                            .unwrap_or(false)
                        {
                            let entry = self.inputs.pop();
                            trace!("removed same-priority invisible entry: {:?}", entry);
                        }
                    }

                    // Push back the running operation on the heap
                    trace!("pushing back entry: {:?}", entry);
                    self.inputs.push(entry);
                }

                return Poll::Ready(Some(result));
            } else {
                // The input was already taken, so this is just a token for a running operation.
                // Push it back and stop popping entries.
                assert!(!entry.duration.is_oneshot());

                // Try to pop the following entries if we'll never see them
                // i.e., if this entry will outlast them

                if let Some(deadline) = entry.duration.deadline() {
                    // The current entry will expire
                    while self
                        .inputs
                        .peek()
                        .map(|item| {
                            item.priority == entry.priority || item.duration.is_expired(deadline)
                        })
                        .unwrap_or(false)
                    {
                        let entry = self.inputs.pop();
                        trace!("removed invisible entry: {:?}", entry);
                    }
                } else {
                    // The current entry will never expire, so clear everything of the same
                    // priority
                    while self
                        .inputs
                        .peek()
                        .map(|item| item.priority == entry.priority)
                        .unwrap_or(false)
                    {
                        let entry = self.inputs.pop();
                        trace!("removed same-priority invisible entry: {:?}", entry);
                    }
                }

                self.inputs.push(entry);
                break;
            }
        }

        // Check if we emptied the queue
        if entries_before > 0 && self.inputs.is_empty() {
            // We expired entries, clear everything
            return Poll::Ready(Some(MuxedInput::StateUpdate {
                update: StateUpdate::clear(),
                update_time: Instant::now(),
                clear_effects: false,
            }));
        }

        // Not ready, no input
        Poll::Pending
    }
}
