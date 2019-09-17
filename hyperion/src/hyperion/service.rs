//! Definition of the Service type

use std::time::Instant;

use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

use futures::sync::mpsc;
use futures::{Async, Future, Poll, Stream};

use crate::color;
use crate::config::{ConfigHandle, ReloadHints};
use crate::image::Processor;
use crate::runtime::{Devices, EffectEngine, MuxedInput, PriorityMuxer};

use super::*;

/// Hyperion service state
pub struct Service {
    /// Priority muxer
    priority_muxer: PriorityMuxer,
    /// Effect engine
    effect_engine: Arc<Mutex<EffectEngine>>,
    /// Device runtime data
    devices: Devices,
    /// Image processor
    image_processor: Processor<f32>,
    /// Debug listener
    debug_listener: Option<std::sync::mpsc::Sender<DebugMessage>>,
}

/// Type of the sender end of the channel for Hyperion inputs
pub type ServiceInputSender = mpsc::UnboundedSender<Input>;

impl Service {
    /// Create a new Service instance
    ///
    /// # Parameters
    ///
    /// * `config`: configuration to derive this instance from
    /// * `debug_listener`: channel to send debug updates to.
    pub fn new(
        config: ConfigHandle,
        debug_listener: Option<std::sync::mpsc::Sender<DebugMessage>>,
    ) -> Result<(Self, ServiceInputSender), HyperionError> {
        // TODO: check channel capacity
        let (sender, receiver) = mpsc::unbounded();

        let devices = Devices::try_from(config.clone()).map_err(HyperionError::from)?;

        let effect_engine = Arc::new(Mutex::new(EffectEngine::new(vec!["effects/".into()])));

        let priority_muxer = PriorityMuxer::new(
            receiver,
            effect_engine.clone(),
            sender.clone(),
            devices.get_led_count(),
        );

        Ok((
            Self {
                priority_muxer,
                effect_engine,
                devices,
                image_processor: Default::default(),
                debug_listener,
            },
            sender,
        ))
    }

    /// Get a handle to the effect engine
    pub fn get_effect_engine(&self) -> Arc<Mutex<EffectEngine>> {
        self.effect_engine.clone()
    }

    /// Handle an incoming state update
    ///
    /// # Parameters
    ///
    /// * `update`: state update message
    fn handle_update(&mut self, update: StateUpdate) {
        // Forward state update to the debug listener if we have one
        if let Some(debug_listener) = self.debug_listener.as_ref() {
            debug_listener
                .send(DebugMessage::StateUpdate(update.clone()))
                .unwrap_or_else(|e| {
                    error!("failed to forward state update to listener: {:?}", e);
                    self.debug_listener = None;
                });
        }

        let now = Instant::now();

        match update {
            StateUpdate::Clear => {
                debug!("clearing all leds");
                self.devices
                    .set_all_leds(now, color::ColorPoint::default(), false);
            }
            StateUpdate::SolidColor { color } => {
                debug!("setting all leds to {}", color);
                self.devices.set_all_leds(now, color, false);
            }
            StateUpdate::Image(raw_image) => {
                let (width, height) = raw_image.get_dimensions();
                debug!("incoming {}x{} image", width, height);

                self.devices
                    .set_from_image(now, &mut self.image_processor, raw_image, false);
            }
            StateUpdate::LedData(leds) => {
                debug!("setting {} leds from color data", leds.len());
                self.devices.set_leds(now, leds, false)
            }
        }
    }

    /// Handle an incoming service command
    ///
    /// # Parameters
    ///
    /// * `service_command`: command to handle
    fn handle_command(&mut self, service_command: ServiceCommand) {
        match service_command {
            ServiceCommand::ReloadDevice {
                device_index,
                reload_hints,
            } => match self.devices.reload_device(device_index, reload_hints) {
                Ok(_) => {
                    if reload_hints.contains(ReloadHints::DEVICE_LEDS) {
                        // Reload the image processor if the LEDs changed
                        trace!("clearing the image processor cache");
                        self.image_processor = Default::default();
                    }
                }
                Err(error) => {
                    error!("error while reloading device: {}", error);
                }
            },
        }
    }
}

impl Future for Service {
    type Item = ();
    type Error = HyperionError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Poll channel for state updates
        while let Async::Ready(value) = self.priority_muxer.poll()? {
            if let Some(muxed_input) = value {
                match muxed_input {
                    MuxedInput::StateUpdate(state_update) => self.handle_update(state_update),
                    MuxedInput::Internal(service_command) => self.handle_command(service_command),
                }
            } else {
                return Ok(Async::Ready(()));
            }
        }

        // Update devices
        try_ready!(self.devices.poll());

        Ok(Async::NotReady)
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        if let Some(debug_listener) = self.debug_listener.as_ref() {
            debug_listener
                .send(DebugMessage::Terminating)
                .unwrap_or_else(|e| {
                    error!("failed to send Terminating message to listener: {:?}", e);
                });
        }
    }
}
