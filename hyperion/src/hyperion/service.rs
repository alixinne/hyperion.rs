//! Definition of the Service type

use std::time::Instant;

use std::convert::TryFrom;

use futures::sync::mpsc;
use futures::{Async, Future, Poll, Stream};

use regex::Regex;

use crate::color;
use crate::config::Configuration;
use crate::image::Processor;
use crate::runtime::{Devices, PriorityMuxer};

use super::*;

/// Hyperion service state
pub struct Service {
    /// Priority muxer
    priority_muxer: PriorityMuxer,
    /// Device runtime data
    devices: Devices,
    /// Image processor
    image_processor: Processor,
    /// Debug listener
    debug_listener: Option<std::sync::mpsc::Sender<DebugMessage>>,
}

impl Service {
    /// Create a new Service instance
    ///
    /// # Parameters
    ///
    /// * `configuration`: configuration to derive this instance from
    /// * `disable_devices`: regular expression to match on device names. Matching devices will not
    ///   be instantiated from the configuration.
    /// * `debug_listener`: channel to send debug updates to.
    pub fn new(
        mut configuration: Configuration,
        disable_devices: Option<Regex>,
        debug_listener: Option<std::sync::mpsc::Sender<DebugMessage>>,
    ) -> Result<(Self, mpsc::UnboundedSender<Input>), HyperionError> {
        // TODO: check channel capacity
        let (sender, receiver) = mpsc::unbounded();

        // Sanitize configuration
        configuration.sanitize();

        let devices: Vec<_> = configuration
            .devices
            .into_iter()
            .filter(|device| {
                if let Some(rgx) = disable_devices.as_ref() {
                    if rgx.is_match(&device.name) {
                        info!("disabling device '{}'", device.name);
                        return false;
                    }
                }

                true
            })
            .collect();

        let correction = configuration.color;

        let devices = Devices::try_from((devices, correction)).map_err(HyperionError::from)?;

        let priority_muxer = PriorityMuxer::new(receiver);

        Ok((
            Self {
                priority_muxer,
                devices,
                image_processor: Default::default(),
                debug_listener,
            },
            sender,
        ))
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
        }
    }
}

impl Future for Service {
    type Item = ();
    type Error = HyperionError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Poll channel for state updates
        while let Async::Ready(value) = self.priority_muxer.poll()? {
            if let Some(state_update) = value {
                self.handle_update(state_update);
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
