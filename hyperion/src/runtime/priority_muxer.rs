//! Input priority muxer

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use std::mem::replace;
use std::time::Instant;

use futures::sync::mpsc;
use futures::{Async, Poll, Stream};

use crate::hyperion::{HyperionError, Input, StateUpdate};

/// Priority muxer
///
/// Type responsible for determining which update applies depending on durations and priorities.
pub struct PriorityMuxer {
    /// Input command receiver
    receiver: mpsc::UnboundedReceiver<Input>,
    /// Priority queue of inputs
    inputs: BinaryHeap<MuxerEntry>,
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
    pub fn new(receiver: mpsc::UnboundedReceiver<Input>) -> Self {
        Self {
            receiver,
            inputs: BinaryHeap::new(),
        }
    }
}

impl Stream for PriorityMuxer {
    type Item = StateUpdate;
    type Error = HyperionError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        // Receive incoming inputs
        while let Async::Ready(value) = self
            .receiver
            .poll()
            .map_err(|_| HyperionError::ChannelReceiveFailed)?
        {
            if let Some(input) = value {
                trace!("received new input {:?}", input);

                let entry = MuxerEntry::from(input);

                if entry.is_clearall() {
                    // Clear all pending messages and turn off everything
                    self.inputs.clear();
                    return Ok(Async::Ready(Some(StateUpdate::Clear)));
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
                trace!("forwarding input: {:?}", input);
                return Ok(Async::Ready(Some(input.into_update())));
            }
        } else if expired_entries {
            // We expired entries and now there are none, clear everything
            return Ok(Async::Ready(Some(StateUpdate::Clear)));
        }

        Ok(Async::NotReady)
    }
}
