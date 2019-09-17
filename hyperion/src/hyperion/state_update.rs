//! Definition of the StateUpdate type

use crate::color;
use crate::image;

/// State update messages for the Hyperion service
#[derive(Debug, Clone)]
pub enum StateUpdate {
    /// Clear all devices
    Clear,
    /// Set all devices to a given color
    SolidColor {
        /// Color to apply to the devices
        color: color::ColorPoint,
    },
    /// Use given image to set colors
    Image(image::RawImage),
    /// Per-LED color data
    LedData(Vec<color::ColorPoint>),
}
