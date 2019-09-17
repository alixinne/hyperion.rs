//! Definition of the EffectError type

/// Errors occurring when running an effect
#[derive(Debug, Fail)]
pub enum EffectError {
    /// Requested effect not found
    #[fail(display = "effect '{}' was not found", 0)]
    NotFound(String),
    /// I/O error
    #[fail(display = "i/o error: {}", 0)]
    IoError(std::io::Error),
}

impl From<std::io::Error> for EffectError {
    fn from(error: std::io::Error) -> Self {
        EffectError::IoError(error)
    }
}
