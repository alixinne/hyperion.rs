//! Definition of the Hyperion data model

use futures::sync::mpsc;
use futures::{Async, Future, Poll, Stream};

/// Definition of the Led type
mod led;
pub use led::*;

/// Definition of the Device type
mod device;
pub use device::*;

/// State update messages for the Hyperion service
#[derive(Debug)]
pub enum StateUpdate {
    ClearAll,
    SolidColor { color: palette::LinSrgb },
}

/// A configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    devices: Vec<Device>,
}

/// Hyperion service state
pub struct Hyperion {
    /// Configured state of LED devices
    configuration: Configuration,
    /// Receiver for update messages
    receiver: mpsc::UnboundedReceiver<StateUpdate>,
}

impl Hyperion {
    pub fn new(configuration: Configuration) -> (Self, mpsc::UnboundedSender<StateUpdate>) {
        // TODO: check channel capacity
        let (sender, receiver) = mpsc::unbounded();
        (
            Self {
                configuration,
                receiver,
            },
            sender,
        )
    }

    fn set_all_leds(&mut self, color: palette::LinSrgb) {
        for device in self.configuration.devices.iter_mut() {
            for led in device.leds.iter_mut() {
                led.current_color = color;
            }
        }
    }

    fn handle_update(&mut self, update: StateUpdate) {
        match update {
            StateUpdate::ClearAll => {
                debug!("clearing all leds");
                self.set_all_leds(palette::LinSrgb::default());
            }
            StateUpdate::SolidColor { color } => {
                debug!("setting all leds to {:?}", color);
                self.set_all_leds(color);
            }
        }
    }
}

#[derive(Debug, Fail)]
pub enum HyperionError {
    #[fail(display = "failed to receive update from channel")]
    ChannelReceiveFailed,
}

impl Future for Hyperion {
    type Item = ();
    type Error = HyperionError;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
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

        Ok(Async::NotReady)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn deserialize_full_config() {
        let config: Configuration = serde_json::from_str(
            r#"
{
    "devices": [
        {
            "name": "Stdout dummy",
            "endpoint": {
                "method": "stdout"
            },
            "leds": [ 
                { "index": 0, "hscan": { "minimum": 0.0, "maximum": 0.5 },
                              "vscan": { "minimum": 0.0, "maximum": 0.5 } }
            ]
        },
        {
            "name": "Remote UDP",
            "endpoint": {
                "method": "udp",
                "target": {
                    "address": "127.0.0.1:20446"
                }
            },
            "leds": [ 
                { "index": 0, "hscan": { "minimum": 0.5, "maximum": 1.0 },
                              "vscan": { "minimum": 0.5, "maximum": 1.0 } }
            ]
        }
    ]
}
        "#,
        )
        .unwrap();

        println!("{:?}", config);
    }
}
