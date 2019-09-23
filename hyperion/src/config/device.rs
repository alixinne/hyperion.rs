//! Definition of the Device type

use std::time::Duration;

use regex::Regex;
use validator::{Validate, ValidationError};

use super::*;

/// Default enabled value
fn default_enabled() -> bool {
    true
}

/// Default frequency for a device
fn default_frequency() -> f64 {
    10.0
}

/// Default device latency
fn default_latency() -> Duration {
    Duration::from_millis(0)
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
#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[validate(schema(function = "validate_device"))]
pub struct Device {
    /// True if this device is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
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
    /// Update latency
    #[serde(
        serialize_with = "crate::serde::hyperion_write_duration",
        deserialize_with = "crate::serde::hyperion_parse_duration",
        default = "default_latency"
    )]
    pub latency: Duration,
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

/// Update to a device instance
#[derive(Debug, Deserialize)]
pub struct DeviceUpdate {
    /// True if this device is enabled
    pub enabled: Option<bool>,
    /// Name of the device
    pub name: Option<String>,
    /// Target endpoint to contact
    pub endpoint: Option<Endpoint>,
    /// List of LED specifications
    pub leds: Option<Vec<Led>>,
    /// Update frequency (Hz)
    pub frequency: Option<f64>,
    /// Update latency
    pub latency: Option<Duration>,
    /// Idle timeout
    pub idle: Option<IdleSettings>,
    /// Filtering method
    pub filter: Option<Filter>,
    /// Color format
    pub format: Option<ColorFormat>,
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

macro_rules! update_field {
    ($field:ident, $device_update:ident, $cloned_self:ident, $changed_flags:ident, $extra_bits:expr) => {
        if let Some($field) = $device_update.$field {
            $cloned_self.$field = $field;
            $changed_flags |= $extra_bits;
        }
    };
}

impl Device {
    /// Update a device configuration
    ///
    /// # Parameters
    ///
    /// * `device_update`: set of possible updates to the device configuration
    pub fn update(
        &mut self,
        device_update: DeviceUpdate,
    ) -> Result<ReloadHints, validator::ValidationErrors> {
        // Clone self
        let mut cloned_self = self.clone();
        let mut changed_flags = ReloadHints::empty();

        // Apply changes
        update_field!(
            enabled,
            device_update,
            cloned_self,
            changed_flags,
            ReloadHints::DEVICE_GENERIC
        );
        update_field!(
            name,
            device_update,
            cloned_self,
            changed_flags,
            ReloadHints::DEVICE_GENERIC
        );
        update_field!(
            endpoint,
            device_update,
            cloned_self,
            changed_flags,
            ReloadHints::DEVICE_ENDPOINT
        );
        update_field!(
            leds,
            device_update,
            cloned_self,
            changed_flags,
            ReloadHints::DEVICE_LEDS
        );
        update_field!(
            frequency,
            device_update,
            cloned_self,
            changed_flags,
            ReloadHints::DEVICE_FREQUENCY
        );
        update_field!(
            latency,
            device_update,
            cloned_self,
            changed_flags,
            ReloadHints::DEVICE_LATENCY
        );
        update_field!(
            idle,
            device_update,
            cloned_self,
            changed_flags,
            ReloadHints::DEVICE_IDLE
        );
        update_field!(
            filter,
            device_update,
            cloned_self,
            changed_flags,
            ReloadHints::DEVICE_FILTER
        );
        update_field!(
            format,
            device_update,
            cloned_self,
            changed_flags,
            ReloadHints::DEVICE_FORMAT
        );

        // Validate changes
        cloned_self.validate()?;

        *self = cloned_self;
        Ok(changed_flags)
    }
}
