use std::fmt;

use serde_derive::{Deserialize, Serialize};

/// Change color adjustement values
#[derive(Debug, Deserialize)]
pub struct Adjustment {
    /// Adjustment name
    id: Option<String>,
    /// Red channel adjustment
    #[serde(rename = "redAdjust")]
    red_adjust: Option<[u8; 3]>,
    /// Green channel adjustment
    #[serde(rename = "greenAdjust")]
    green_adjust: Option<[u8; 3]>,
    /// Blue channel adjustment
    #[serde(rename = "blueAdjust")]
    blue_adjust: Option<[u8; 3]>,
}

/// Change color correction values
#[derive(Debug, Deserialize)]
pub struct Correction {
    /// Correction name
    id: Option<String>,
    /// RGB Correction values
    #[serde(rename = "correctionValues")]
    correction_values: Option<[u8; 3]>,
}

/// Trigger an effect by name
#[derive(Debug, Deserialize)]
pub struct Effect {
    /// Effect name
    name: String,
    /// Effect parameters
    args: Option<serde_json::Value>,
}

/// Change color temperature values
#[derive(Debug, Deserialize)]
pub struct Temperature {
    /// Temperature correction name
    id: Option<String>,
    /// RGB temperature values
    #[serde(rename = "correctionValues")]
    correction_values: Option<[u8; 3]>,
}

/// Change color transform values
#[derive(Debug, Deserialize)]
pub struct Transform {
    /// Color transform name
    id: Option<String>,
    /// HSV Saturation gain
    #[serde(rename = "saturationGain")]
    saturation_gain: Option<f32>,
    /// HSV Value gain
    #[serde(rename = "valueGain")]
    value_gain: Option<f32>,
    /// HSV Saturation-Luminance gain
    #[serde(rename = "saturationLGain")]
    saturation_lgain: Option<f32>,
    /// HSV Luminance gain
    #[serde(rename = "luminanceGain")]
    luminance_gain: Option<f32>,
    /// Minimum luminance
    #[serde(rename = "luminanceMinimum")]
    luminance_minimum: Option<f32>,
    /// Transform threshold
    threshold: Option<[f32; 3]>,
    /// Transform gamma
    gamma: Option<[f32; 3]>,
    /// Transform black level
    blacklevel: Option<[f32; 3]>,
    /// Transform white level
    whitelevel: Option<[f32; 3]>,
}

/// Serde visitor for deserializing Base64-encoded values
struct Base64Visitor;

impl<'a> serde::de::Visitor<'a> for Base64Visitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("base64 image")
    }

    fn visit_str<A>(self, string: &str) -> Result<Self::Value, A>
    where
        A: serde::de::Error,
    {
        base64::decode(string).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

/// Decode a base64-encoded value
///
/// # Parameters
///
/// `deserializer`: Serde deserializer
fn from_base64<'de, D>(deserializer: D) -> std::result::Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_str(Base64Visitor {})
}

/// Incoming Hyperion JSON message
#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum HyperionMessage {
    /// Change color adjustement values
    #[serde(rename = "adjustment")]
    Adjustment {
        /// Adjustment parameters
        adjustment: Adjustment,
    },
    /// Clear LED values
    #[serde(rename = "clear")]
    Clear {
        /// Command priority
        priority: i32,
    },
    /// Clear all LED values
    #[serde(rename = "clearall")]
    ClearAll,
    /// Set LEDs to a given color
    #[serde(rename = "color")]
    Color {
        /// Command priority
        priority: i32,
        /// Command duration
        duration: Option<i32>,
        /// Color to set
        color: Vec<u8>,
    },
    /// Change color correction values
    #[serde(rename = "correction")]
    Correction {
        /// Correction parameters
        correction: Correction,
    },
    /// Trigger an effect by name
    #[serde(rename = "effect")]
    Effect {
        /// Command priority
        priority: i32,
        /// Command duration
        duration: i32,
        /// Effect parameters
        effect: Effect,
    },
    /// Incoming image data
    #[serde(rename = "image")]
    Image {
        /// Command priority
        priority: i32,
        /// Command duration
        duration: Option<i32>,
        /// Raw image width
        imagewidth: u32,
        /// Raw image height
        imageheight: u32,
        /// Raw image data
        #[serde(deserialize_with = "from_base64")]
        imagedata: Vec<u8>,
    },
    /// Request for server information
    #[serde(rename = "serverinfo")]
    ServerInfo,
    /// Change color temperature values
    #[serde(rename = "temperature")]
    Temperature {
        /// Temperature parameters
        temperature: Temperature,
    },
    /// Change color transform values
    #[serde(rename = "transform")]
    Transform {
        /// Transform parameters
        transform: Transform,
    },
}

/// Hyperion JSON response
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum HyperionResponse {
    /// Success response
    SuccessResponse {
        /// Success value (should be true)
        success: bool,
    },
    /// Error response
    ErrorResponse {
        /// Success value (should be false)
        success: bool,
        /// Error message
        error: String,
    },
}