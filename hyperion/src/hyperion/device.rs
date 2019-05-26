use super::Led;

use serde_yaml::Value;
use std::collections::BTreeMap as Map;

use std::fmt;

use std::time::Duration;

fn default_bit_depth() -> i32 {
    8
}

/// Device endpoint definition
///
/// An endpoint is defined by a method (how to contact the target device) and
/// parameters used by the method to determine the actual target (where to contact
/// the device).
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "target")]
pub enum Endpoint {
    #[serde(rename = "stdout")]
    Stdout {
        #[serde(default = "default_bit_depth")]
        bits: i32,
    },
    #[serde(rename = "udp")]
    Udp { address: String },
    #[serde(rename = "script")]
    Script {
        path: String,
        #[serde(flatten)]
        params: Map<String, Value>,
    },
}

fn default_frequency() -> f64 {
    10.0
}

struct DurationVisitor;

impl<'a> serde::de::Visitor<'a> for DurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("duration")
    }

    fn visit_str<A>(self, string: &str) -> Result<Self::Value, A>
    where
        A: serde::de::Error,
    {
        string
            .parse::<humantime::Duration>()
            .map(std::convert::Into::<Duration>::into)
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

fn hyperion_parse_duration<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(DurationVisitor {})
}

fn hyperion_write_duration<S>(
    duration: &Duration,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("{}", humantime::Duration::from(*duration)))
}

fn default_idle_delay() -> Duration {
    Duration::from_millis(10000)
}

fn default_idle_enabled() -> bool {
    true
}

fn default_idle_holds() -> bool {
    false
}

fn default_idle_resolution() -> u32 {
    16
}

fn default_idle_retries() -> u32 {
    5
}

/// Settings for idling devices
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdleSettings {
    /// Time before the device is considered idle (default: 10s)
    #[serde(
        serialize_with = "hyperion_write_duration",
        deserialize_with = "hyperion_parse_duration",
        default = "default_idle_delay"
    )]
    pub delay: Duration,
    /// true if the device should be idled, false otherwise (default: true)
    #[serde(default = "default_idle_enabled")]
    pub enabled: bool,
    /// true if the devices holds its color without updates, false otherwise (default: false)
    ///
    /// If false, the device will be updated on a timer with a `delay/2` period to keep the device
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
    fn sanitize(&mut self, device_name: &str) {
        if self.retries < 1 {
            warn!("device '{}': invalid idle retries, defaulted to 1", device_name);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_frequency() {
        let mut device = Device {
            name: "test".into(),
            endpoint: Endpoint::Stdout { bits: 8 },
            leds: Vec::new(),
            frequency: -2.0,
            idle: IdleSettings::default(),
        };

        device.sanitize();
        assert!(device.frequency >= 1.0f64 / 3600f64);
    }

    #[test]
    fn sanitize_idle() {
        let mut device = Device {
            name: "test".into(),
            endpoint: Endpoint::Stdout { bits: 8 },
            leds: Vec::new(),
            frequency: 1.0,
            idle: IdleSettings {
                delay: Duration::from_millis(5),
                .. Default::default()
            },
        };

        device.sanitize();
        assert!(device.idle.delay > Duration::from_millis((1_000f64 / device.frequency) as u64));
    }

    #[test]
    fn serialize_udp_endpoint() {
        let endpoint = Endpoint::Udp {
            address: "127.0.0.1:19446".into(),
        };
        println!(
            "udp endpoint: {}",
            serde_yaml::to_string(&endpoint).unwrap()
        );
    }
}
