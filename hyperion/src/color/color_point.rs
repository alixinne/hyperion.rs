//! ColorPoint type definition

use std::fmt;
use std::ops::{Add, Mul};

use palette::Blend;

/// Represents a color in an arbitrary color space
///
/// Operations that require specific spaces will automatically convert this color to the right
/// space before operating on it.
#[derive(Default, Debug, Clone, Copy)]
pub struct ColorPoint {
    /// Value of this color point
    value: palette::Color,
}

impl ColorPoint {
    /// Create a new color point from raw linear RGB component values
    ///
    /// # Parameters
    ///
    /// * `rgb`: RGB component values
    pub fn from_rgb(rgb: (f32, f32, f32)) -> Self {
        Self {
            value: palette::Color::linear_rgb(rgb.0, rgb.1, rgb.2),
        }
    }

    /// Return the linear RGB components of this color point
    pub fn as_rgb(&self) -> (f32, f32, f32) {
        palette::LinSrgb::from(self.value).into_components()
    }

    /// Return a number indicating the difference between the this color and the other
    ///
    /// # Parameters
    ///
    /// * `other`: other color to compare
    pub fn diff(&self, other: &Self) -> f32 {
        let (cr, cg, cb) = self.as_rgb();
        let (nr, ng, nb) = other.as_rgb();

        // Compute color difference
        (cr - nr).abs() + (cg - ng).abs() + (cb - nb).abs()
    }

    /// Return true if this color is pure black
    pub fn is_black(&self) -> bool {
        ulps_eq!(self.value, palette::Color::default())
    }
}

impl fmt::Display for ColorPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (r, g, b) = self.as_rgb();
        write!(f, "({}, {}, {})", r, g, b)
    }
}

impl Add<ColorPoint> for ColorPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            value: self.value.plus(rhs.value),
        }
    }
}

impl Mul<f32> for ColorPoint {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            value: palette::Color::from(palette::LinSrgb::from(self.value) * rhs),
        }
    }
}
