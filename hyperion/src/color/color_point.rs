//! ColorPoint type definition

use std::fmt;
use std::ops::{Add, Mul};

use palette::{Blend, LinSrgb};

use super::DeviceColor;
use crate::config;

/// Represents a color in an arbitrary color space
///
/// Operations that require specific spaces will automatically convert this color to the right
/// space before operating on it.
#[derive(Default, Debug, Clone, Copy)]
pub struct ColorPoint {
    /// Value of this color point
    value: palette::Color,
}

/// Return the whitepoint for a given temperature
///
/// # Parameters
///
/// * `t`: temperature in Kelvin
fn get_whitepoint(t: f32) -> LinSrgb {
    // http://www.tannerhelland.com/4435/convert-temperature-rgb-algorithm-code/
    //
    // Check bounds on temperature
    let t = if t > 40000.0 { 40000.0 } else { t };
    let t = if t < 1000.0 { 1000.0 } else { t };

    // Scale
    let t = t / 100.0;

    let r = if t <= 66.0 {
        255.0
    } else {
        329.698727446 * (t - 60.0).powf(-0.1332047592)
    };

    let g = if t <= 66.0 {
        99.4708025861 * t.ln() - 161.1195681661
    } else {
        288.1221695283 * (t - 60.0).powf(-0.0755148492)
    };

    let b = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.5177312231 * (t - 10.0).ln() - 305.0447927307
    };

    let r = if r > 255.0 { 255.0 } else { r };
    let g = if g > 255.0 { 255.0 } else { g };
    let b = if b > 255.0 { 255.0 } else { b };

    LinSrgb::from_components((r / 255.0, g / 255.0, b / 255.0))
}

/// Transforms the given color to fix its white balance
fn whitebalance(c: LinSrgb, src_white: LinSrgb, dst_white: LinSrgb) -> LinSrgb {
    let corr = dst_white / src_white;
    c * corr
}

/// Get the LinSrgb white
fn srgb_white() -> LinSrgb {
    LinSrgb::from_components((1.0, 1.0, 1.0))
}

/// Min value
macro_rules! min {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = min!($($z),*);
        if $x < y {
            $x
        } else {
            y
        }
    }}
}

/// Get the min value of all channels
fn color_min(c: LinSrgb) -> f32 {
    let (r, g, b) = c.into_components();
    min!(r, g, b)
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
        LinSrgb::from(self.value).into_components()
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

    /// Convert this color point to a device color
    ///
    /// # Parameters
    ///
    /// * `format`: color format to convert to
    pub fn to_device(&self, format: &config::ColorFormat) -> DeviceColor {
        match format {
            config::ColorFormat::Rgb { rgb, gamma, .. } => {
                // Whitebalance the RGB white
                let (r, g, b) = whitebalance(
                    LinSrgb::from(self.value),
                    srgb_white(),
                    get_whitepoint(*rgb),
                )
                .into_components();

                DeviceColor::Rgb {
                    r: r.powf(gamma.r),
                    g: g.powf(gamma.g),
                    b: b.powf(gamma.b),
                }
            }
            config::ColorFormat::Rgbw { rgb, white, gamma, .. } => {
                let rgb_value = LinSrgb::from(self.value);
                let dest_white = get_whitepoint(*white);

                // Move RGB value to white space
                let white_rgb = whitebalance(rgb_value, srgb_white(), dest_white);

                // Get white value
                let w = color_min(white_rgb);

                // Adjust value
                let rgb_value = white_rgb - LinSrgb::from_components((w, w, w));

                // Whitebalance the RGB white
                let (r, g, b) =
                    whitebalance(rgb_value, dest_white, get_whitepoint(*rgb)).into_components();

                DeviceColor::Rgbw {
                    r: r.powf(gamma.r),
                    g: g.powf(gamma.g),
                    b: b.powf(gamma.b),
                    w: w.powf(gamma.w),
                }
            }
            config::ColorFormat::Rgbcw { rgb, gamma, .. } => {
                // Whitebalance the RGB white
                let (r, g, b) = whitebalance(
                    LinSrgb::from(self.value),
                    srgb_white(),
                    get_whitepoint(*rgb),
                )
                .into_components();

                // TODO: Implement RGBCW
                DeviceColor::Rgbcw {
                    r: r.powf(gamma.r),
                    g: g.powf(gamma.g),
                    b: b.powf(gamma.b),
                    c: 0.0f32.powf(gamma.c),
                    w: 0.0f32.powf(gamma.w),
                }
            }
        }
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
            value: palette::Color::from(LinSrgb::from(self.value) * rhs),
        }
    }
}
