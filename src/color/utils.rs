//! Color utilities

use palette::LinSrgb;

use crate::models::{Color, Color16};

/// Return the whitepoint for a given color temperature
///
/// # Parameters
///
/// * `t`: temperature in Kelvin
fn kelvin_to_rgbf32(t: f32) -> LinSrgb {
    let t = f64::from(t);

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
        329.698_727_446 * (t - 60.0).powf(-0.133_204_759_2)
    };

    let g = if t <= 66.0 {
        99.470_802_586_1 * t.ln() - 161.119_568_166_1
    } else {
        288.122_169_528_3 * (t - 60.0).powf(-0.075_514_849_2)
    };

    let b = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_731_223_1 * (t - 10.0).ln() - 305.044_792_730_7
    };

    let r = if r > 255.0 { 255.0 } else { r };
    let g = if g > 255.0 { 255.0 } else { g };
    let b = if b > 255.0 { 255.0 } else { b };

    LinSrgb::from_components(((r / 255.0) as f32, (g / 255.0) as f32, (b / 255.0) as f32))
}

pub fn kelvin_to_rgb16(t: u32) -> Color16 {
    let (r, g, b) = kelvin_to_rgbf32(t as f32).into_components();
    Color16::new(
        (r * (u16::MAX as f32)) as u16,
        (g * (u16::MAX as f32)) as u16,
        (b * (u16::MAX as f32)) as u16,
    )
}

/// Get the sRGB whitepoint
pub fn srgb_white() -> Color16 {
    Color16::new(u16::MAX, u16::MAX, u16::MAX)
}

/// Transforms the given color to fix its white balance
///
/// # Parameters
///
/// * `c`: color to transform
/// * `src_white`: whitepoint of the current space of `c`
/// * `dst_white`: whitepoint of the destination space of `c`
pub fn whitebalance(c: Color16, src_white: Color16, dst_white: Color16) -> Color16 {
    let (cr, cg, cb) = c.into_components();
    let (sr, sg, sb) = src_white.into_components();
    let (dr, dg, db) = dst_white.into_components();

    Color16::new(
        ((cr as u32 * dr as u32) / sr as u32) as u16,
        ((cg as u32 * dg as u32) / sg as u32) as u16,
        ((cb as u32 * db as u32) / sb as u32) as u16,
    )
}

const FACTOR: u16 = 65535 / 255;

pub fn color_to8(color: Color16) -> Color {
    let (r, g, b) = color.into_components();
    Color::new((r / FACTOR) as u8, (g / FACTOR) as u8, (b / FACTOR) as u8)
}

pub fn color_to16(color: Color) -> Color16 {
    let (r, g, b) = color.into_components();
    Color16::new(
        (r as u16) * FACTOR,
        (g as u16) * FACTOR,
        (b as u16) * FACTOR,
    )
}
