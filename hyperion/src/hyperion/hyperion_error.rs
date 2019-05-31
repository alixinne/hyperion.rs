//! Definition of the HyperionError type

use crate::methods;

/// Hyperion instance errors
#[derive(Debug, Fail)]
pub enum HyperionError {
    /// StateUpdate channel receive error
    #[fail(display = "failed to receive update from channel")]
    ChannelReceiveFailed,
    /// Device Interval polling error
    #[fail(display = "failed to poll the updater interval")]
    UpdaterPollFailed,
    /// Device initialization failed
    #[fail(display = "failed to initialize LED devices: {}", error)]
    LedDeviceInitFailed {
        /// Device method error which caused this HyperionError
        error: methods::MethodError,
    },
}

impl From<methods::MethodError> for HyperionError {
    fn from(error: methods::MethodError) -> Self {
        HyperionError::LedDeviceInitFailed { error }
    }
}

impl From<tokio::timer::Error> for HyperionError {
    fn from(_error: tokio::timer::Error) -> Self {
        HyperionError::UpdaterPollFailed
    }
}
