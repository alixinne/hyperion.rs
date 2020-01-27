//! Definition of the HyperionListener type

use pyo3::prelude::*;
use pyo3::{exceptions, PyErr};

use tokio::sync::mpsc;

use super::{ByteRgb, EffectListener};

use crate::color::ColorPoint;
use crate::hyperion::{Input, StateUpdate};
use crate::image::RawImage;

/// Implementation of EffectListener for the Hyperion service
pub struct HyperionListener {
    /// Update channel
    sender: mpsc::Sender<Input>,
}

impl HyperionListener {
    /// Create a new HyperionListener
    ///
    /// # Parameters
    ///
    /// * `sender`: update channel
    pub fn new(sender: mpsc::Sender<Input>) -> HyperionListener {
        Self { sender }
    }
}

impl EffectListener for HyperionListener {
    fn set_rgb(&mut self, rgb: (u8, u8, u8)) -> PyResult<()> {
        futures::executor::block_on(
            self.sender
                .send(Input::effect_input(StateUpdate::solid(rgb.into()))),
        )
        .map_err(|error| PyErr::new::<exceptions::RuntimeError, _>(error.to_string()))
    }

    fn set_leds_rgb(&mut self, leds: &[ByteRgb]) -> PyResult<()> {
        futures::executor::block_on(
            self.sender.send(Input::effect_input(StateUpdate::led_data(
                leds.iter()
                    .map(|rgb| ColorPoint::from((rgb.r, rgb.g, rgb.b)))
                    .collect(),
            ))),
        )
        .map_err(|error| PyErr::new::<exceptions::RuntimeError, _>(error.to_string()))
    }

    fn set_image(&mut self, image: RawImage) -> PyResult<()> {
        futures::executor::block_on(
            self.sender
                .send(Input::effect_input(StateUpdate::image(image))),
        )
        .map_err(|error| PyErr::new::<exceptions::RuntimeError, _>(error.to_string()))
    }
}
