use serde_derive::{Deserialize, Serialize};
use serde_json::json;
use validator::Validate;

use crate::models::Color as RgbColor;

/// Change color adjustement values
#[derive(Debug, Deserialize, Validate)]
pub struct Adjustment {
    #[validate]
    pub adjustment: ChannelAdjustment,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChannelAdjustment {
    pub id: Option<String>,
    pub white: RgbColor,
    pub red: RgbColor,
    pub green: RgbColor,
    pub blue: RgbColor,
    pub cyan: RgbColor,
    pub magenta: RgbColor,
    pub yellow: RgbColor,
    #[validate(range(min = 0, max = 100))]
    pub backlight_threshold: i32,
    pub backlight_colored: bool,
    #[validate(range(min = 0, max = 100))]
    pub brightness: i32,
    #[validate(range(min = 0, max = 100))]
    pub brightness_compensation: i32,
    #[validate(range(min = 0.1, max = 5.0))]
    pub gamma_red: f32,
    #[validate(range(min = 0.1, max = 5.0))]
    pub gamma_green: f32,
    #[validate(range(min = 0.1, max = 5.0))]
    pub gamma_blue: f32,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "camelCase")]
pub enum AuthorizeCommand {
    RequestToken,
    CreateToken,
    RenameToken,
    DeleteToken,
    GetTokenList,
    Logout,
    Login,
    TokenRequired,
    AdminRequired,
    NewPasswordRequired,
    NewPassword,
    AnswerRequest,
    GetPendingTokenRequests,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Authorize {
    pub subcommand: AuthorizeCommand,
    #[validate(length(min = 8))]
    pub password: Option<String>,
    #[validate(length(min = 8))]
    pub new_password: Option<String>,
    #[validate(length(min = 36))]
    pub token: Option<String>,
    #[validate(length(min = 5))]
    pub comment: Option<String>,
    #[validate(length(min = 5, max = 5))]
    pub id: Option<String>,
    pub accept: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Clear {
    #[validate(range(min = -1, max = 253))]
    pub priority: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Color {
    #[validate(range(min = 1, max = 253))]
    pub priority: i32,
    /// Duration in miliseconds
    #[validate(range(min = 0))]
    pub duration: Option<i32>,
    /// Origin for the command
    #[validate(length(min = 4, max = 20))]
    pub origin: Option<String>,
    pub color: RgbColor,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Component {
    All,
    Smoothing,
    BlackBorder,
    Forwarder,
    BoblightServer,
    Grabber,
    V4L,
    LedDevice,
}

#[derive(Debug, Deserialize)]
pub struct ComponentStatus {
    pub component: Component,
    pub state: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ComponentState {
    pub componentstate: ComponentStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigCommand {
    SetConfig,
    GetConfig,
    GetSchema,
    Reload,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Config {
    pub subcommand: ConfigCommand,
    #[serde(default)]
    pub config: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EffectCreate {
    pub name: String,
    pub script: String,
    pub args: serde_json::Map<String, serde_json::Value>,
    #[serde(deserialize_with = "crate::serde::from_base64")]
    pub image_data: Vec<u8>,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct EffectDelete {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct EffectRequest {
    /// Effect name
    pub name: String,
    /// Effect parameters
    #[serde(default)]
    pub args: serde_json::Map<String, serde_json::Value>,
}

/// Trigger an effect by name
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Effect {
    #[validate(range(min = 1, max = 253))]
    pub priority: i32,
    #[validate(range(min = 0))]
    pub duration: Option<i32>,
    #[validate(length(min = 4, max = 20))]
    pub origin: Option<String>,
    pub effect: EffectRequest,
    pub python_script: Option<String>,
    #[serde(deserialize_with = "crate::serde::from_base64")]
    pub image_data: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Auto,
}

impl Default for ImageFormat {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct Image {
    #[validate(range(min = 1, max = 253))]
    pub priority: i32,
    #[validate(length(min = 4, max = 20))]
    pub origin: Option<String>,
    #[validate(range(min = 0))]
    pub duration: Option<i32>,
    pub imagewidth: u32,
    pub imageheight: u32,
    #[serde(deserialize_with = "crate::serde::from_base64")]
    pub imagedata: Vec<u8>,
    #[serde(default)]
    pub format: ImageFormat,
    #[validate(range(min = 25, max = 2000))]
    pub scale: Option<i32>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstanceCommand {
    CreateInstance,
    DeleteInstance,
    StartInstance,
    StopInstance,
    SaveName,
    SwitchTo,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Instance {
    pub subcommand: InstanceCommand,
    #[validate(range(max = 255))]
    pub instance: Option<u32>,
    #[validate(length(min = 5))]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LedColorsSubcommand {
    #[serde(rename = "ledstream-stop")]
    LedStreamStop,
    #[serde(rename = "ledstream-start")]
    LedStreamStart,
    TestLed,
    #[serde(rename = "imagestream-start")]
    ImageStreamStart,
    #[serde(rename = "imagestream-stop")]
    ImageStreamStop,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LedColors {
    pub subcommand: LedColorsSubcommand,
    pub oneshot: Option<bool>,
    #[validate(range(min = 50))]
    pub interval: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LedDeviceCommand {
    Discover,
    GetProperties,
    Identify,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LedDevice {
    pub subcommand: LedDeviceCommand,
    pub led_device_type: String,
    pub params: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingCommand {
    Stop,
    Start,
    Update,
}

#[derive(Debug, Deserialize, Validate)]
pub struct Logging {
    pub subcommand: LoggingCommand,
    pub oneshot: Option<bool>,
    pub interval: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingType {
    MulticolorMean,
    UnicolorMean,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Processing {
    pub mapping_type: MappingType,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ServerInfoRequest {
    pub subscribe: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SourceSelect {
    #[validate(range(min = 0, max = 255))]
    pub priority: i32,
    pub auto: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub enum VideoMode {
    #[serde(rename = "2D")]
    Mode2D,
    #[serde(rename = "3DSBS")]
    Mode3DSBS,
    #[serde(rename = "3DTAB")]
    Mode3DTAB,
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct VideoModeRequest {
    pub video_mode: VideoMode,
}

/// Incoming Hyperion JSON command
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase", tag = "command")]
pub enum HyperionCommand {
    Adjustment(Adjustment),
    Authorize(Authorize),
    Clear(Clear),
    ClearAll,
    Color(Color),
    ComponentState(ComponentState),
    Config(Config),
    #[serde(rename = "create-effect")]
    EffectCreate(EffectCreate),
    #[serde(rename = "delete-effect")]
    EffectDelete(EffectDelete),
    Effect(Effect),
    Image(Image),
    Instance(Instance),
    LedColors(LedColors),
    LedDevice(LedDevice),
    Logging(Logging),
    Processing(Processing),
    ServerInfo(ServerInfoRequest),
    SourceSelect(SourceSelect),
    SysInfo,
    VideoMode(VideoModeRequest),
}

/// Incoming Hyperion JSON message
#[derive(Debug, Deserialize)]
pub struct HyperionMessage {
    /// Request identifier
    pub tan: Option<i32>,
    #[serde(flatten)]
    pub command: HyperionCommand,
}

impl Validate for HyperionMessage {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match &self.command {
            HyperionCommand::Adjustment(adjustment) => adjustment.validate(),
            HyperionCommand::Authorize(authorize) => authorize.validate(),
            HyperionCommand::Clear(clear) => clear.validate(),
            HyperionCommand::ClearAll => Ok(()),
            HyperionCommand::Color(color) => color.validate(),
            HyperionCommand::ComponentState(component_state) => component_state.validate(),
            HyperionCommand::Config(config) => config.validate(),
            HyperionCommand::EffectCreate(effect_create) => effect_create.validate(),
            HyperionCommand::EffectDelete(effect_delete) => effect_delete.validate(),
            HyperionCommand::Effect(effect) => effect.validate(),
            HyperionCommand::Image(image) => image.validate(),
            HyperionCommand::Instance(instance) => instance.validate(),
            HyperionCommand::LedColors(led_colors) => led_colors.validate(),
            HyperionCommand::LedDevice(led_device) => led_device.validate(),
            HyperionCommand::Logging(logging) => logging.validate(),
            HyperionCommand::Processing(processing) => processing.validate(),
            HyperionCommand::ServerInfo(server_info) => server_info.validate(),
            HyperionCommand::SourceSelect(source_select) => source_select.validate(),
            HyperionCommand::SysInfo => Ok(()),
            HyperionCommand::VideoMode(video_mode) => video_mode.validate(),
        }
    }
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
    priorities: Vec<PriorityInfo>,
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

#[derive(Debug, Serialize)]
pub struct PriorityInfo {
    priority: i32,
    duration: i32,
    r#type: &'static str,
}

// TODO: Better From impl for PriorityInfo
impl From<crate::global::InputMessage> for PriorityInfo {
    fn from(msg: crate::global::InputMessage) -> Self {
        use crate::global::{InputMessageData, Message};
        match msg.data() {
            InputMessageData::SolidColor {
                priority, duration, ..
            } => Self {
                priority: *priority,
                duration: duration.map(|d| d.num_milliseconds() as i32).unwrap_or(0),
                r#type: "color",
            },
            InputMessageData::Image {
                priority, duration, ..
            } => Self {
                priority: *priority,
                duration: duration.map(|d| d.num_milliseconds() as i32).unwrap_or(0),
                r#type: "color",
            },
            InputMessageData::Clear { .. }
            | InputMessageData::ClearAll { .. }
            | InputMessageData::PrioritiesRequest { .. } => {
                panic!("cannot create PriorityInfo for InputMessage")
            }
        }
    }
}

/// Hyperion JSON response
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum HyperionResponse {
    /// Success response
    SuccessResponse {
        /// Success value (should be true)
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        tan: Option<i32>,
    },
    /// Error response
    ErrorResponse {
        /// Success value (should be false)
        success: bool,
        /// Error message
        error: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tan: Option<i32>,
    },
    /// Server information response
    ServerInfoResponse {
        /// Success value (should be true)
        success: bool,
        /// Server information
        // Box because of large size difference
        info: Box<ServerInfo>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tan: Option<i32>,
    },
}

impl HyperionResponse {
    /// Return a success response
    pub fn success(tan: Option<i32>) -> Self {
        HyperionResponse::SuccessResponse { success: true, tan }
    }

    /// Return an error response
    pub fn error(tan: Option<i32>, error: &impl std::fmt::Display) -> Self {
        HyperionResponse::ErrorResponse {
            success: false,
            error: error.to_string(),
            tan,
        }
    }

    /// Return a server information response
    pub fn server_info(
        tan: Option<i32>,
        effects: Vec<EffectDefinition>,
        priorities: Vec<PriorityInfo>,
    ) -> Self {
        HyperionResponse::ServerInfoResponse {
            success: true,
            info: Box::new(ServerInfo {
                hostname: hostname::get()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "<unknown hostname>".to_owned()),
                effects,
                hyperion_build: BuildInfo {
                    version: git_version::git_version!(
                        prefix = "hyperion.rs-",
                        args = ["--always", "--tags"]
                    )
                    .to_owned(),
                    time: "".to_owned(),
                },

                priorities,
                correction: json!([]),
                temperature: json!([]),
                adjustment: json!([]),
                active_effects: json!([]),
                active_led_color: json!([]),
            }),
            tan,
        }
    }
}
