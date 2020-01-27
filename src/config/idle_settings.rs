//! Definition of the IdleSettings type

use std::fmt;
use std::time::Duration;

use validator::Validate;

/// Settings for idling devices
#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(default)]
pub struct IdleSettings {
    /// Time before the device is considered idle (default: 5s)
    #[serde(
        serialize_with = "crate::serde::hyperion_write_duration",
        deserialize_with = "crate::serde::hyperion_parse_duration"
    )]
    pub delay: Duration,
    /// true if the device should be idled, false otherwise (default: true)
    pub enabled: bool,
    /// true if the devices holds its color without updates, false otherwise (default: false)
    ///
    /// If false, the device will be updated on a timer with a `delay` period to keep the device
    /// active. Otherwise the device will not receive state updates as soon as it is considered
    /// idle, even when displaying a color.
    pub holds: bool,
    /// Default idle value resolution, in bits (default: 16)
    ///
    /// Changes smaller in value than `2^(-resolution)` will not be considered as updates
    pub resolution: u32,
    /// Number of state updates to send on oneshot changes (default: 5)
    ///
    /// Specific devices may be unreliable, and may lose some state updates. This is true for UDP
    /// devices on unreliable networks for example. This means that single updates might not reach
    /// the physical device, so this setting ensures that at least `retries` are sent in those
    /// cases.
    #[validate(range(min = 1))]
    pub retries: u32,
}

impl Default for IdleSettings {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(5000),
            enabled: true,
            holds: false,
            resolution: 16,
            retries: 5,
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
