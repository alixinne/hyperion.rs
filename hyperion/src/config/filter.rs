//! Definition of the Filter type

/// Default linear filter frequency
fn default_linear_frequency() -> f32 {
    30.0
}

/// Temporal filter definition
///
/// Specifies how LED values should be filtered before being sent to the device.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Filter {
    #[serde(rename = "none")]
    /// No filter is used, the last sampled value is used directly
    Nearest,
    #[serde(rename = "linear")]
    /// First-order linear filter, combines the current and previous sample
    Linear {
        /// Width of the linear filter window, in Hz
        ///
        /// The recommended value is the update frequency of the target device.
        #[serde(rename = "frequency", default = "default_linear_frequency")]
        frequency: f32,
    },
}

impl Default for Filter {
    fn default() -> Self {
        Filter::Nearest
    }
}
