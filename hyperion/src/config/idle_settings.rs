//! Definition of the IdleSettings type

use std::fmt;
use std::time::Duration;

/// Default idle delay
fn default_idle_delay() -> Duration {
    Duration::from_millis(5000)
}

/// Default idle enabled state
fn default_idle_enabled() -> bool {
    true
}

/// Default idle holds state
fn default_idle_holds() -> bool {
    false
}

/// Default idle min. change resolution
fn default_idle_resolution() -> u32 {
    16
}

/// Default idle device retries
fn default_idle_retries() -> u32 {
    5
}

/// Settings for idling devices
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdleSettings {
    /// Time before the device is considered idle (default: 5s)
    #[serde(
        serialize_with = "crate::serde::hyperion_write_duration",
        deserialize_with = "crate::serde::hyperion_parse_duration",
        default = "default_idle_delay"
    )]
    pub delay: Duration,
    /// true if the device should be idled, false otherwise (default: true)
    #[serde(default = "default_idle_enabled")]
    pub enabled: bool,
    /// true if the devices holds its color without updates, false otherwise (default: false)
    ///
    /// If false, the device will be updated on a timer with a `delay` period to keep the device
    /// active. Otherwise the device will not receive state updates as soon as it is considered
    /// idle, even when displaying a color.
    #[serde(default = "default_idle_holds")]
    pub holds: bool,
    /// Default idle value resolution, in bits (default: 16)
    ///
    /// Changes smaller in value than `2^(-resolution)` will not be considered as updates
    #[serde(default = "default_idle_resolution")]
    pub resolution: u32,
    /// Number of state updates to send on oneshot changes (default: 5)
    ///
    /// Specific devices may be unreliable, and may lose some state updates. This is true for UDP
    /// devices on unreliable networks for example. This means that single updates might not reach
    /// the physical device, so this setting ensures that at least `retries` are sent in those
    /// cases.
    #[serde(default = "default_idle_retries")]
    pub retries: u32,
}

impl IdleSettings {
    /// Ensures these idle settings are valid
    ///
    /// This will warn about invalid retries.
    ///
    /// # Parameters
    ///
    /// `device_name`: name of the device these settings are being sanitized for
    pub fn sanitize(&mut self, device_name: &str) {
        if self.retries < 1 {
            warn!(
                "device '{}': invalid idle retries, defaulted to 1",
                device_name
            );
            self.retries = 1;
        }
    }
}

impl Default for IdleSettings {
    fn default() -> Self {
        IdleSettings {
            delay: default_idle_delay(),
            enabled: default_idle_enabled(),
            holds: default_idle_holds(),
            resolution: default_idle_resolution(),
            retries: default_idle_retries(),
        }
    }
}

impl fmt::Display for IdleSettings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.enabled {
            write!(f, "{}", humantime::Duration::from(self.delay))
        } else {
            write!(f, "disabled")
        }
    }
}
