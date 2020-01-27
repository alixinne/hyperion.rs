//! ServiceCommand type definition

/// Represents a command for the hyperion service instance
#[derive(Debug, Clone)]
pub enum ServiceCommand {
    /// An effect finished running
    EffectCompleted {
        /// Name of the effect that completed
        name: String,
        /// Result of running the effect
        result: Result<(), String>,
    },
}
