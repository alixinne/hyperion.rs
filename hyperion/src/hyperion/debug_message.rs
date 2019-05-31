//! Definition of the DebugMessage type

use super::StateUpdate;

/// Messages sent to the debug monitor
pub enum DebugMessage {
    /// A state update forwarded from one of the sources
    StateUpdate(StateUpdate),
    /// The hyperion instance is terminating
    Terminating,
}
