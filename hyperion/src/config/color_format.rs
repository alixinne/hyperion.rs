//! Definition of the ColorFormat type

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
#[derive(Debug, Serialize, Deserialize)]
pub struct RgbGamma {
    /// Red channel gamma
    #[serde(default = "default_gamma")]
    pub r: f32,
    /// Green channel gamma
    #[serde(default = "default_gamma")]
    pub g: f32,
    /// Blue channel gamma
    #[serde(default = "default_gamma")]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct RgbwGamma {
    /// Red channel gamma
    #[serde(default = "default_gamma")]
    pub r: f32,
    /// Green channel gamma
    #[serde(default = "default_gamma")]
    pub g: f32,
    /// Blue channel gamma
    #[serde(default = "default_gamma")]
    pub b: f32,
    /// White channel gamma
    #[serde(default = "default_gamma")]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct RgbcwGamma {
    /// Red channel gamma
    #[serde(default = "default_gamma")]
    pub r: f32,
    /// Green channel gamma
    #[serde(default = "default_gamma")]
    pub g: f32,
    /// Blue channel gamma
    #[serde(default = "default_gamma")]
    pub b: f32,
    /// Cold white channel gamma,
    #[serde(default = "default_gamma")]
    pub c: f32,
    /// Warm white channel gamma
    #[serde(default = "default_gamma")]
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

/// Color data format used by a device
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ColorFormat {
    /// RGB
    #[serde(rename = "rgb")]
    Rgb {
        /// LED order string
        #[serde(default = "default_rgb_order")]
        order: String,
        /// RGB White point
        #[serde(default = "default_rgb")]
        rgb: ColorPoint,
        /// Gamma values
        #[serde(default)]
        gamma: RgbGamma,
    },
    /// RGB + White
    #[serde(rename = "rgbw")]
    Rgbw {
        /// LED order string
        #[serde(default = "default_rgbw_order")]
        order: String,
        /// RGB White temperature
        #[serde(default = "default_rgb")]
        rgb: ColorPoint,
        /// White temperature (Kelvin)
        #[serde(default = "default_rgbw_white")]
        white: ColorPoint,
        /// Gamma values
        #[serde(default)]
        gamma: RgbwGamma,
    },
    /// RGB + Cold white + Warm white
    #[serde(rename = "rgbcw")]
    Rgbcw {
        /// LED order string
        #[serde(default = "default_rgbcw_order")]
        order: String,
        /// RGB White temperature
        #[serde(default = "default_rgb")]
        rgb: ColorPoint,
        /// Cold white temperature (Kelvin)
        #[serde(default = "default_rgbcw_cold_white")]
        cold_white: ColorPoint,
        /// Warm white temperature (Kelvin)
        #[serde(default = "default_rgbcw_warm_white")]
        warm_white: ColorPoint,
        /// Gamma values
        #[serde(default)]
        gamma: RgbcwGamma,
    },
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
            ColorFormat::Rgb { order, .. } => order,
            ColorFormat::Rgbw { order, .. } => order,
            ColorFormat::Rgbcw { order, .. } => order,
        }
    }
}

impl Default for ColorFormat {
    fn default() -> Self {
        ColorFormat::Rgb {
            order: default_rgb_order(),
            rgb: default_rgb(),
            gamma: Default::default(),
        }
    }
}
