//! Definition of the HyperionError type

use crate::methods;

#[derive(Debug, Fail)]
pub enum HyperionError {
    #[fail(display = "failed to receive update from channel")]
    ChannelReceiveFailed,
    #[fail(display = "failed to poll the updater interval")]
    UpdaterPollFailed,
    #[fail(display = "failed to initialize LED devices: {}", error)]
    LedDeviceInitFailed { error: methods::MethodError },
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
