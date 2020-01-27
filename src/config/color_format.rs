//! Definition of the ColorFormat type

use lazy_static::lazy_static;
use regex::Regex;
use validator::{Validate, ValidationErrors};

use crate::color::ColorPoint;

/// RGB Gamma data
#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(default)]
pub struct RgbGamma {
    /// Red channel gamma
    #[validate(range(min = 0.0))]
    pub r: f32,
    /// Green channel gamma
    #[validate(range(min = 0.0))]
    pub g: f32,
    /// Blue channel gamma
    #[validate(range(min = 0.0))]
    pub b: f32,
}

impl Default for RgbGamma {
    fn default() -> Self {
        Self {
            r: 2.2,
            g: 2.2,
            b: 2.2,
        }
    }
}

/// RGBW Gamma data
#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(default)]
pub struct RgbwGamma {
    /// Red channel gamma
    #[validate(range(min = 0.0))]
    pub r: f32,
    /// Green channel gamma
    #[validate(range(min = 0.0))]
    pub g: f32,
    /// Blue channel gamma
    #[validate(range(min = 0.0))]
    pub b: f32,
    /// White channel gamma
    #[validate(range(min = 0.0))]
    pub w: f32,
}

impl Default for RgbwGamma {
    fn default() -> Self {
        Self {
            r: 2.2,
            g: 2.2,
            b: 2.2,
            w: 2.2,
        }
    }
}

/// RGBCW Gamma data
#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(default)]
pub struct RgbcwGamma {
    /// Red channel gamma
    #[validate(range(min = 0.0))]
    pub r: f32,
    /// Green channel gamma
    #[validate(range(min = 0.0))]
    pub g: f32,
    /// Blue channel gamma
    #[validate(range(min = 0.0))]
    pub b: f32,
    /// Cold white channel gamma,
    #[validate(range(min = 0.0))]
    pub c: f32,
    /// Warm white channel gamma
    #[validate(range(min = 0.0))]
    pub w: f32,
}

impl Default for RgbcwGamma {
    fn default() -> Self {
        Self {
            r: 2.2,
            g: 2.2,
            b: 2.2,
            c: 2.2,
            w: 2.2,
        }
    }
}

lazy_static! {
    static ref ORDER_REGEX: Regex = Regex::new(r"^[rgbcw]*$").unwrap();
}

/// RGB format data
#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(default)]
pub struct RgbFormat {
    /// LED order string
    #[validate(regex = "ORDER_REGEX")]
    pub order: String,
    /// RGB White point
    pub rgb: ColorPoint,
    /// Gamma values
    pub gamma: RgbGamma,
}

impl Default for RgbFormat {
    fn default() -> Self {
        Self {
            order: "rgb".to_owned(),
            rgb: ColorPoint::srgb_white(),
            gamma: Default::default(),
        }
    }
}

/// RGBW format data
#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(default)]
pub struct RgbwFormat {
    /// LED order string
    #[validate(regex = "ORDER_REGEX")]
    pub order: String,
    /// RGB White temperature
    pub rgb: ColorPoint,
    /// White temperature (Kelvin)
    pub white: ColorPoint,
    /// Gamma values
    pub gamma: RgbwGamma,
    /// Relative power of the white LED
    ///
    /// 1.0 means the white LED emits the same amount of power as the RGB LED when fully lit. 2.0
    ///   means the white LED emits twice as much light as the RGB LED for the same input level.
    pub white_factor: f32,
}

impl Default for RgbwFormat {
    fn default() -> Self {
        Self {
            order: "rgbw".to_owned(),
            rgb: ColorPoint::srgb_white(),
            white: ColorPoint::srgb_white(),
            gamma: Default::default(),
            white_factor: 1.0,
        }
    }
}

/// RGBCW format data
#[derive(Clone, Debug, Validate, Serialize, Deserialize)]
#[serde(default)]
pub struct RgbcwFormat {
    /// LED order string
    #[validate(regex = "ORDER_REGEX")]
    pub order: String,
    /// RGB White temperature
    pub rgb: ColorPoint,
    /// Cold white temperature (Kelvin)
    pub cold_white: ColorPoint,
    /// Warm white temperature (Kelvin)
    pub warm_white: ColorPoint,
    /// Gamma values
    pub gamma: RgbcwGamma,
}

impl Default for RgbcwFormat {
    fn default() -> Self {
        Self {
            order: "rgbcw".to_owned(),
            rgb: ColorPoint::srgb_white(),
            cold_white: ColorPoint::srgb_white(),
            warm_white: ColorPoint::from_kelvin(2800.),
            gamma: Default::default(),
        }
    }
}

/// Color data format used by a device
#[derive(Clone, Debug, Serialize, Deserialize)]
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
