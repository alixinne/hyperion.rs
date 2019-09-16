//! Definition of the MethodError type

/// Device method error
#[derive(Debug, Fail)]
pub enum MethodError {
    /// Wrapped I/O error
    #[fail(display = "I/O error: {}", error)]
    IoError {
        /// I/O error which triggered the MethodError
        error: std::io::Error,
    },
}

impl From<std::io::Error> for MethodError {
    fn from(error: std::io::Error) -> MethodError {
        MethodError::IoError { error }
    }
}
