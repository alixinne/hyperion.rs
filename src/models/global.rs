use serde_derive::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use validator::Validate;

use super::ServerConfig;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
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
#[serde(default, deny_unknown_fields)]
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
#[serde(rename_all = "lowercase", deny_unknown_fields)]
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

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Framegrabber {
    pub enable: bool,
    #[serde(rename = "type")]
    pub ty: FramegrabberType,
    #[serde(rename = "available_devices")]
    pub available_devices: String,
    pub device: String,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "device_inputs")]
    pub device_inputs: usize,
    #[validate(range(min = 10))]
    pub width: u32,
    #[validate(range(min = 10))]
    pub height: u32,
    pub fps: u32,
    #[serde_as(as = "DisplayFromStr")]
    pub framerates: u32,
    pub input: u32,
    #[serde_as(as = "DisplayFromStr")]
    pub resolutions: usize,
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
            enable: false,
            ty: Default::default(),
            available_devices: "".to_owned(),
            device: "".to_owned(),
            device_inputs: 0,
            width: 80,
            height: 45,
            fps: 25,
            framerates: 25,
            input: 0,
            resolutions: 0,
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
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct General {
    #[validate(length(min = 4, max = 20))]
    pub name: String,
    pub watched_version_branch: WatchedVersionBranch,
    pub show_opt_help: bool,
    pub previous_version: Option<String>,
    pub config_version: String,
}

impl Default for General {
    fn default() -> Self {
        Self {
            name: "My Hyperion Config".to_owned(),
            watched_version_branch: WatchedVersionBranch::Stable,
            show_opt_help: true,
            previous_version: None,
            config_version: "".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum FlipMode {
    #[serde(rename = "NO_CHANGE")]
    NoChange,
    Horizontal,
    Vertical,
    Both,
}

impl Default for FlipMode {
    fn default() -> Self {
        Self::NoChange
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE", deny_unknown_fields)]
pub enum PixelFormat {
    #[serde(rename = "NO_CHANGE")]
    NoChange,
    Yuyv,
    Uyvy,
    Bgr16,
    Bgr24,
    Rgb32,
    Bgr32,
    I420,
    Nv12,
    Mjpeg,
}

impl Default for PixelFormat {
    fn default() -> Self {
        Self::NoChange
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct GrabberV4L2 {
    pub enable: bool,
    pub device: String,
    pub input: i32,
    pub standard: V4L2Standard,
    pub width: u32,
    pub height: u32,
    pub encoding: PixelFormat,
    pub flip: FlipMode,
    #[validate(range(min = 1))]
    pub fps: u32,
    pub fps_software_decimation: u32,
    #[validate(range(min = 1, max = 30))]
    pub size_decimation: u32,
    pub crop_left: u32,
    pub crop_right: u32,
    pub crop_top: u32,
    pub crop_bottom: u32,
    pub cec_detection: bool,
    #[serde(rename = "hardware_brightness")]
    pub hardware_brightness: u32,
    #[serde(rename = "hardware_contrast")]
    pub hardware_contrast: u32,
    #[serde(rename = "hardware_hue")]
    pub hardware_hue: u32,
    #[serde(rename = "hardware_saturation")]
    pub hardware_saturation: u32,
    pub no_signal_counter_threshold: u32,
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
            enable: false,
            device: "auto".to_owned(),
            input: 0,
            standard: Default::default(),
            width: 0,
            height: 0,
            encoding: Default::default(),
            flip: Default::default(),
            fps: 15,
            fps_software_decimation: 0,
            size_decimation: 6,
            crop_left: 0,
            crop_right: 0,
            crop_top: 0,
            crop_bottom: 0,
            cec_detection: false,
            hardware_brightness: 0,
            hardware_contrast: 0,
            hardware_hue: 0,
            hardware_saturation: 0,
            no_signal_counter_threshold: 200,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default, deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub enum LoggerLevel {
    Silent,
    Warn,
    Verbose,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default, deny_unknown_fields)]
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
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Network {
    pub api_auth: bool,
    #[serde(default, rename = "internetAccessAPI")]
    pub internet_access_api: bool,
    #[serde(
        default,
        rename = "restrictedInternetAccessAPI",
        alias = "restirctedInternetAccessAPI"
    )]
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
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
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
    #[validate(range(min = 1))]
    pub max_sessions: u32,
}

impl WebConfig {
    pub const SYSTEM_DOCUMENT_ROOT: &'static str = "$ROOT/webconfig";
}

impl Default for WebConfig {
    fn default() -> Self {
        Self {
            document_root: String::new(),
            port: 8090,
            ssl_port: 8092,
            crt_path: String::new(),
            key_path: String::new(),
            key_pass_phrase: String::new(),
            max_sessions: 4,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct Hooks {
    /// Command to run when an instance is started. HYPERION_INSTANCE_ID environment variable
    /// will hold the instance id.
    pub instance_start: Vec<String>,
    /// Command to run when an instance is stopped. HYPERION_INSTANCE_ID environment variable
    /// will hold the instance id.
    pub instance_stop: Vec<String>,
    /// Command to run when an instance is activated. HYPERION_INSTANCE_ID environment variable
    /// will hold the instance id.
    pub instance_activate: Vec<String>,
    /// Command to run when an instance is deactivated. HYPERION_INSTANCE_ID environment variable
    /// will hold the instance id.
    pub instance_deactivate: Vec<String>,
    /// Command to run when hyperion.rs starts
    pub start: Vec<String>,
    /// Command to run when hyperion.rs stops
    pub stop: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Validate)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct GlobalConfig {
    pub flatbuffers_server: FlatbuffersServer,
    pub forwarder: Forwarder,
    pub framegrabber: Framegrabber,
    pub general: General,
    #[serde(rename = "grabberV4L2")]
    pub grabber_v4l2: GrabberV4L2,
    pub json_server: JsonServer,
    pub logger: Logger,
    pub network: Network,
    pub proto_server: ProtoServer,
    pub web_config: WebConfig,
    pub hooks: Hooks,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_framegrabber() {
        let json_data = r#"
        {
            "available_devices": "UVC Camera (046d:0825)",
            "blueSignalThreshold": 0,
            "cecDetection": false,
            "cropBottom": 0,
            "cropLeft": 0,
            "cropRight": 0,
            "cropTop": 0,
            "device": "/dev/video1",
            "device_inputs": "0",
            "enable": true,
            "encoding": "YUYV",
            "flip": "NO_CHANGE",
            "fps": 15,
            "fpsSoftwareDecimation": 0,
            "framerates": "15",
            "greenSignalThreshold": 100,
            "hardware_brightness": 128,
            "hardware_contrast": 32,
            "hardware_hue": 0,
            "hardware_saturation": 32,
            "height": 480,
            "input": 0,
            "noSignalCounterThreshold": 200,
            "redSignalThreshold": 0,
            "resolutions": "8",
            "sDHOffsetMax": 0.46,
            "sDHOffsetMin": 0.4,
            "sDVOffsetMax": 0.9,
            "sDVOffsetMin": 0.1,
            "signalDetection": false,
            "sizeDecimation": 8,
            "standard": "NONE",
            "width": 640
        }"#;

        let deserialized = serde_json::de::from_str::<Framegrabber>(json_data)
            .expect("Failed to deserialize json data");

        assert_eq!(deserialized.available_devices, "UVC Camera (046d:0825)");
        assert_eq!(deserialized.device_inputs, 0);
        assert_eq!(deserialized.framerates, 15);
        assert_eq!(deserialized.resolutions, 8);
    }

    #[test]
    fn serialize_framegrabber() {
        let framegrabber = {
            let mut f = Framegrabber::default();
            f.available_devices = "Some camera (0123:321d)".to_string();
            f.device_inputs = 12;
            f.framerates = 30;
            f.resolutions = 10;
            f
        };
        let serialized = serde_json::ser::to_string(&framegrabber)
            .expect("Failed to serialize Framegrabber struct");

        assert!(serialized.contains(r#""available_devices":"Some camera (0123:321d)""#));
        assert!(serialized.contains(r#""device_inputs":"12""#));
        assert!(serialized.contains(r#""framerates":"30""#));
        assert!(serialized.contains(r#""resolutions":"10""#));
    }
}
