//! Definition of the Led type

use validator::Validate;

use super::ScanRange;

/// Basic element of the Hyperion internal state
///
/// It defines the area of the screen it maps to (hscan, vscan), in the context
/// of a given device. The index is unique within the device.
///
/// It holds its current color as a linear-space RGB value in image space. The
/// corresponding value on the target devices is computed downstream by the
/// filters.
#[derive(Debug, Validate, Serialize, Deserialize, Clone, Default)]
pub struct Led {
    /// Horizontal scan range
    ///
    /// Horizontal span on screen that this LED covers.
    #[validate]
    pub hscan: ScanRange,
    /// Vertical scan range
    ///
    /// Vertical span on screen that this LED covers.
    #[validate]
    pub vscan: ScanRange,
}
