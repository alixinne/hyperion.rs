//! Definition of the Endpoint type

use serde_yaml::Value;
use std::collections::BTreeMap as Map;
use validator::{Validate, ValidationError, ValidationErrors};

/// Default stdout method bit depth
fn default_bit_depth() -> i32 {
    8
}

/// Device endpoint definition
///
/// An endpoint is defined by a method (how to contact the target device) and
/// parameters used by the method to determine the actual target (where to contact
/// the device).
#[derive(Clone, Debug, Serialize, Deserialize)]
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

impl Validate for Endpoint {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            Endpoint::Stdout { bits } => {
                if *bits < 1 {
                    let mut errors = ValidationErrors::new();
                    errors.add("bits", ValidationError::new("bits must be greater than 0"));
                    return Err(errors);
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}
