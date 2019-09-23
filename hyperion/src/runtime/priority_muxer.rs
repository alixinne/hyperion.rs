//! Input priority muxer

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use std::mem::replace;
use std::time::{Duration, Instant};

use futures::{Async, Poll, Stream};

use crate::hyperion::{Input, ServiceCommand, ServiceInputReceiver, StateUpdate};
use crate::runtime::HostHandle;

/// Priority muxer
///
/// Type responsible for determining which update applies depending on durations and priorities.
pub struct PriorityMuxer {
    /// Input command receiver
    receiver: ServiceInputReceiver,
    /// Priority queue of inputs
    inputs: BinaryHeap<MuxerEntry>,
    /// Components host
    host: HostHandle,
}

/// Result of service inputs muxed by priority
pub enum MuxedInput {
    /// Lighting system state update
    StateUpdate(StateUpdate),
    /// Internal service update
    Internal(ServiceCommand),
}

impl From<StateUpdate> for MuxedInput {
    fn from(state_update: StateUpdate) -> Self {
        MuxedInput::StateUpdate(state_update)
    }
}

impl From<ServiceCommand> for MuxedInput {
    fn from(service_command: ServiceCommand) -> Self {
        MuxedInput::Internal(service_command)
    }
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
            host: HostHandle::new(),
        }
    }

    /// Get a reference to the host handle
    pub fn get_host_mut(&mut self) -> &mut HostHandle {
        &mut self.host
    }
}

impl Stream for PriorityMuxer {
    type Item = MuxedInput;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // Receive incoming inputs
        while let Async::Ready(value) = self.receiver.poll()? {
            if let Some(input) = value {
                trace!("received new input {:?}", input);

                // Forward internal commands directly
                if let Input::Internal(service_command) = input {
                    return Ok(Async::Ready(Some(service_command.into())));
                }

                // Push inputs into queue
                self.inputs.push(input.into());
            } else {
                return Ok(Async::Ready(None));
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
                        // User input cancels running effects
                        self.host.get_effect_engine().clear_all();

                        // No duration => one shot
                        pop_top_entry = deadline.is_none();

                        // Forward input
                        trace!("forwarding state update: {:?}", update);
                        result = Some(Ok(Async::Ready(Some(update.into()))));
                    }
                    Input::EffectInput { update } => {
                        // No duration => one shot
                        pop_top_entry = deadline.is_none();

                        // Effect input, forward directly
                        trace!("forwarding state update: {:?}", update);
                        result = Some(Ok(Async::Ready(Some(update.into()))));
                    }
                    Input::Effect { effect, .. } => {
                        let mut ee = self.host.get_effect_engine();

                        let name = effect.name.clone();
                        let args = effect.args.clone();

                        // Remove effect entry so we can process user inputs
                        pop_top_entry = true;

                        // Launch effect request
                        match ee.launch(
                            effect,
                            Some(deadline.unwrap_or_else(|| {
                                Instant::now() + Duration::from_millis(1000 * 60 * 10)
                            })),
                            self.host.get_service_input_sender(),
                            self.host.get_devices().get_led_count(),
                        ) {
                            Ok(()) => debug!(
                                "launched effect {} with args {}",
                                name,
                                args.map(|a| serde_json::to_string(&a).unwrap())
                                    .unwrap_or_else(|| "null".to_owned())
                            ),
                            Err(error) => warn!("failed to launch effect {}: {}", name, error),
                        }
                    }
                    Input::Internal(_) => panic!("unexpected internal command in input processing"),
                }
            }
        } else if expired_entries {
            // We expired entries and now there are none, clear everything
            return Ok(Async::Ready(Some(StateUpdate::Clear.into())));
        }

        // Pop one-shot top entry
        if pop_top_entry {
            self.inputs.pop();
        }

        // Return actual result
        if let Some(result) = result {
            return result;
        }

        // Not ready, no input
        Ok(Async::NotReady)
    }
}
