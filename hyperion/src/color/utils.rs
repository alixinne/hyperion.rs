//! Color utilities

use palette::LinSrgb;

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

/// Get the minimum value of all channels
///
/// # Parameters
///
/// * `c`: RGB color to get the minimum of
pub fn color_min(c: LinSrgb) -> f32 {
    let (r, g, b) = c.into_components();
    min!(r, g, b)
}

/// Max value
macro_rules! max {
    ($x: expr) => ($x);
    ($x: expr, $($z: expr),+) => {{
        let y = max!($($z),*);
        if $x > y {
            $x
        } else {
            y
        }
    }}
}

/// Get the maximum value of all channels
///
/// # Parameters
///
/// * `c`: RGB color to get the maximum of
pub fn color_max(c: LinSrgb) -> f32 {
    let (r, g, b) = c.into_components();
    max!(r, g, b)
}

/// Return the whitepoint for a given color temperature
///
/// # Parameters
///
/// * `t`: temperature in Kelvin
pub fn kelvin_to_rgb(t: f32) -> LinSrgb {
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

/// Get the sRGB whitepoint
pub fn srgb_white() -> LinSrgb {
    LinSrgb::from_components((1.0, 1.0, 1.0))
}

/// Transforms the given color to fix its white balance
///
/// # Parameters
///
/// * `c`: color to transform
/// * `src_white`: whitepoint of the current space of `c`
/// * `dst_white`: whitepoint of the destination space of `c`
pub fn whitebalance(c: LinSrgb, src_white: LinSrgb, dst_white: LinSrgb) -> LinSrgb {
    let corr = dst_white / src_white;
    c * corr / color_max(corr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_white() {
        assert!(ulps_eq!(
            kelvin_to_rgb(6600.0),
            LinSrgb::from_components((1.0, 1.0, 1.0))
        ));
    }
}
