use std::collections::BTreeMap;
use std::convert::TryFrom;

use ambassador::{delegatable_trait, Delegate};
use palette::rgb::Rgb;
use serde_derive::{Deserialize, Serialize};
use strum_macros::{EnumDiscriminants, EnumString, IntoStaticStr};
use thiserror::Error;
use validator::Validate;

use crate::db::models as db_models;

pub type Color = Rgb<palette::encoding::srgb::Srgb, u8>;
pub type Color16 = Rgb<palette::encoding::srgb::Srgb, u16>;

pub trait ServerConfig {
    fn port(&self) -> u16;
}

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("error parsing date: {0}")]
    Chrono(#[from] chrono::ParseError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlackBorderDetectorMode {
    Default,
    Classic,
    Osd,
    Letterbox,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BlackBorderDetector {
    pub enable: bool,
    #[validate(range(min = 0, max = 100))]
    pub threshold: u32,
    pub unknown_frame_cnt: u32,
    pub border_frame_cnt: u32,
    pub max_inconsistent_cnt: u32,
    pub blur_remove_cnt: u32,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct BoblightServer {
    pub enable: bool,
    #[validate(range(min = 1024))]
    pub port: u16,
    #[validate(range(min = 100, max = 254))]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ColorAdjustment {
    pub image_to_led_mapping_type: ImageToLedMappingType,
    #[validate]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
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
    #[validate(range(min = 0, max = 100))]
    pub backlight_threshold: u32,
    pub backlight_colored: bool,
    #[validate(range(min = 0, max = 100))]
    pub brightness: u32,
    #[validate(range(min = 0, max = 100))]
    pub brightness_compensation: u32,
    #[validate(range(min = 0.1, max = 5.0))]
    pub gamma_red: f32,
    #[validate(range(min = 0.1, max = 5.0))]
    pub gamma_green: f32,
    #[validate(range(min = 0.1, max = 5.0))]
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

impl ColorOrder {
    pub fn reorder_from_rgb(&self, color: Color) -> Color {
        let (r, g, b) = color.into_components();

        Color::from_components(match self {
            ColorOrder::Rgb => (r, g, b),
            ColorOrder::Bgr => (b, g, r),
            ColorOrder::Rbg => (r, b, g),
            ColorOrder::Brg => (b, r, g),
            ColorOrder::Gbr => (g, b, r),
            ColorOrder::Grb => (g, r, b),
        })
    }
}

#[delegatable_trait]
pub trait DeviceConfig {
    fn hardware_led_count(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct Dummy {
    #[validate(range(min = 1))]
    pub hardware_led_count: u32,
}

impl DeviceConfig for Dummy {
    fn hardware_led_count(&self) -> usize {
        self.hardware_led_count as _
    }
}

impl Default for Dummy {
    fn default() -> Self {
        Self {
            hardware_led_count: 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Ws2812Spi {
    pub color_order: ColorOrder,
    #[validate(range(min = 1))]
    pub hardware_led_count: u32,
    pub invert: bool,
    pub latch_time: i32,
    pub output: String,
    pub rate: i32,
    pub rewrite_time: i32,
}

impl DeviceConfig for Ws2812Spi {
    fn hardware_led_count(&self) -> usize {
        self.hardware_led_count as _
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
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
    #[validate(range(min = 1))]
    pub hardware_led_count: u32,
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

impl DeviceConfig for PhilipsHue {
    fn hardware_led_count(&self) -> usize {
        self.hardware_led_count as _
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, IntoStaticStr, Delegate)]
#[serde(rename_all = "lowercase", tag = "type")]
#[delegate(DeviceConfig)]
pub enum Device {
    Dummy(Dummy),
    Ws2812Spi(Ws2812Spi),
    PhilipsHue(PhilipsHue),
}

impl Default for Device {
    fn default() -> Self {
        Self::Dummy(Dummy::default())
    }
}

impl Validate for Device {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            Device::Dummy(device) => device.validate(),
            Device::Ws2812Spi(device) => device.validate(),
            Device::PhilipsHue(device) => device.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Effects {
    #[validate(length(min = 1))]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct FlatbuffersServer {
    pub enable: bool,
    #[validate(range(min = 1024))]
    pub port: u16,
    #[validate(range(min = 1))]
    pub timeout: u32,
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

impl ServerConfig for FlatbuffersServer {
    fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct ForegroundEffect {
    pub color: Color,
    pub effect: String,
    pub enable: bool,
    #[serde(rename = "type")]
    pub ty: EffectType,
    #[validate(range(min = 100))]
    pub duration_ms: Option<i32>,
}

impl Default for ForegroundEffect {
    fn default() -> Self {
        Self {
            enable: true,
            ty: EffectType::Effect,
            color: Color::from_components((255, 0, 0)),
            effect: String::new(),
            duration_ms: Some(3000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Framegrabber {
    #[serde(rename = "type")]
    pub ty: FramegrabberType,
    #[validate(range(min = 10))]
    pub width: u32,
    #[validate(range(min = 10))]
    pub height: u32,
    #[serde(rename = "frequency_Hz")]
    #[validate(range(min = 1))]
    pub frequency_hz: u32,
    pub crop_left: u32,
    pub crop_right: u32,
    pub crop_top: u32,
    pub crop_bottom: u32,
    #[validate(range(min = 1, max = 30))]
    pub pixel_decimation: u32,
    #[serde(default)]
    pub display: u32,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct General {
    #[validate(length(min = 4, max = 20))]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct GrabberV4L2 {
    pub device: String,
    pub input: i32,
    pub standard: V4L2Standard,
    pub width: u32,
    pub height: u32,
    #[validate(range(min = 1))]
    pub fps: u32,
    #[validate(range(min = 1, max = 30))]
    pub size_decimation: u32,
    pub crop_left: u32,
    pub crop_right: u32,
    pub crop_top: u32,
    pub crop_bottom: u32,
    pub cec_detection: bool,
    pub signal_detection: bool,
    #[validate(range(min = 0, max = 100))]
    pub red_signal_threshold: u32,
    #[validate(range(min = 0, max = 100))]
    pub green_signal_threshold: u32,
    #[validate(range(min = 0, max = 100))]
    pub blue_signal_threshold: u32,
    #[serde(rename = "sDVOffsetMin")]
    #[validate(range(min = 0., max = 1.))]
    pub sdv_offset_min: f32,
    #[serde(rename = "sDVOffsetMax")]
    #[validate(range(min = 0., max = 1.))]
    pub sdv_offset_max: f32,
    #[serde(rename = "sDHOffsetMin")]
    #[validate(range(min = 0., max = 1.))]
    pub sdh_offset_min: f32,
    #[serde(rename = "sDHOffsetMax")]
    #[validate(range(min = 0., max = 1.))]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct InstanceCapture {
    pub system_enable: bool,
    #[validate(range(min = 100, max = 253))]
    pub system_priority: i32,
    pub v4l_enable: bool,
    #[validate(range(min = 100, max = 253))]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Validate)]
pub struct JsonServer {
    #[validate(range(min = 1024))]
    pub port: u16,
}

impl Default for JsonServer {
    fn default() -> Self {
        Self { port: 19444 }
    }
}

impl ServerConfig for JsonServer {
    fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct ClassicLedConfig {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
    pub glength: u32,
    pub gpos: u32,
    pub position: i32,
    pub reverse: bool,
    #[validate(range(min = 1, max = 100))]
    pub hdepth: u32,
    #[validate(range(min = 1, max = 100))]
    pub vdepth: u32,
    pub overlap: u32,
    #[validate(range(max = 50))]
    pub edgegap: u32,
    #[validate(range(max = 100))]
    pub ptlh: u32,
    #[validate(range(max = 100))]
    pub ptlv: u32,
    #[validate(range(max = 100))]
    pub ptrh: u32,
    #[validate(range(max = 100))]
    pub ptrv: u32,
    #[validate(range(max = 100))]
    pub pblh: u32,
    #[validate(range(max = 100))]
    pub pblv: u32,
    #[validate(range(max = 100))]
    pub pbrh: u32,
    #[validate(range(max = 100))]
    pub pbrv: u32,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct MatrixLedConfig {
    #[validate(range(max = 50))]
    pub ledshoriz: u32,
    #[validate(range(max = 50))]
    pub ledsvert: u32,
    pub cabling: MatrixCabling,
    pub start: MatrixStart,
}

impl Default for MatrixLedConfig {
    fn default() -> Self {
        // TODO: Check those default values?
        Self {
            ledshoriz: 0,
            ledsvert: 0,
            cabling: MatrixCabling::Snake,
            start: MatrixStart::TopLeft,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct LedConfig {
    #[validate]
    pub classic: ClassicLedConfig,
    #[validate]
    pub matrix: MatrixLedConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[validate(schema(function = "validate_scan_range", message = "invalid range"))]
pub struct Led {
    #[validate(range(min = 0., max = 1.))]
    pub hmin: f32,
    #[validate(range(min = 0., max = 1.))]
    pub hmax: f32,
    #[validate(range(min = 0., max = 1.))]
    pub vmin: f32,
    #[validate(range(min = 0., max = 1.))]
    pub vmax: f32,
    pub color_order: Option<ColorOrder>,
    pub name: Option<String>,
}

/// Validate the bounds of a scan range
fn validate_scan_range(led: &Led) -> Result<(), validator::ValidationError> {
    if led.hmin > led.hmax {
        return Err(validator::ValidationError::new("invalid_range"));
    }

    if led.vmin > led.vmax {
        return Err(validator::ValidationError::new("invalid_range"));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Validate)]
pub struct Leds {
    #[validate]
    pub leds: Vec<Led>,
}

impl Default for Leds {
    fn default() -> Self {
        Self {
            leds: vec![Led {
                hmin: 0.,
                hmax: 1.,
                vmin: 0.,
                vmax: 1.,
                color_order: None,
                name: None,
            }],
        }
    }
}

impl serde::Serialize for Leds {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.leds.len()))?;
        for led in &self.leds {
            seq.serialize_element(led)?;
        }
        seq.end()
    }
}

impl<'de> serde::Deserialize<'de> for Leds {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Leds {
            leds: <Vec<Led> as serde::Deserialize>::deserialize(deserializer)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggerLevel {
    Silent,
    Warn,
    Verbose,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ProtoServer {
    pub enable: bool,
    #[validate(range(min = 1024))]
    pub port: u16,
    #[validate(range(min = 1))]
    pub timeout: u32,
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

impl ServerConfig for ProtoServer {
    fn port(&self) -> u16 {
        self.port
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmoothingType {
    Linear,
    Decay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Smoothing {
    pub enable: bool,
    #[serde(rename = "type")]
    pub ty: SmoothingType,
    #[serde(rename = "time_ms")]
    #[validate(range(min = 25, max = 5000))]
    pub time_ms: u32,
    #[validate(range(min = 1., max = 2000.))]
    pub update_frequency: f32,
    #[validate(range(min = 1., max = 1000.))]
    pub interpolation_rate: f32,
    #[validate(range(min = 1., max = 1000.))]
    pub output_rate: f32,
    #[validate(range(min = 1., max = 20.))]
    pub decay: f32,
    pub dithering: bool,
    #[validate(range(max = 2048))]
    pub update_delay: u32,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct WebConfig {
    #[serde(rename = "document_root")]
    pub document_root: String,
    #[validate(range(min = 80))]
    pub port: u16,
    #[validate(range(min = 80))]
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
    Leds(Leds),
    Logger(Logger),
    Network(Network),
    ProtoServer(ProtoServer),
    Smoothing(Smoothing),
    WebConfig(WebConfig),
}

impl Validate for SettingData {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            SettingData::BackgroundEffect(setting) => setting.validate(),
            SettingData::BlackBorderDetector(setting) => setting.validate(),
            SettingData::BoblightServer(setting) => setting.validate(),
            SettingData::ColorAdjustment(setting) => setting.validate(),
            SettingData::Device(setting) => setting.validate(),
            SettingData::Effects(setting) => setting.validate(),
            SettingData::FlatbuffersServer(setting) => setting.validate(),
            SettingData::ForegroundEffect(setting) => setting.validate(),
            SettingData::Forwarder(setting) => setting.validate(),
            SettingData::Framegrabber(setting) => setting.validate(),
            SettingData::General(setting) => setting.validate(),
            SettingData::GrabberV4L2(setting) => setting.validate(),
            SettingData::InstanceCapture(setting) => setting.validate(),
            SettingData::JsonServer(setting) => setting.validate(),
            SettingData::LedConfig(setting) => setting.validate(),
            SettingData::Leds(setting) => setting.validate(),
            SettingData::Logger(setting) => setting.validate(),
            SettingData::Network(setting) => setting.validate(),
            SettingData::ProtoServer(setting) => setting.validate(),
            SettingData::Smoothing(setting) => setting.validate(),
            SettingData::WebConfig(setting) => setting.validate(),
        }
    }
}

#[derive(Debug, Error)]
pub enum SettingError {
    #[error("error processing JSON: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("error parsing date: {0}")]
    Chrono(#[from] chrono::ParseError),
    #[error("validation error: {0}")]
    Validation(#[from] validator::ValidationErrors),
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

        config.validate()?;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct GlobalConfig {
    pub flatbuffers_server: FlatbuffersServer,
    pub forwarder: Forwarder,
    pub framegrabber: Framegrabber,
    pub general: General,
    pub grabber_v4l2: GrabberV4L2,
    pub json_server: JsonServer,
    pub logger: Logger,
    pub network: Network,
    pub proto_server: ProtoServer,
    pub web_config: WebConfig,
}

impl From<GlobalConfigCreator> for GlobalConfig {
    fn from(creator: GlobalConfigCreator) -> Self {
        Self {
            flatbuffers_server: creator.flatbuffers_server.unwrap_or_default(),
            forwarder: creator.forwarder.unwrap_or_default(),
            framegrabber: creator.framegrabber.unwrap_or_default(),
            general: creator.general.unwrap_or_default(),
            grabber_v4l2: creator.grabber_v4l2.unwrap_or_default(),
            json_server: creator.json_server.unwrap_or_default(),
            logger: creator.logger.unwrap_or_default(),
            network: creator.network.unwrap_or_default(),
            proto_server: creator.proto_server.unwrap_or_default(),
            web_config: creator.web_config.unwrap_or_default(),
        }
    }
}

#[derive(Default)]
struct GlobalConfigCreator {
    flatbuffers_server: Option<FlatbuffersServer>,
    forwarder: Option<Forwarder>,
    framegrabber: Option<Framegrabber>,
    general: Option<General>,
    grabber_v4l2: Option<GrabberV4L2>,
    json_server: Option<JsonServer>,
    logger: Option<Logger>,
    network: Option<Network>,
    proto_server: Option<ProtoServer>,
    web_config: Option<WebConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
pub struct InstanceConfig {
    #[validate]
    pub instance: Instance,
    #[validate]
    pub background_effect: BackgroundEffect,
    #[validate]
    pub black_border_detector: BlackBorderDetector,
    #[validate]
    pub boblight_server: BoblightServer,
    #[validate]
    pub color: ColorAdjustment,
    #[validate]
    pub device: Device,
    #[validate]
    pub effects: Effects,
    #[validate]
    pub foreground_effect: ForegroundEffect,
    #[validate]
    pub instance_capture: InstanceCapture,
    #[validate]
    pub led_config: LedConfig,
    #[validate]
    pub leds: Leds,
    #[validate]
    pub smoothing: Smoothing,
}

impl InstanceConfig {
    pub fn new_dummy(id: i32) -> Self {
        Self {
            instance: Instance {
                id,
                friendly_name: "Dummy device".to_owned(),
                enabled: true,
                last_use: chrono::Utc::now(),
            },
            background_effect: Default::default(),
            black_border_detector: Default::default(),
            boblight_server: Default::default(),
            color: Default::default(),
            device: Default::default(),
            effects: Default::default(),
            foreground_effect: Default::default(),
            instance_capture: Default::default(),
            led_config: Default::default(),
            leds: Default::default(),
            smoothing: Default::default(),
        }
    }
}

impl From<InstanceConfigCreator> for InstanceConfig {
    fn from(creator: InstanceConfigCreator) -> Self {
        Self {
            instance: creator.instance,
            background_effect: creator.background_effect.unwrap_or_default(),
            black_border_detector: creator.black_border_detector.unwrap_or_default(),
            boblight_server: creator.boblight_server.unwrap_or_default(),
            color: creator.color.unwrap_or_default(),
            device: creator.device.unwrap_or_default(),
            effects: creator.effects.unwrap_or_default(),
            foreground_effect: creator.foreground_effect.unwrap_or_default(),
            instance_capture: creator.instance_capture.unwrap_or_default(),
            led_config: creator.led_config.unwrap_or_default(),
            leds: creator.leds.unwrap_or_default(),
            smoothing: creator.smoothing.unwrap_or_default(),
        }
    }
}

struct InstanceConfigCreator {
    instance: Instance,
    background_effect: Option<BackgroundEffect>,
    black_border_detector: Option<BlackBorderDetector>,
    boblight_server: Option<BoblightServer>,
    color: Option<ColorAdjustment>,
    device: Option<Device>,
    effects: Option<Effects>,
    foreground_effect: Option<ForegroundEffect>,
    instance_capture: Option<InstanceCapture>,
    led_config: Option<LedConfig>,
    leds: Option<Leds>,
    smoothing: Option<Smoothing>,
}

impl InstanceConfigCreator {
    fn new(instance: Instance) -> Self {
        Self {
            instance,
            background_effect: None,
            black_border_detector: None,
            boblight_server: None,
            color: None,
            device: None,
            effects: None,
            foreground_effect: None,
            instance_capture: None,
            led_config: None,
            leds: None,
            smoothing: None,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("error querying the database: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("error loading instance: {0}")]
    Instance(#[from] InstanceError),
    #[error("error loading setting: {0}")]
    Setting(#[from] SettingError),
    #[error("error loading meta: {0}")]
    Meta(#[from] MetaError),
    #[error("error loading user: {0}")]
    User(#[from] UserError),
    // TODO: Say which setting?
    #[error("missing hyperion_inst field on instance setting")]
    MissingHyperionInst,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub instances: BTreeMap<i32, InstanceConfig>,
    pub global: GlobalConfig,
    meta: Vec<Meta>,
    users: Vec<User>,
}

impl Config {
    pub async fn load(db: &mut crate::db::Db) -> Result<Self, ConfigError> {
        let mut instances = BTreeMap::new();
        let mut global = GlobalConfigCreator::default();

        for instance in sqlx::query_as::<_, db_models::DbInstance>("SELECT * FROM instances")
            .fetch_all(&mut **db)
            .await?
            .into_iter()
            .map(Instance::try_from)
        {
            let instance = instance?;
            instances.insert(instance.id, InstanceConfigCreator::new(instance));
        }

        for setting in sqlx::query_as::<_, db_models::DbSetting>("SELECT * FROM settings")
            .fetch_all(&mut **db)
            .await?
            .into_iter()
            .map(Setting::try_from)
        {
            let setting = setting?;
            match setting.config {
                SettingData::BackgroundEffect(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .background_effect = Some(config)
                }
                SettingData::BlackBorderDetector(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .black_border_detector = Some(config)
                }
                SettingData::BoblightServer(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .boblight_server = Some(config)
                }
                SettingData::ColorAdjustment(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .color = Some(config)
                }
                SettingData::Device(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .device = Some(config)
                }
                SettingData::Effects(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .effects = Some(config)
                }
                SettingData::ForegroundEffect(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .foreground_effect = Some(config)
                }
                SettingData::InstanceCapture(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .instance_capture = Some(config)
                }
                SettingData::LedConfig(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .led_config = Some(config)
                }
                SettingData::Leds(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .leds = Some(config)
                }
                SettingData::Smoothing(config) => {
                    instances
                        .get_mut(
                            &setting
                                .hyperion_inst
                                .ok_or(ConfigError::MissingHyperionInst)?,
                        )
                        .unwrap()
                        .smoothing = Some(config)
                }

                SettingData::FlatbuffersServer(config) => {
                    global.flatbuffers_server = Some(config);
                }
                SettingData::Forwarder(config) => {
                    global.forwarder = Some(config);
                }
                SettingData::Framegrabber(config) => {
                    global.framegrabber = Some(config);
                }
                SettingData::General(config) => {
                    global.general = Some(config);
                }
                SettingData::GrabberV4L2(config) => {
                    global.grabber_v4l2 = Some(config);
                }
                SettingData::JsonServer(config) => {
                    global.json_server = Some(config);
                }
                SettingData::Logger(config) => {
                    global.logger = Some(config);
                }
                SettingData::Network(config) => {
                    global.network = Some(config);
                }
                SettingData::ProtoServer(config) => {
                    global.proto_server = Some(config);
                }
                SettingData::WebConfig(config) => {
                    global.web_config = Some(config);
                }
            }
        }

        let meta: Result<Vec<_>, _> = sqlx::query_as::<_, db_models::DbMeta>("SELECT * FROM meta")
            .fetch_all(&mut **db)
            .await?
            .into_iter()
            .map(Meta::try_from)
            .collect();
        let meta = meta?;

        let users: Result<Vec<_>, _> = sqlx::query_as::<_, db_models::DbUser>("SELECT * FROM auth")
            .fetch_all(&mut **db)
            .await?
            .into_iter()
            .map(User::try_from)
            .collect();
        let users = users?;

        let instances: BTreeMap<i32, InstanceConfig> =
            instances.into_iter().map(|(k, v)| (k, v.into())).collect();

        let global: GlobalConfig = global.into();

        debug!(
            "`{}`: loaded {} instance(s), {} meta, {} user(s)",
            global.general.name,
            instances.len(),
            meta.len(),
            users.len()
        );

        Ok(Self {
            instances,
            global,
            meta,
            users,
        })
    }
}
