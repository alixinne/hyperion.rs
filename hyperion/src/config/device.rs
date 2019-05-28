//! Definition of the Device type

use std::time::Duration;

use super::*;

fn default_frequency() -> f64 {
    10.0
}

/// Physical or virtual ambient lighting device representation
///
/// Devices in an Hyperion instance are uniquely identified by a name.
///
/// A device is defined by a set of Leds that can be contacted through at a given endpoint, either
/// local (USB, SPI, I2C device, etc.) or remote (UDP, MQTT, etc.).
///
/// All color transform and filtering settings are applied per device, since these characteristics
/// are defined for each device.
///
/// The device method is reponsible for transforming filtered color data into the target
/// representation for the physical device.
#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    /// Name of the device
    pub name: String,
    /// Target endpoint to contact
    pub endpoint: Endpoint,
    /// List of LED specifications
    pub leds: Vec<Led>,
    /// Update frequency (Hz)
    #[serde(default = "default_frequency")]
    pub frequency: f64,
    /// Idle timeout
    #[serde(default)]
    pub idle: IdleSettings,
}

impl Device {
    /// Ensures the configuration of the device is valid
    ///
    /// This will warn about invalid frequencies and idle timeouts.
    pub fn sanitize(&mut self) {
        // Clamp frequency to 1/hour Hz
        let mut freq = 1.0f64 / 3600f64;
        if self.frequency > freq {
            freq = self.frequency;
        } else {
            warn!(
                "device '{}': invalid frequency {}Hz",
                self.name, self.frequency
            );
        }

        self.frequency = freq;

        // Compute interval from frequency
        let update_duration = Duration::from_nanos((1_000_000_000f64 / self.frequency) as u64);

        // The idle timeout should be at least 2/frequency
        let mut idle_duration = self.idle.delay;
        if 2 * update_duration > idle_duration {
            warn!("device '{}': idle duration too short", self.name);
            idle_duration = 2 * update_duration;
        }

        self.idle.delay = idle_duration;

        // Sanitize idle settings
        self.idle.sanitize(&self.name);
    }
}

