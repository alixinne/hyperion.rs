//! Definition of the Service type

use std::time::Instant;

use futures::sync::mpsc;
use futures::{Async, Future, Poll, Stream};

use crate::color;
use crate::config::ReloadHints;
use crate::runtime::{HostHandle, MuxedInput};

use super::*;

/// Hyperion service state
pub struct Service {
    /// Components host
    host: HostHandle,
    /// Debug listener
    debug_listener: Option<std::sync::mpsc::Sender<DebugMessage>>,
}

/// Type of the sender end of the channel for Hyperion inputs
pub type ServiceInputSender = mpsc::UnboundedSender<Input>;
/// Type of the receiver end of the channel for Hyperion inputs
pub type ServiceInputReceiver = mpsc::UnboundedReceiver<Input>;

impl Service {
    /// Create a new Service instance
    ///
    /// # Parameters
    ///
    /// * `host`: components host
    /// * `debug_listener`: channel to send debug updates to.
    pub fn new(
        host: HostHandle,
        debug_listener: Option<std::sync::mpsc::Sender<DebugMessage>>,
    ) -> Self {
        Self {
            host,
            debug_listener,
        }
    }

    /// Handle an incoming state update
    ///
    /// # Parameters
    ///
    /// * `update`: state update message
    fn handle_update(&self, update: StateUpdate) {
        // Forward state update to the debug listener if we have one
        if let Some(debug_listener) = self.debug_listener.as_ref() {
            debug_listener
                .send(DebugMessage::StateUpdate(update.clone()))
                .unwrap_or_else(|e| {
                    error!("failed to forward state update to listener: {:?}", e);
                });
        }

        let now = Instant::now();

        let mut devices = self.host.get_devices();

        match update {
            StateUpdate::Clear => {
                debug!("clearing all leds");
                devices.set_all_leds(now, color::ColorPoint::default(), false);
            }
            StateUpdate::SolidColor { color } => {
                debug!("setting all leds to {}", color);
                devices.set_all_leds(now, color, false);
            }
            StateUpdate::Image(raw_image) => {
                let (width, height) = raw_image.get_dimensions();
                debug!("incoming {}x{} image", width, height);

                let mut image_processor = self.host.get_image_processor();
                devices.set_from_image(now, &mut image_processor, raw_image, false);
            }
            StateUpdate::LedData(leds) => {
                debug!("setting {} leds from color data", leds.len());
                devices.set_leds(now, leds, false)
            }
        }
    }

    /// Handle an incoming service command
    ///
    /// # Parameters
    ///
    /// * `service_command`: command to handle
    fn handle_command(&self, service_command: ServiceCommand) {
        let mut devices = self.host.get_devices();

        match service_command {
            ServiceCommand::ReloadDevice {
                device_index,
                reload_hints,
            } => match devices.reload_device(device_index, reload_hints) {
                Ok(_) => {
                    if reload_hints.contains(ReloadHints::DEVICE_LEDS) {
                        // Reload the image processor if the LEDs changed
                        trace!("clearing the image processor cache");
                        *self.host.get_image_processor() = Default::default();
                    }
                }
                Err(error) => {
                    error!("error while reloading device: {}", error);
                }
            },
            ServiceCommand::EffectCompleted { name, result } => match result {
                Ok(_) => debug!("effect '{}' executed successfully", name),
                Err(e) => warn!("effect '{}' encountered an error: {}", name, e),
            },
        }
    }
}

impl Future for Service {
    type Item = ();
    type Error = tokio::timer::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Poll channel for state updates
        let mut priority_muxer = self.host.get_priority_muxer();
        // Unwrap because the priority muxer never fails (Error = ())
        while let Async::Ready(value) = priority_muxer.poll().unwrap() {
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
        try_ready!(self.host.get_devices().poll());

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
