//! ServiceCommand type definition

use crate::config::ReloadHints;

/// Represents a command for the hyperion service instance
#[derive(Debug, Clone)]
pub enum ServiceCommand {
    /// A device configuration changed
    ReloadDevice {
        /// Index of the changed device
        device_index: usize,
        /// Details of the change
        reload_hints: ReloadHints,
    },
    /// An effect finished running
    EffectCompleted {
        /// Name of the effect that completed
        name: String,
        /// Result of running the effect
        result: Result<(), String>,
    },
}
