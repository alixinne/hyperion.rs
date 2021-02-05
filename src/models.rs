use std::convert::TryFrom;

use palette::rgb::Rgb;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{EnumDiscriminants, EnumString};
use thiserror::Error;

use crate::db::models as db_models;

pub type Color = Rgb<palette::encoding::srgb::Srgb, u8>;

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("error parsing date: {0}")]
    Chrono(#[from] chrono::ParseError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    pub id: i32,
    pub friendly_name: String,
    pub enabled: bool,
    pub last_use: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<db_models::DbInstance> for Instance {
    type Error = InstanceError;

    fn try_from(db: db_models::DbInstance) -> Result<Self, Self::Error> {
        Ok(Self {
            id: db.instance,
            friendly_name: db.friendly_name,
            enabled: db.enabled != 0,
            last_use: chrono::DateTime::parse_from_rfc3339(&db.last_use)?
                .with_timezone(&chrono::Utc),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setting {
    pub hyperion_inst: Option<i32>,
    pub config: SettingData,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffectType {
    Color,
    Effect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundEffect {
    pub color: Color,
    pub effect: String,
    pub enable: bool,
    #[serde(rename = "type")]
    pub ty: EffectType,
}

impl Default for BackgroundEffect {
    fn default() -> Self {
        Self {
            enable: true,
            ty: EffectType::Effect,
            color: Color::from_components((255, 138, 0)),
            effect: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlackBorderDetectorMode {
    Default,
    Classic,
    Osd,
    Letterbox,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlackBorderDetector {
    pub enable: bool,
    pub threshold: i32,
    pub unknown_frame_cnt: i32,
    pub border_frame_cnt: i32,
    pub max_inconsistent_cnt: i32,
    pub blur_remove_cnt: i32,
    pub mode: BlackBorderDetectorMode,
}

impl Default for BlackBorderDetector {
    fn default() -> Self {
        Self {
            enable: true,
            threshold: 5,
            unknown_frame_cnt: 600,
            border_frame_cnt: 50,
            max_inconsistent_cnt: 10,
            blur_remove_cnt: 1,
            mode: BlackBorderDetectorMode::Default,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoblightServer {
    pub enable: bool,
    pub port: u16,
    pub priority: i32,
}

impl Default for BoblightServer {
    fn default() -> Self {
        Self {
            enable: false,
            port: 19333,
            priority: 128,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageToLedMappingType {
    MulticolorMean,
    UnicolorMean,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorAdjustment {
    pub image_to_led_mapping_type: ImageToLedMappingType,
    pub channel_adjustment: Vec<ChannelAdjustment>,
}

impl Default for ColorAdjustment {
    fn default() -> Self {
        Self {
            image_to_led_mapping_type: ImageToLedMappingType::MulticolorMean,
            channel_adjustment: vec![ChannelAdjustment::default()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelAdjustment {
    pub id: String,
    pub leds: String,
    pub white: Color,
    pub red: Color,
    pub green: Color,
    pub blue: Color,
    pub cyan: Color,
    pub magenta: Color,
    pub yellow: Color,
    pub backlight_threshold: i32,
    pub backlight_colored: bool,
    pub brightness: i32,
    pub brightness_compensation: i32,
    pub gamma_red: f32,
    pub gamma_green: f32,
    pub gamma_blue: f32,
}

impl Default for ChannelAdjustment {
    fn default() -> Self {
        Self {
            id: "A userdefined name".to_owned(),
            leds: "*".to_owned(),
            white: Color::from_components((255, 255, 255)),
            red: Color::from_components((255, 0, 0)),
            green: Color::from_components((0, 255, 0)),
            blue: Color::from_components((0, 0, 255)),
            cyan: Color::from_components((0, 255, 255)),
            magenta: Color::from_components((255, 0, 255)),
            yellow: Color::from_components((255, 255, 0)),
            backlight_threshold: 0,
            backlight_colored: false,
            brightness: 100,
            brightness_compensation: 0,
            gamma_red: 1.5,
            gamma_green: 1.5,
            gamma_blue: 1.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorOrder {
    Rgb,
    Bgr,
    Rbg,
    Brg,
    Gbr,
    Grb,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ws2812Spi {
    color_order: ColorOrder,
    hardware_led_count: i32,
    invert: bool,
    latch_time: i32,
    output: String,
    rate: i32,
    rewrite_time: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhilipsHue {
    pub black_lights_timeout: i32,
    pub brightness_factor: f32,
    pub brightness_max: f32,
    pub brightness_min: f32,
    pub brightness_threshold: f32,
    #[serde(rename = "clientkey")]
    pub client_key: String,
    pub color_order: ColorOrder,
    pub debug_level: String,
    pub debug_streamer: bool,
    pub group_id: i32,
    pub hardware_led_count: i32,
    pub light_ids: Vec<String>,
    pub output: String,
    pub restore_original_state: bool,
    #[serde(rename = "sslHSTimeoutMax")]
    pub ssl_hs_timeout_max: i32,
    #[serde(rename = "sslHSTimeoutMin")]
    pub ssl_hs_timeout_min: i32,
    pub ssl_read_timeout: i32,
    pub switch_off_on_black: bool,
    #[serde(rename = "transitiontime")]
    pub transition_time: f32,
    #[serde(rename = "useEntertainmentAPI")]
    pub use_entertainment_api: bool,
    pub username: String,
    pub verbose: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum Device {
    Ws2812Spi(Ws2812Spi),
    PhilipsHue(PhilipsHue),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Effects {
    pub paths: Vec<String>,
    pub disable: Vec<String>,
}

impl Default for Effects {
    fn default() -> Self {
        Self {
            paths: vec!["$ROOT/custom-effects".to_owned()],
            disable: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlatbuffersServer {
    pub enable: bool,
    pub port: u16,
    pub timeout: i32,
}

impl Default for FlatbuffersServer {
    fn default() -> Self {
        Self {
            enable: true,
            port: 19400,
            timeout: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForegroundEffect {
    pub color: Color,
    pub effect: String,
    pub enable: bool,
    #[serde(rename = "type")]
    pub ty: EffectType,
    pub duration_ms: i32,
}

impl Default for ForegroundEffect {
    fn default() -> Self {
        Self {
            enable: true,
            ty: EffectType::Effect,
            color: Color::from_components((255, 0, 0)),
            effect: String::new(),
            duration_ms: 3000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Forwarder {
    pub enable: bool,
    pub json: Vec<String>,
    pub flat: Vec<String>,
}

impl Default for Forwarder {
    fn default() -> Self {
        Self {
            enable: false,
            json: vec!["127.0.0.1:19446".to_owned()],
            flat: vec!["127.0.0.1:19401".to_owned()],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FramegrabberType {
    Auto,
    AMLogic,
    DispmanX,
    DirectX9,
    Framebuffer,
    OSX,
    QT,
    X11,
    XCB,
}

impl Default for FramegrabberType {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Framegrabber {
    #[serde(rename = "type")]
    pub ty: FramegrabberType,
    pub width: i32,
    pub height: i32,
    #[serde(rename = "frequency_Hz")]
    pub frequency_hz: i32,
    pub crop_left: i32,
    pub crop_right: i32,
    pub crop_top: i32,
    pub crop_bottom: i32,
    pub pixel_decimation: i32,
    #[serde(default)]
    pub display: i32,
}

impl Default for Framegrabber {
    fn default() -> Self {
        Self {
            ty: Default::default(),
            width: 80,
            height: 45,
            frequency_hz: 10,
            crop_left: 0,
            crop_right: 0,
            crop_top: 0,
            crop_bottom: 0,
            pixel_decimation: 8,
            display: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WatchedVersionBranch {
    Stable,
    Beta,
    Alpha,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct General {
    pub name: String,
    pub watched_version_branch: WatchedVersionBranch,
    pub show_opt_help: bool,
}

impl Default for General {
    fn default() -> Self {
        Self {
            name: "My Hyperion Config".to_owned(),
            watched_version_branch: WatchedVersionBranch::Stable,
            show_opt_help: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum V4L2Standard {
    NoChange,
    Pal,
    Ntsc,
    Secam,
}

impl Default for V4L2Standard {
    fn default() -> Self {
        Self::NoChange
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrabberV4L2 {
    pub device: String,
    pub input: i32,
    pub standard: V4L2Standard,
    pub width: i32,
    pub height: i32,
    pub fps: i32,
    pub size_decimation: i32,
    pub crop_left: i32,
    pub crop_right: i32,
    pub crop_top: i32,
    pub crop_bottom: i32,
    pub cec_detection: bool,
    pub signal_detection: bool,
    pub red_signal_threshold: i32,
    pub green_signal_threshold: i32,
    pub blue_signal_threshold: i32,
    #[serde(rename = "sDVOffsetMin")]
    pub sdv_offset_min: f32,
    #[serde(rename = "sDVOffsetMax")]
    pub sdv_offset_max: f32,
    #[serde(rename = "sDHOffsetMin")]
    pub sdh_offset_min: f32,
    #[serde(rename = "sDHOffsetMax")]
    pub sdh_offset_max: f32,
}

impl Default for GrabberV4L2 {
    fn default() -> Self {
        Self {
            device: "auto".to_owned(),
            input: 0,
            standard: Default::default(),
            width: 0,
            height: 0,
            fps: 15,
            size_decimation: 6,
            crop_left: 0,
            crop_right: 0,
            crop_top: 0,
            crop_bottom: 0,
            cec_detection: false,
            signal_detection: false,
            red_signal_threshold: 5,
            green_signal_threshold: 5,
            blue_signal_threshold: 5,
            sdv_offset_min: 0.25,
            sdv_offset_max: 0.75,
            sdh_offset_min: 0.25,
            sdh_offset_max: 0.75,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceCapture {
    pub system_enable: bool,
    pub system_priority: i32,
    pub v4l_enable: bool,
    pub v4l_priority: i32,
}

impl Default for InstanceCapture {
    fn default() -> Self {
        Self {
            system_enable: true,
            system_priority: 250,
            v4l_enable: false,
            v4l_priority: 240,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct JsonServer {
    pub port: u16,
}

impl Default for JsonServer {
    fn default() -> Self {
        Self { port: 19444 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassicLedConfig {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
    pub glength: i32,
    pub gpos: i32,
    pub position: i32,
    pub reverse: bool,
    pub hdepth: i32,
    pub vdepth: i32,
    pub overlap: i32,
    pub edgegap: i32,
    pub ptlh: i32,
    pub ptlv: i32,
    pub ptrh: i32,
    pub ptrv: i32,
    pub pblh: i32,
    pub pblv: i32,
    pub pbrh: i32,
    pub pbrv: i32,
}

impl Default for ClassicLedConfig {
    fn default() -> Self {
        Self {
            top: 1,
            bottom: 0,
            left: 0,
            right: 0,
            glength: 0,
            gpos: 0,
            position: 0,
            reverse: false,
            hdepth: 8,
            vdepth: 5,
            overlap: 0,
            edgegap: 0,
            ptlh: 0,
            ptlv: 0,
            ptrh: 0,
            ptrv: 0,
            pblh: 0,
            pblv: 0,
            pbrh: 0,
            pbrv: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MatrixCabling {
    Snake,
    Parallel,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MatrixStart {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatrixLedConfig {
    pub ledshoriz: i32,
    pub ledsvert: i32,
    pub cabling: MatrixCabling,
    pub start: MatrixStart,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedConfig {
    pub classic: ClassicLedConfig,
    pub matrix: MatrixLedConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Led {
    pub hmin: f32,
    pub hmax: f32,
    pub vmin: f32,
    pub vmax: f32,
    pub color_order: Option<ColorOrder>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggerLevel {
    Silent,
    Warn,
    Verbose,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Logger {
    pub level: LoggerLevel,
}

impl Default for Logger {
    fn default() -> Self {
        Self {
            level: LoggerLevel::Warn,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    pub api_auth: bool,
    #[serde(default)]
    pub internet_access_api: bool,
    #[serde(default)]
    pub restricted_internet_access_api: bool,
    pub ip_whitelist: Vec<String>,
    pub local_api_auth: bool,
    pub local_admin_auth: bool,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            api_auth: true,
            internet_access_api: false,
            restricted_internet_access_api: false,
            ip_whitelist: vec![],
            local_api_auth: false,
            local_admin_auth: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtoServer {
    pub enable: bool,
    pub port: u16,
    pub timeout: i32,
}

impl Default for ProtoServer {
    fn default() -> Self {
        Self {
            enable: true,
            port: 19445,
            timeout: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmoothingType {
    Linear,
    Decay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Smoothing {
    pub enable: bool,
    #[serde(rename = "type")]
    pub ty: SmoothingType,
    #[serde(rename = "time_ms")]
    pub time_ms: i32,
    pub update_frequency: f32,
    pub interpolation_rate: f32,
    pub output_rate: f32,
    pub decay: f32,
    pub dithering: bool,
    pub update_delay: i32,
    pub continuous_output: bool,
}

impl Default for Smoothing {
    fn default() -> Self {
        Self {
            enable: true,
            ty: SmoothingType::Linear,
            time_ms: 200,
            update_frequency: 25.0,
            interpolation_rate: 1.0,
            output_rate: 1.0,
            decay: 1.0,
            dithering: true,
            update_delay: 0,
            continuous_output: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebConfig {
    #[serde(rename = "document_root")]
    pub document_root: String,
    pub port: u16,
    pub ssl_port: u16,
    pub crt_path: String,
    pub key_path: String,
    pub key_pass_phrase: String,
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            // TODO: Check document_root
            document_root: "$ROOT/webconfig".to_owned(),
            port: 8090,
            ssl_port: 8092,
            crt_path: String::new(),
            key_path: String::new(),
            key_pass_phrase: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, EnumDiscriminants, Serialize, Deserialize)]
#[strum_discriminants(name(SettingKind), derive(EnumString))]
pub enum SettingData {
    BackgroundEffect(BackgroundEffect),
    BlackBorderDetector(BlackBorderDetector),
    BoblightServer(BoblightServer),
    ColorAdjustment(ColorAdjustment),
    Device(Device),
    Effects(Effects),
    FlatbuffersServer(FlatbuffersServer),
    ForegroundEffect(ForegroundEffect),
    Forwarder(Forwarder),
    Framegrabber(Framegrabber),
    General(General),
    GrabberV4L2(GrabberV4L2),
    InstanceCapture(InstanceCapture),
    JsonServer(JsonServer),
    LedConfig(LedConfig),
    Leds(Vec<Led>),
    Logger(Logger),
    Network(Network),
    ProtoServer(ProtoServer),
    Smoothing(Smoothing),
    WebConfig(WebConfig),
}

#[derive(Debug, Error)]
pub enum SettingError {
    #[error("error processing JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("error parsing date: {0}")]
    Chrono(#[from] chrono::ParseError),
}

impl TryFrom<db_models::DbSetting> for Setting {
    type Error = SettingError;

    fn try_from(db: db_models::DbSetting) -> Result<Self, Self::Error> {
        let config = match db.ty.as_str() {
            "backgroundEffect" => SettingData::BackgroundEffect(serde_json::from_str(&db.config)?),
            "blackborderdetector" => {
                SettingData::BlackBorderDetector(serde_json::from_str(&db.config)?)
            }
            "boblightServer" => SettingData::BoblightServer(serde_json::from_str(&db.config)?),
            "color" => SettingData::ColorAdjustment(serde_json::from_str(&db.config)?),
            "device" => SettingData::Device(serde_json::from_str(&db.config)?),
            "effects" => SettingData::Effects(serde_json::from_str(&db.config)?),
            "flatbufServer" => SettingData::FlatbuffersServer(serde_json::from_str(&db.config)?),
            "foregroundEffect" => SettingData::ForegroundEffect(serde_json::from_str(&db.config)?),
            "forwarder" => SettingData::Forwarder(serde_json::from_str(&db.config)?),
            "framegrabber" => SettingData::Framegrabber(serde_json::from_str(&db.config)?),
            "general" => SettingData::General(serde_json::from_str(&db.config)?),
            "grabberV4L2" => SettingData::GrabberV4L2(serde_json::from_str(&db.config)?),
            "instCapture" => SettingData::InstanceCapture(serde_json::from_str(&db.config)?),
            "jsonServer" => SettingData::JsonServer(serde_json::from_str(&db.config)?),
            "ledConfig" => SettingData::LedConfig(serde_json::from_str(&db.config)?),
            "leds" => SettingData::Leds(serde_json::from_str(&db.config)?),
            "logger" => SettingData::Logger(serde_json::from_str(&db.config)?),
            "network" => SettingData::Network(serde_json::from_str(&db.config)?),
            "protoServer" => SettingData::ProtoServer(serde_json::from_str(&db.config)?),
            "smoothing" => SettingData::Smoothing(serde_json::from_str(&db.config)?),
            "webConfig" => SettingData::WebConfig(serde_json::from_str(&db.config)?),
            other => panic!("unsupported setting type: {}", other),
        };

        Ok(Self {
            hyperion_inst: db.hyperion_inst,
            config,
            updated_at: chrono::DateTime::parse_from_rfc3339(&db.updated_at)?
                .with_timezone(&chrono::Utc),
        })
    }
}

#[derive(Debug, Error)]
pub enum MetaError {
    #[error("error parsing date: {0}")]
    Chrono(#[from] chrono::ParseError),
    #[error("error parsing uuid: {0}")]
    Uuid(#[from] uuid::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    pub uuid: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<db_models::DbMeta> for Meta {
    type Error = MetaError;

    fn try_from(db: db_models::DbMeta) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: uuid::Uuid::parse_str(&db.uuid)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)?
                .with_timezone(&chrono::Utc),
        })
    }
}

#[derive(Debug, Error)]
pub enum UserError {
    #[error("error parsing date: {0}")]
    Chrono(#[from] chrono::ParseError),
    #[error("error parsing uuid: {0}")]
    Uuid(#[from] uuid::Error),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub password: Vec<u8>,
    pub token: Vec<u8>,
    pub salt: Vec<u8>,
    pub comment: Option<String>,
    pub id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_use: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<db_models::DbUser> for User {
    type Error = UserError;

    fn try_from(db: db_models::DbUser) -> Result<Self, Self::Error> {
        Ok(Self {
            name: db.user,
            password: db.password,
            token: db.token,
            salt: db.salt,
            comment: db.comment,
            id: db.id,
            created_at: chrono::DateTime::parse_from_rfc3339(&db.created_at)?
                .with_timezone(&chrono::Utc),
            last_use: chrono::DateTime::parse_from_rfc3339(&db.last_use)?
                .with_timezone(&chrono::Utc),
        })
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("error querying the database: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("error loading instance: {0}")]
    Instance(#[from] InstanceError),
    #[error("error loading setting: {0}")]
    Setting(#[from] SettingError),
    #[error("error loading meta: {0}")]
    Meta(#[from] MetaError),
    #[error("error loading user: {0}")]
    User(#[from] UserError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    instances: Vec<Instance>,
    settings: Vec<Setting>,
    meta: Vec<Meta>,
    users: Vec<User>,
}

impl Config {
    pub fn load(db: &crate::db::Db) -> Result<Self, ConfigError> {
        use crate::db::schema::auth::dsl::auth as db_auths;
        use crate::db::schema::instances::dsl::instances as db_instances;
        use crate::db::schema::meta::dsl::meta as db_meta;
        use crate::db::schema::settings::dsl::settings as db_settings;
        use diesel::prelude::*;

        let instances: Result<Vec<_>, _> = db_instances
            .load::<db_models::DbInstance>(&**db)?
            .into_iter()
            .map(Instance::try_from)
            .collect();
        let instances = instances?;

        let settings: Result<Vec<_>, _> = db_settings
            .load::<db_models::DbSetting>(&**db)?
            .into_iter()
            .map(Setting::try_from)
            .collect();
        let settings = settings?;

        let meta: Result<Vec<_>, _> = db_meta
            .load::<db_models::DbMeta>(&**db)?
            .into_iter()
            .map(Meta::try_from)
            .collect();
        let meta = meta?;

        let users: Result<Vec<_>, _> = db_auths
            .load::<db_models::DbUser>(&**db)?
            .into_iter()
            .map(User::try_from)
            .collect();
        let users = users?;

        debug!(
            "loaded {} instance(s), {} setting(s), {} meta, {} user(s)",
            instances.len(),
            settings.len(),
            meta.len(),
            users.len()
        );

        Ok(Self {
            instances,
            settings,
            meta,
            users,
        })
    }

    pub fn get(&self, instance_id: Option<i32>, ty: SettingKind) -> Option<&Setting> {
        // TODO: Not O(n)
        for setting in &self.settings {
            if setting.hyperion_inst == instance_id {
                let kind: SettingKind = (&setting.config).into();

                if kind == ty {
                    return Some(setting);
                }
            }
        }

        None
    }
}
