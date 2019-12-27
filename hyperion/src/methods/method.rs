use crate::runtime::LedData;

/// Error raised when the method failed to write to the target device
pub enum WriteError {
    /// The target device is not ready yet. The caller should try writing again soon using updated
    /// data
    NotReady,
    /// The target device failed to initialize. The caller should try writing again later (for
    /// example if the device comes back online).
    Errored {
        /// Error message
        error: String,
    },
}

/// Result of a method write operation
pub type WriteResult = Result<(), WriteError>;

/// Trait for methods to write LED data to devices
#[async_trait]
pub trait Method {
    /// Write the current LED data to the device
    ///
    /// # Parameters
    ///
    /// * `led_data`: formatted and filtered LED data to be written to the device
    ///
    /// # Returns
    ///
    /// Result indicating if the write was successful, or should be tried again later because of a
    /// delay or an error.
    async fn write(&mut self, led_data: &Vec<LedData>) -> WriteResult;
}
