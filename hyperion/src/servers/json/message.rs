#[derive(Debug, Deserialize)]
pub struct Adjustment {
    id: Option<String>,
    #[serde(rename = "redAdjust")]
    red_adjust: Option<[u8; 3]>,
    #[serde(rename = "greenAdjust")]
    green_adjust: Option<[u8; 3]>,
    #[serde(rename = "blueAdjust")]
    blue_adjust: Option<[u8; 3]>,
}

#[derive(Debug, Deserialize)]
pub struct Correction {
    id: Option<String>,
    #[serde(rename = "correctionValues")]
    correction_values: Option<[u8; 3]>,
}

#[derive(Debug, Deserialize)]
pub struct Effect {
    name: String,
    args: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Temperature {
    id: Option<String>,
    #[serde(rename = "correctionValues")]
    correction_values: Option<[u8; 3]>,
}

#[derive(Debug, Deserialize)]
pub struct Transform {
    id: Option<String>,
    #[serde(rename = "saturationGain")]
    saturation_gain: Option<f32>,
    #[serde(rename = "valueGain")]
    value_gain: Option<f32>,
    #[serde(rename = "saturationLGain")]
    saturation_lgain: Option<f32>,
    #[serde(rename = "luminanceGain")]
    luminance_gain: Option<f32>,
    #[serde(rename = "luminanceMinimum")]
    luminance_minimum: Option<f32>,
    threshold: Option<[f32; 3]>,
    gamma: Option<[f32; 3]>,
    blacklevel: Option<[f32; 3]>,
    whitelevel: Option<[f32; 3]>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum HyperionMessage {
    #[serde(rename = "adjustment")]
    Adjustment { adjustment: Adjustment },
    #[serde(rename = "clear")]
    Clear { priority: i32 },
    #[serde(rename = "clearall")]
    ClearAll,
    #[serde(rename = "color")]
    Color {
        priority: i32,
        duration: Option<i32>,
        color: Vec<u8>,
    },
    #[serde(rename = "correction")]
    Correction { correction: Correction },
    #[serde(rename = "effect")]
    Effect {
        priority: i32,
        duration: i32,
        effect: Effect,
    },
    #[serde(rename = "image")]
    Image {
        priority: i32,
        duration: Option<i32>,
        imagewidth: i32,
        imageheight: i32,
        imagedata: String,
    },
    #[serde(rename = "serverinfo")]
    ServerInfo,
    #[serde(rename = "temperature")]
    Temperature { temperature: Temperature },
    #[serde(rename = "transform")]
    Transform { transform: Transform },
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum HyperionResponse {
    SuccessResponse { success: bool },
    ErrorResponse { success: bool, error: String },
}
