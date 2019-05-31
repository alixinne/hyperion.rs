//! Definition of the Endpoint type

use serde_yaml::Value;
use std::collections::BTreeMap as Map;

/// Default stdout method bit depth
fn default_bit_depth() -> i32 {
    8
}

/// Device endpoint definition
///
/// An endpoint is defined by a method (how to contact the target device) and
/// parameters used by the method to determine the actual target (where to contact
/// the device).
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Endpoint {
    /// Logging output (requires stdout.lua)
    #[serde(rename = "stdout")]
    Stdout {
        /// Bit depth for the output values
        #[serde(default = "default_bit_depth")]
        bits: i32,
    },
    /// UDP protocol method
    #[serde(rename = "udp")]
    Udp {
        /// Device address
        address: String,
    },
    /// Scripting engine method
    #[serde(rename = "script")]
    Script {
        /// Script path
        path: String,
        /// Script parameters
        #[serde(flatten)]
        params: Map<String, Value>,
    },
}
