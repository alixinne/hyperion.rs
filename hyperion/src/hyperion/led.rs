/// Floating-point range in a picture
#[derive(Debug, Serialize, Deserialize, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Led {
    pub hscan: ScanRange,
    pub vscan: ScanRange,
}

/// Instance of a LED at runtime
///
/// Combines the specification of the LED coverage of the screen area plus
/// its current state.
#[derive(Debug, Default)]
pub struct LedInstance {
    pub spec: Led,
    pub current_color: palette::LinSrgb,
}

impl LedInstance {
    /// Initialize a new LedInstance from its corresponding Led object
    ///
    /// # Parameters
    ///
    /// * `spec`: specification for this LED
    pub fn new(spec: &Led) -> Self {
        Self {
            spec: (*spec).clone(),
            current_color: palette::LinSrgb::default(),
        }
    }
}
