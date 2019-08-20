//! Definition of the Device type

use std::time::Duration;

use regex::Regex;
use validator::{Validate, ValidationError};

use super::*;

/// Default frequency for a device
fn default_frequency() -> f64 {
    10.0
}

lazy_static! {
    static ref NAME_REGEX: Regex = Regex::new(r"\S").unwrap();
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
#[derive(Debug, Validate, Serialize, Deserialize)]
#[validate(schema(function = "validate_device"))]
pub struct Device {
    /// Name of the device
    #[validate(regex = "NAME_REGEX")]
    pub name: String,
    /// Target endpoint to contact
    #[validate]
    pub endpoint: Endpoint,
    /// List of LED specifications
    #[validate]
    pub leds: Vec<Led>,
    /// Update frequency (Hz)
    #[serde(default = "default_frequency")]
    #[validate(range(min = 0.0))]
    pub frequency: f64,
    /// Idle timeout
    #[serde(default)]
    #[validate]
    pub idle: IdleSettings,
    /// Filtering method
    #[serde(default)]
    #[validate]
    pub filter: Filter,
    /// Color format
    #[serde(default)]
    #[validate]
    pub format: ColorFormat,
}

/// Ensures the configuration of the device is valid
fn validate_device(device: &Device) -> Result<(), ValidationError> {
    // Clamp frequency to 1/hour Hz
    let freq = 1.0f64 / 3600f64;
    if device.frequency <= freq {
        return Err(ValidationError::new("invalid_frequency"));
    }

    // Compute interval from frequency
    let update_duration = Duration::from_nanos((1_000_000_000f64 / device.frequency) as u64);

    // The idle timeout should be at least 2/frequency
    let idle_duration = device.idle.delay;
    if 2 * update_duration > idle_duration {
        let mut error = ValidationError::new("invalid_idle_duration");
        error.add_param(
            "minimum_duration".into(),
            &humantime::Duration::from(2 * update_duration).to_string(),
        );
        return Err(error);
    }

    Ok(())
}
