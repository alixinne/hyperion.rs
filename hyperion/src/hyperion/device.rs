use super::Led;

use serde_json::Value;
use std::collections::BTreeMap as Map;

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

fn default_frequency() -> u32 {
    10
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
    pub frequency: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_udp_endpoint() {
        let endpoint = Endpoint::Udp {
            address: "127.0.0.1:19446".into(),
        };
        println!(
            "udp endpoint: {}",
            serde_json::to_string(&endpoint).unwrap()
        );
    }
}
