//! Definition of the EffectListener trait

use pyo3::prelude::*;

use super::ByteRgb;
use crate::image::RawImage;

/// Trait for an object that listens to updates from Python scripts
pub trait EffectListener {
    /// Set all LEDs to a solid color
    ///
    /// # Parameters
    ///
    /// * `rgb`: color to set
    fn set_rgb(&mut self, rgb: (u8, u8, u8)) -> PyResult<()>;
    /// Set all LEDs from RGB data
    ///
    /// # Parameters
    ///
    /// * `leds`: slice of LED RGB data
    fn set_leds_rgb(&mut self, leds: &[ByteRgb]) -> PyResult<()>;
    /// Set LED colors from an image
    ///
    /// # Parameters
    ///
    /// * `image`: RGB image data
    fn set_image(&mut self, image: RawImage) -> PyResult<()>;
}
