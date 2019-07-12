//! Definition of the ColorFormat type

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
fn default_rgb() -> f32 {
    6800.
}

/// Default RGBW white temperature
fn default_rgbw_white() -> f32 {
    6500.
}

/// Default RGBCW cold white temperature
fn default_rgbcw_cold_white() -> f32 {
    6500.
}

/// Default RGBCW warm white temperature
fn default_rgbcw_warm_white() -> f32 {
    2800.
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
        /// RGB White temperature
        #[serde(default = "default_rgb")]
        rgb: f32,
    },
    /// RGB + White
    #[serde(rename = "rgbw")]
    Rgbw {
        /// LED order string
        #[serde(default = "default_rgbw_order")]
        order: String,
        /// RGB White temperature
        #[serde(default = "default_rgb")]
        rgb: f32,
        /// White temperature (Kelvin)
        #[serde(default = "default_rgbw_white")]
        white: f32,
    },
    /// RGB + Cold white + Warm white
    #[serde(rename = "rgbcw")]
    Rgbcw {
        /// LED order string
        #[serde(default = "default_rgbcw_order")]
        order: String,
        /// RGB White temperature
        #[serde(default = "default_rgb")]
        rgb: f32,
        /// Cold white temperature (Kelvin)
        #[serde(default = "default_rgbcw_cold_white")]
        cold_white: f32,
        /// Warm white temperature (Kelvin)
        #[serde(default = "default_rgbcw_warm_white")]
        warm_white: f32,
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
        }
    }
}
