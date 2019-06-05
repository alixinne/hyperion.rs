//! Definition of the Hyperion data model

use std::time::Instant;

use std::convert::TryFrom;

use futures::sync::mpsc;
use futures::{Async, Future, Poll, Stream};

use regex::Regex;

use crate::color;
use crate::config::Configuration;
use crate::image::Processor;
use crate::runtime::Devices;

mod debug_message;
pub use debug_message::*;

mod hyperion_error;
pub use hyperion_error::*;

mod state_update;
pub use state_update::*;

/// Hyperion service state
pub struct Hyperion {
    /// Receiver for update messages
    receiver: mpsc::UnboundedReceiver<StateUpdate>,
    /// Device runtime data
    devices: Devices,
    /// Image processor
    image_processor: Processor,
    /// Debug listener
    debug_listener: Option<std::sync::mpsc::Sender<DebugMessage>>,
}

impl Hyperion {
    /// Create a new Hyperion instance
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
    ) -> Result<(Self, mpsc::UnboundedSender<StateUpdate>), HyperionError> {
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

        let devices = Devices::try_from(devices).map_err(HyperionError::from)?;

        Ok((
            Self {
                receiver,
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
            StateUpdate::ClearAll => {
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

impl Future for Hyperion {
    type Item = ();
    type Error = HyperionError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Poll channel for state updates
        while let Async::Ready(value) = self
            .receiver
            .poll()
            .map_err(|_| HyperionError::ChannelReceiveFailed)?
        {
            if let Some(state_update) = value {
                trace!("got state update: {:?}", state_update);
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

impl Drop for Hyperion {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn deserialize_full_config() {
        let config: Configuration = serde_yaml::from_str(
            r#"
devices:
  - name: Stdout dummy
    endpoint:
      type: stdout
    leds:
      - hscan: { min: 0.0, max: 0.5 }
        vscan: { min: 0.0, max: 0.5 }
  - name: Remote UDP
    endpoint:
      type: udp
      address: 127.0.0.1:20446
    leds:
      - hscan: { min: 0.5, max: 1.0 }
        vscan: { min: 0.5, max: 1.0 }
        "#,
        )
        .unwrap();

        println!("{:?}", config);
    }
}
