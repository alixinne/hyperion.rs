//! Definition of the Configuration type

use super::Device;

/// Configuration for an Hyperion instance
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    /// List of devices for this configuration
    pub devices: Vec<Device>,
}

impl Configuration {
    /// Ensures the configuration is well-formed
    ///
    /// This will issue warnings for fields that need to be changed.
    pub fn sanitize(&mut self) {
        for device in &mut self.devices {
            device.sanitize();
        }
    }
}
