//! Definition of the ColorValue type

/// A color in a given color space
#[derive(Debug, Clone, Copy)]
pub enum ColorValue {
    /// A linear RGB color value
    Rgb(palette::LinSrgb),
}

impl ColorValue {
    /// Return the linear RGB components of this color value
    pub fn into_rgb(self) -> (f32, f32, f32) {
        match self {
            ColorValue::Rgb(color) => color.into_components(),
        }
    }

    /// Return the color value as a linear RGB value
    pub fn into_lin_srgb(self) -> palette::LinSrgb {
        match self {
            ColorValue::Rgb(color) => color,
        }
    }
}

impl From<palette::LinSrgb> for ColorValue {
    fn from(lin_srgb: palette::LinSrgb) -> Self {
        ColorValue::Rgb(lin_srgb)
    }
}

impl Default for ColorValue {
    fn default() -> Self {
        ColorValue::Rgb(Default::default())
    }
}
