//! Definition of the ColorFormat type

use regex::Regex;
use validator::{Validate, ValidationErrors};

use crate::color::ColorPoint;

/// Default RGB LED order
fn default_rgb_order() -> String {
    "rgb".to_owned()
}

/// Default RGBW LED order
fn default_rgbw_order() -> String {
    "rgbw".to_owned()
}

/// Default RGBCW LED order
fn default_rgbcw_order() -> String {
    "rgbcw".to_owned()
}

/// Default RGB white temperature
fn default_rgb() -> ColorPoint {
    ColorPoint::srgb_white()
}

/// Default RGBW white temperature
fn default_rgbw_white() -> ColorPoint {
    ColorPoint::from_kelvin(6500.)
}

/// Default RGBCW cold white temperature
fn default_rgbcw_cold_white() -> ColorPoint {
    ColorPoint::from_kelvin(6500.)
}

/// Default RGBCW warm white temperature
fn default_rgbcw_warm_white() -> ColorPoint {
    ColorPoint::from_kelvin(2800.)
}

/// Default gamma value
fn default_gamma() -> f32 {
    2.2
}

/// RGB Gamma data
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct RgbGamma {
    /// Red channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub r: f32,
    /// Green channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub g: f32,
    /// Blue channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub b: f32,
}

impl Default for RgbGamma {
    fn default() -> Self {
        Self {
            r: default_gamma(),
            g: default_gamma(),
            b: default_gamma(),
        }
    }
}

/// RGBW Gamma data
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct RgbwGamma {
    /// Red channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub r: f32,
    /// Green channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub g: f32,
    /// Blue channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub b: f32,
    /// White channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub w: f32,
}

impl Default for RgbwGamma {
    fn default() -> Self {
        Self {
            r: default_gamma(),
            g: default_gamma(),
            b: default_gamma(),
            w: default_gamma(),
        }
    }
}

/// RGBCW Gamma data
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct RgbcwGamma {
    /// Red channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub r: f32,
    /// Green channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub g: f32,
    /// Blue channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub b: f32,
    /// Cold white channel gamma,
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub c: f32,
    /// Warm white channel gamma
    #[serde(default = "default_gamma")]
    #[validate(range(min = 0.0))]
    pub w: f32,
}

impl Default for RgbcwGamma {
    fn default() -> Self {
        Self {
            r: default_gamma(),
            g: default_gamma(),
            b: default_gamma(),
            c: default_gamma(),
            w: default_gamma(),
        }
    }
}

lazy_static! {
    static ref RGB_REGEX: Regex = Regex::new(r"^[rgb]*$").unwrap();
    static ref RGBW_REGEX: Regex = Regex::new(r"^[rgbw]*$").unwrap();
    static ref RGBCW_REGEX: Regex = Regex::new(r"^[rgbcw]*$").unwrap();
}

/// RGB format data
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct RgbFormat {
    /// LED order string
    #[serde(default = "default_rgb_order")]
    #[validate(regex = "RGB_REGEX")]
    pub order: String,
    /// RGB White point
    #[serde(default = "default_rgb")]
    pub rgb: ColorPoint,
    /// Gamma values
    #[serde(default)]
    pub gamma: RgbGamma,
}

impl Default for RgbFormat {
    fn default() -> Self {
        Self {
            order: default_rgb_order(),
            rgb: default_rgb(),
            gamma: Default::default(),
        }
    }
}

/// RGBW format data
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct RgbwFormat {
    /// LED order string
    #[serde(default = "default_rgbw_order")]
    #[validate(regex = "RGBW_REGEX")]
    pub order: String,
    /// RGB White temperature
    #[serde(default = "default_rgb")]
    pub rgb: ColorPoint,
    /// White temperature (Kelvin)
    #[serde(default = "default_rgbw_white")]
    pub white: ColorPoint,
    /// Gamma values
    #[serde(default)]
    pub gamma: RgbwGamma,
}

impl Default for RgbwFormat {
    fn default() -> Self {
        Self {
            order: default_rgb_order(),
            rgb: default_rgb(),
            white: default_rgbw_white(),
            gamma: Default::default(),
        }
    }
}

/// RGBCW format data
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct RgbcwFormat {
    /// LED order string
    #[serde(default = "default_rgbcw_order")]
    #[validate(regex = "RGBCW_REGEX")]
    pub order: String,
    /// RGB White temperature
    #[serde(default = "default_rgb")]
    pub rgb: ColorPoint,
    /// Cold white temperature (Kelvin)
    #[serde(default = "default_rgbcw_cold_white")]
    pub cold_white: ColorPoint,
    /// Warm white temperature (Kelvin)
    #[serde(default = "default_rgbcw_warm_white")]
    pub warm_white: ColorPoint,
    /// Gamma values
    #[serde(default)]
    pub gamma: RgbcwGamma,
}

impl Default for RgbcwFormat {
    fn default() -> Self {
        Self {
            order: default_rgb_order(),
            rgb: default_rgb(),
            cold_white: default_rgbcw_cold_white(),
            warm_white: default_rgbcw_warm_white(),
            gamma: Default::default(),
        }
    }
}

/// Color data format used by a device
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ColorFormat {
    /// RGB
    #[serde(rename = "rgb")]
    Rgb(RgbFormat),
    /// RGB + White
    #[serde(rename = "rgbw")]
    Rgbw(RgbwFormat),
    /// RGB + Cold white + Warm white
    #[serde(rename = "rgbcw")]
    Rgbcw(RgbcwFormat),
}

impl ColorFormat {
    /// Return the number of components in the output of a given format
    pub fn components(&self) -> usize {
        match self {
            ColorFormat::Rgb { .. } => 3,
            ColorFormat::Rgbw { .. } => 4,
            ColorFormat::Rgbcw { .. } => 5,
        }
    }

    /// Return the color order string
    pub fn order(&self) -> &str {
        match self {
            ColorFormat::Rgb(RgbFormat { order, .. }) => order,
            ColorFormat::Rgbw(RgbwFormat { order, .. }) => order,
            ColorFormat::Rgbcw(RgbcwFormat { order, .. }) => order,
        }
    }
}

impl Default for ColorFormat {
    fn default() -> Self {
        ColorFormat::Rgb(RgbFormat::default())
    }
}

impl Validate for ColorFormat {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            ColorFormat::Rgb(rgb_format) => rgb_format.validate(),
            ColorFormat::Rgbw(rgbw_format) => rgbw_format.validate(),
            ColorFormat::Rgbcw(rgbcw_format) => rgbcw_format.validate(),
        }
    }
}
