//! Definition of the StateUpdate type

use std::time::Instant;

use crate::color;
use crate::image;

/// State update data
#[derive(Clone)]
pub enum StateUpdateKind {
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

impl std::fmt::Debug for StateUpdateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StateUpdateKind::Clear => write!(f, "Clear"),
            StateUpdateKind::SolidColor { color } => {
                write!(f, "SolidColor {{ color: {:?} }}", color)
            }
            StateUpdateKind::Image(image) => {
                let (width, height) = image.get_dimensions();
                write!(f, "Image({}x{} image)", width, height)
            }
            StateUpdateKind::LedData(led_data) => write!(f, "LedData({} LEDs)", led_data.len()),
        }
    }
}

/// State update messages for the Hyperion service
#[derive(Debug, Clone)]
pub struct StateUpdate {
    /// Instant at which this update was requested
    pub initiated: Instant,
    /// Type of update
    pub kind: StateUpdateKind,
}

impl StateUpdate {
    /// Update the initiated time to now
    pub fn recreate(self) -> Self {
        Self {
            initiated: Instant::now(),
            kind: self.kind,
        }
    }

    /// Create a Clear StateUpdate
    pub fn clear() -> Self {
        Self {
            initiated: Instant::now(),
            kind: StateUpdateKind::Clear,
        }
    }

    /// Create a SolidColor StateUpdate
    pub fn solid(color: color::ColorPoint) -> Self {
        Self {
            initiated: Instant::now(),
            kind: StateUpdateKind::SolidColor { color },
        }
    }

    /// Create an Image StateUpdate
    pub fn image(image: image::RawImage) -> Self {
        Self {
            initiated: Instant::now(),
            kind: StateUpdateKind::Image(image),
        }
    }

    /// Create a LedData StateUpdate
    pub fn led_data(led_data: Vec<color::ColorPoint>) -> Self {
        Self {
            initiated: Instant::now(),
            kind: StateUpdateKind::LedData(led_data),
        }
    }
}
