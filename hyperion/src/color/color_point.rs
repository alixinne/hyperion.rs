//! ColorPoint type definition

use std::fmt;
use std::ops::{Add, Mul};

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

use palette::{Blend, LinSrgb};

use super::*;
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

impl ColorPoint {
    /// Return the sRGB whitepoint
    pub fn srgb_white() -> Self {
        Self {
            value: palette::Color::from(super::srgb_white()),
        }
    }

    /// Return the color corresponding the given color temperature
    ///
    /// # Parameters
    ///
    /// * `temperature`: color temperature, in Kelvin
    pub fn from_kelvin(temperature: f32) -> Self {
        Self {
            value: palette::Color::from(super::kelvin_to_rgb(temperature))
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
                let (r, g, b) =
                    whitebalance(LinSrgb::from(self.value), rgb.value.into(), srgb_white())
                        .into_components();

                DeviceColor::Rgb {
                    r: r.powf(gamma.r),
                    g: g.powf(gamma.g),
                    b: b.powf(gamma.b),
                }
            }
            config::ColorFormat::Rgbw {
                rgb, white, gamma, ..
            } => {
                let rgb_value = LinSrgb::from(self.value);
                let dest_white = white.value.into();

                // Move RGB value to white space
                let white_rgb = whitebalance(rgb_value, dest_white, srgb_white());

                // Get white value
                let w = color_min(white_rgb);

                // Adjust value
                let rgb_value = white_rgb - LinSrgb::from_components((w, w, w));

                // Whitebalance the RGB white
                let (r, g, b) =
                    whitebalance(rgb_value, rgb.value.into(), dest_white).into_components();

                DeviceColor::Rgbw {
                    r: r.powf(gamma.r),
                    g: g.powf(gamma.g),
                    b: b.powf(gamma.b),
                    w: w.powf(gamma.w),
                }
            }
            config::ColorFormat::Rgbcw { rgb, gamma, .. } => {
                // Whitebalance the RGB white
                let (r, g, b) =
                    whitebalance(LinSrgb::from(self.value), rgb.value.into(), srgb_white())
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

impl From<(u8, u8, u8)> for ColorPoint {
    /// Create a new color point from raw linear RGB component values
    ///
    /// # Parameters
    ///
    /// * `rgb`: RGB component values
    fn from(rgb: (u8, u8, u8)) -> Self {
        Self {
            value: palette::Color::linear_rgb(
                f32::from(rgb.0) / 255.0,
                f32::from(rgb.1) / 255.0,
                f32::from(rgb.2) / 255.0,
            ),
        }
    }
}

impl From<(f32, f32, f32)> for ColorPoint {
    /// Create a new color point from raw linear RGB component values
    ///
    /// # Parameters
    ///
    /// * `rgb`: RGB component values
    fn from(rgb: (f32, f32, f32)) -> Self {
        Self {
            value: palette::Color::linear_rgb(rgb.0, rgb.1, rgb.2),
        }
    }
}

impl From<LinSrgb> for ColorPoint {
    fn from(color: LinSrgb) -> Self {
        Self {
            value: palette::Color::from(color),
        }
    }
}

impl fmt::Display for ColorPoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (r, g, b) = self.as_rgb();
        write!(f, "rgb({}, {}, {})", r, g, b)
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

impl Serialize for ColorPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let (r, g, b) = self.as_rgb();
        serializer.serialize_str(&format!("rgb({}, {}, {})", r, g, b))
    }
}

/// Serde visitor for deserializing ColorPoint
struct ColorPointVisitor;

impl<'de> Visitor<'de> for ColorPointVisitor {
    type Value = ColorPoint;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an rgb color as rgb(r, g, b) or color temperature in kelvins as xxxxK")
    }

    fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // Trim whitespace
        let string = string.trim();

        if string.starts_with("rgb(") && string.ends_with(')') {
            // rgb(R, G, B) format
            let color_start = string.find('(').map(|p| p + 1);
            let color_end = string.find(')');

            if let (Some(start), Some(end)) = (color_start, color_end) {
                let split: Result<Vec<_>, _> = string[start..end]
                    .split(',')
                    .map(|s| s.trim().parse::<f32>())
                    .collect();

                if let Ok(components) = split {
                    if components.len() == 3 {
                        return Ok(ColorPoint::from((
                            components[0],
                            components[1],
                            components[2],
                        )));
                    }
                }
            }

            Err(E::custom(format!("failed to parse rgb color: {}", string)))
        } else if string.starts_with('#') {
            // #RRGGBB format
            let rr = u8::from_str_radix(&string[1..3], 16);
            let rg = u8::from_str_radix(&string[3..5], 16);
            let rb = u8::from_str_radix(&string[5..7], 16);

            if let (Ok(r), Ok(g), Ok(b)) = (rr, rg, rb) {
                Ok(ColorPoint::from((r, g, b)))
            } else {
                Err(E::custom(format!("failed to parse hex color: {}", string)))
            }
        } else if string.ends_with('K') || string.ends_with('k') {
            // Kelvin
            if let Ok(temperature) = string.trim_end_matches(|c| c == 'k' || c == 'K').parse() {
                if temperature < 1000.0 || temperature > 40000.0 {
                    Err(E::custom(format!(
                        "color temperature out of range [1000.0, 40000.0]: {}",
                        string
                    )))
                } else {
                    Ok(ColorPoint::from(kelvin_to_rgb(temperature)))
                }
            } else {
                Err(E::custom(format!(
                    "failed to parse color temperature: {}",
                    string
                )))
            }
        } else {
            Err(E::custom(format!("unknown color format: {}", string)))
        }
    }
}

impl<'de> Deserialize<'de> for ColorPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorPointVisitor)
    }
}
