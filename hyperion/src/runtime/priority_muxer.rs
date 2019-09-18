//! Input priority muxer

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use std::mem::replace;
use std::time::Instant;

use std::sync::{Arc, Mutex};

use futures::sync::mpsc;
use futures::{Async, Poll, Stream};

use crate::hyperion::{
    HyperionError, HyperionErrorKind, Input, ServiceCommand, ServiceInputSender, StateUpdate,
};
use crate::runtime::EffectEngine;

/// Priority muxer
///
/// Type responsible for determining which update applies depending on durations and priorities.
pub struct PriorityMuxer {
    /// Input command receiver
    receiver: mpsc::UnboundedReceiver<Input>,
    /// Priority queue of inputs
    inputs: BinaryHeap<MuxerEntry>,
    /// Effect engine
    effect_engine: Arc<Mutex<EffectEngine>>,
    /// Sender for effect inputs
    sender: ServiceInputSender,
    /// Number of LEDs for effects
    led_count: usize,
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
    deadline: Instant,
    /// Priority of the entry
    priority: i32,
}

impl MuxerEntry {
    /// Returns true if this input entry is a ClearAll message
    fn is_clearall(&self) -> bool {
        if let Some(Input::OneShot(StateUpdate::Clear)) = self.input {
            return true;
        }

        false
    }
}

impl From<Input> for MuxerEntry {
    fn from(input: Input) -> Self {
        let deadline = Instant::now() + input.get_duration();
        let priority = input.get_priority();

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
    /// * `effect_engine`: handle to the effect engine for this instance
    pub fn new(
        receiver: mpsc::UnboundedReceiver<Input>,
        effect_engine: Arc<Mutex<EffectEngine>>,
        sender: ServiceInputSender,
        led_count: usize,
    ) -> Self {
        Self {
            receiver,
            inputs: BinaryHeap::new(),
            effect_engine,
            sender,
            led_count,
        }
    }
}

impl Stream for PriorityMuxer {
    type Item = MuxedInput;
    type Error = HyperionError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // Receive incoming inputs
        while let Async::Ready(value) = self
            .receiver
            .poll()
            .map_err(|_| HyperionError::from(HyperionErrorKind::ChannelReceive))?
        {
            if let Some(input) = value {
                trace!("received new input {:?}", input);

                // Forward internal commands directly
                if let Input::Internal(service_command) = input {
                    return Ok(Async::Ready(Some(service_command.into())));
                }

                let entry = MuxerEntry::from(input);

                if entry.is_clearall() {
                    // Clear all pending messages and turn off everything
                    self.inputs.clear();
                    // Clear all effects
                    self.effect_engine.lock().unwrap().clear_all();

                    return Ok(Async::Ready(Some(StateUpdate::Clear.into())));
                } else {
                    // Other message, add to queue
                    self.inputs.push(entry);
                }
            } else {
                return Ok(Async::Ready(None));
            }
        }

        let now = Instant::now();
        let mut expired_entries = false;

        // Remove expired inputs
        while let Some(entry) = self.inputs.peek() {
            if entry.deadline < now {
                trace!("input {:?} has expired", entry.input);
                self.inputs.pop();
                expired_entries = true;
            } else {
                break;
            }
        }

        // Send non-forwarded top input if any
        if let Some(mut entry) = self.inputs.peek_mut() {
            // Replace with None marks this as forwarded without cloning
            let input = replace(&mut entry.input, None);

            if let Some(input) = input {
                if input.is_update() {
                    trace!("forwarding input: {:?}", input);
                    return Ok(Async::Ready(Some(input.into_update().into())));
                } else if let Input::Effect {
                    effect,
                    priority,
                    duration,
                } = input
                {
                    let name = effect.name.clone();
                    let deadline = duration.map(|d| now + d);

                    let mut ee = self.effect_engine.lock().unwrap();

                    // Stop current effects
                    ee.clear_all();

                    // Start next one
                    match ee.launch(
                        effect,
                        priority,
                        deadline,
                        self.sender.clone(),
                        self.led_count,
                    ) {
                        Ok(()) => debug!("launched effect {}", name),
                        Err(error) => warn!("failed to launch effect {}: {}", name, error),
                    }
                }
            }
        } else if expired_entries {
            // We expired entries and now there are none, clear everything
            return Ok(Async::Ready(Some(StateUpdate::Clear.into())));
        }

        Ok(Async::NotReady)
    }
}
