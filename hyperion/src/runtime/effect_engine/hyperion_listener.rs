//! Definition of the HyperionListener type

use pyo3::prelude::*;
use pyo3::{exceptions, PyErr};

use super::{ByteRgb, EffectListener};

use crate::color::ColorPoint;
use crate::hyperion::{Input, ServiceInputSender, StateUpdate};
use crate::image::RawImage;

/// Implementation of EffectListener for the Hyperion service
pub struct HyperionListener {
    /// Update channel
    sender: ServiceInputSender,
}

impl HyperionListener {
    /// Create a new HyperionListener
    ///
    /// # Parameters
    ///
    /// * `sender`: update channel
    pub fn new(sender: ServiceInputSender) -> HyperionListener {
        Self { sender }
    }
}

impl EffectListener for HyperionListener {
    fn set_rgb(&mut self, rgb: (u8, u8, u8)) -> PyResult<()> {
        self.sender
            .unbounded_send(Input::effect_input(StateUpdate::SolidColor {
                color: rgb.into(),
            }))
            .map_err(|error| PyErr::new::<exceptions::RuntimeError, _>(error.to_string()))
    }

    fn set_leds_rgb(&mut self, leds: &[ByteRgb]) -> PyResult<()> {
        self.sender
            .unbounded_send(Input::effect_input(StateUpdate::LedData(
                leds.iter()
                    .map(|rgb| ColorPoint::from((rgb.r, rgb.g, rgb.b)))
                    .collect(),
            )))
            .map_err(|error| PyErr::new::<exceptions::RuntimeError, _>(error.to_string()))
    }

    fn set_image(&mut self, image: RawImage) -> PyResult<()> {
        self.sender
            .unbounded_send(Input::effect_input(StateUpdate::Image(image)))
            .map_err(|error| PyErr::new::<exceptions::RuntimeError, _>(error.to_string()))
    }
}
