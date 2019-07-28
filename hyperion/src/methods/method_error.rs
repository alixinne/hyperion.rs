//! Definition of the MethodError type

use super::script;

/// Device method error
#[derive(Debug, Fail)]
pub enum MethodError {
    /// Wrapped I/O error
    #[fail(display = "I/O error: {}", error)]
    IoError {
        /// I/O error which triggered the MethodError
        error: std::io::Error,
    },
    /// Wrapped scripting engine error
    #[fail(display = "script error: {}", error)]
    ScriptError {
        /// I/O error which triggered the MethodError
        error: script::ScriptError,
    },
}

impl From<std::io::Error> for MethodError {
    fn from(error: std::io::Error) -> MethodError {
        MethodError::IoError { error }
    }
}

impl From<script::ScriptError> for MethodError {
    fn from(error: script::ScriptError) -> MethodError {
        MethodError::ScriptError { error }
    }
}
