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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Effect {
    /// Effect name
    pub name: String,
    /// Effect parameters
    pub args: Option<serde_json::Value>,
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
    #[serde(rename = "lightnessGain")]
    lightness_gain: Option<f32>,
    /// Minimum lightness
    #[serde(rename = "lightnessMinimum")]
    lightness_minimum: Option<f32>,
    /// Transform threshold
    threshold: Option<[f32; 3]>,
    /// Transform gamma
    gamma: Option<[f32; 3]>,
    /// Transform black level
    blacklevel: Option<[f32; 3]>,
    /// Transform white level
    whitelevel: Option<[f32; 3]>,
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
        duration: Option<i32>,
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
        imagewidth: i32,
        /// Raw image height
        imageheight: i32,
        /// Raw image data
        #[serde(deserialize_with = "crate::serde::from_base64")]
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

/// Effect definition details
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectDefinition {
    /// User-friendly name of the effect
    pub name: String,
    /// Path to the script to run
    pub script: String,
    /// Extra script arguments
    pub args: serde_json::Value,
}

/// Hyperion build info
#[derive(Debug, Serialize)]
pub struct BuildInfo {
    /// Version number
    version: String,
    /// Build time
    time: String,
}

/// Hyperion server info
#[derive(Debug, Serialize)]
pub struct ServerInfo {
    /// Server hostname
    hostname: String,
    /// Effects
    effects: Vec<EffectDefinition>,
    /// Build info
    hyperion_build: BuildInfo,

    /// Priority information (array)
    priorities: serde_json::Value,
    /// Color correction information (array)
    correction: serde_json::Value,
    /// Temperature correction information (array)
    temperature: serde_json::Value,
    /// Transform correction information (array)
    adjustment: serde_json::Value,
    /// Active effect info (array)
    #[serde(rename = "activeEffects")]
    active_effects: serde_json::Value,
    /// Active static LED color (array)
    #[serde(rename = "activeLedColor")]
    active_led_color: serde_json::Value,
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
    /// Server information response
    ServerInfoResponse {
        /// Success value (should be true)
        success: bool,
        /// Server information
        // Box because of large size difference
        info: Box<ServerInfo>,
    },
}

impl HyperionResponse {
    /// Return a success response
    pub fn success() -> Self {
        HyperionResponse::SuccessResponse { success: true }
    }

    /// Return an error response
    pub fn error(error: impl ToString) -> Self {
        HyperionResponse::ErrorResponse {
            success: false,
            error: error.to_string(),
        }
    }

    /// Return a server information response
    pub fn server_info(hostname: String, effects: Vec<EffectDefinition>, version: String) -> Self {
        use serde_json::json;

        HyperionResponse::ServerInfoResponse {
            success: true,
            info: Box::new(ServerInfo {
                hostname,
                effects,
                hyperion_build: BuildInfo {
                    version,
                    time: "".to_owned(),
                },

                priorities: json!([]),
                correction: json!([]),
                temperature: json!([]),
                adjustment: json!([]),
                active_effects: json!([]),
                active_led_color: json!([]),
            }),
        }
    }
}
