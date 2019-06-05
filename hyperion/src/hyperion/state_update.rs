//! Definition of the StateUpdate type

use crate::color;

/// State update messages for the Hyperion service
#[derive(Debug, Clone)]
pub enum StateUpdate {
    /// Clear all devices
    ClearAll,
    /// Set all devices to a given color
    SolidColor {
        /// Color to apply to the devices
        color: color::ColorPoint,
    },
    /// Use given image to set colors
    Image {
        /// Raw 8-bit RGB data
        data: Vec<u8>,
        /// Width of the image in `data`
        width: u32,
        /// Height of the image in `data`
        height: u32,
    },
}
