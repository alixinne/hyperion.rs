//! ColorPoint type definition

use super::ColorValue;

use std::fmt;
use std::ops::{Add, Mul, Sub};

/// Represents a color in an arbitrary color space
///
/// Operations that require specific spaces will automatically convert this color to the right
/// space before operating on it.
#[derive(Default, Debug, Clone, Copy)]
pub struct ColorPoint {
    /// Value of this color point
    value: ColorValue,
}

impl ColorPoint {
    /// Create a new color point from raw linear RGB component values
    ///
    /// # Parameters
    ///
    /// * `rgb`: RGB component values
    pub fn from_rgb(rgb: (f32, f32, f32)) -> Self {
        Self {
            value: ColorValue::from(palette::LinSrgb::from_components(rgb)),
        }
    }

    /// Return the linear RGB components of this color point
    pub fn as_rgb(&self) -> (f32, f32, f32) {
        self.value.into_rgb()
    }
}

impl fmt::Display for ColorPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (r, g, b) = self.value.into_rgb();
        write!(f, "({}, {}, {})", r, g, b)
    }
}

impl Add<ColorPoint> for ColorPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            value: ColorValue::from(self.value.into_lin_srgb() + rhs.value.into_lin_srgb()),
        }
    }
}

impl Sub<ColorPoint> for ColorPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            value: ColorValue::from(self.value.into_lin_srgb() - rhs.value.into_lin_srgb()),
        }
    }
}

impl Mul<f32> for ColorPoint {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            value: ColorValue::from(self.value.into_lin_srgb() * rhs),
        }
    }
}
