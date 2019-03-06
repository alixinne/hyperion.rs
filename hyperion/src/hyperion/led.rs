/// Floating-point range in a picture
#[derive(Debug, Serialize, Deserialize)]
pub struct ScanRange {
    pub minimum: f32,
    pub maximum: f32,
}

impl Default for ScanRange {
    fn default() -> Self {
        Self {
            minimum: 0.0,
            maximum: 1.0,
        }
    }
}

/// Basic element of the Hyperion internal state
///
/// It defines the area of the screen it maps to (hscan, vscan), in the context
/// of a given device. The index is unique within the device.
///
/// It holds its current color as a linear-space RGB value in image space. The
/// corresponding value on the target devices is computed downstream by the
/// filters.
#[derive(Debug, Serialize, Deserialize)]
pub struct Led {
    pub index: i32,
    pub hscan: ScanRange,
    pub vscan: ScanRange,
    #[serde(skip_deserializing, skip_serializing)]
    pub current_color: palette::LinSrgb,
}

impl Default for Led {
    fn default() -> Self {
        Self {
            index: 0,
            hscan: ScanRange::default(),
            vscan: ScanRange::default(),
            current_color: palette::LinSrgb::from_components((0.1, 0.2, 0.3))
        }
    }
}
