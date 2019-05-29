//! Definition of the Endpoint type

use serde_yaml::Value;
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
#[serde(tag = "type")]
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
