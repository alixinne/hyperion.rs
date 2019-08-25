//! ConfigLoadError type definition

/// Config loading error
#[derive(Debug, Fail)]
pub enum ConfigLoadError {
    /// I/O error
    #[fail(display = "an i/o error occurred: {}", 0)]
    IoError(std::io::Error),
    /// Deserialization error
    #[fail(display = "invalid syntax: {}", 0)]
    InvalidSyntax(serde_yaml::Error),
    /// Validator error
    #[fail(display = "failed to validate config: {}", 0)]
    InvalidConfig(validator::ValidationErrors),
}

impl From<std::io::Error> for ConfigLoadError {
    fn from(error: std::io::Error) -> Self {
        ConfigLoadError::IoError(error)
    }
}

impl From<serde_yaml::Error> for ConfigLoadError {
    fn from(error: serde_yaml::Error) -> Self {
        ConfigLoadError::InvalidSyntax(error)
    }
}

impl From<validator::ValidationErrors> for ConfigLoadError {
    fn from(error: validator::ValidationErrors) -> Self {
        ConfigLoadError::InvalidConfig(error)
    }
}
