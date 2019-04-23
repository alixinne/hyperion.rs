//! Definition of the Hyperion data model

use std::time::Duration;

use tokio::timer::Interval;

use futures::sync::mpsc;
use futures::{Async, Future, Poll, Stream};

use regex::Regex;

/// Definition of the Led type
mod led;
pub use led::*;

/// Definition of the Device type
mod device;
pub use device::*;

use crate::methods;
use crate::methods::Method;

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

/// Runtime data for a given device
///
/// This type is constructed from the configuration details in the config file.
struct DeviceInstance {
    /// Communication method
    method: Box<dyn Method + Send>,
    /// Updater future
    updater: Interval,
    /// List of LED data
    leds: Vec<LedInstance>,
}

impl DeviceInstance {
    fn from_device(device: &Device) -> Result<DeviceInstance, methods::MethodError> {
        Ok(DeviceInstance {
            method: methods::from_endpoint(&device.endpoint)?,
            updater: Interval::new_interval(Duration::from_nanos(1_000_000_000u64 / std::cmp::max(1u64, device.frequency as u64))),
            leds: device.leds.iter().map(|led| LedInstance::new(led)).collect()
        })
    }
}

/// Hyperion service state
pub struct Hyperion {
    /// Receiver for update messages
    receiver: mpsc::UnboundedReceiver<StateUpdate>,
    /// Device runtime data
    devices: Vec<DeviceInstance>,
}

impl Hyperion {
    pub fn new(configuration: Configuration, disable_devices: Option<Regex>) -> Result<(Self, mpsc::UnboundedSender<StateUpdate>), HyperionError> {
        // TODO: check channel capacity
        let (sender, receiver) = mpsc::unbounded();

        let devices = configuration
            .devices
            .iter()
            .filter(|device| {
                if let Some(rgx) = disable_devices.as_ref() {
                    if rgx.is_match(&device.name) {
                        info!("disabling device '{}'", device.name);
                        return false;
                    }
                }

                true
            })
            .map(DeviceInstance::from_device)
            .collect::<Result<Vec<_>, _>>()
            .map_err(HyperionError::from)?;

        Ok((
            Self {
                receiver,
                devices,
            },
            sender,
        ))
    }

    fn set_all_leds(&mut self, color: palette::LinSrgb) {
        for device in self.devices.iter_mut() {
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
    #[fail(display = "failed to poll the updater interval")]
    UpdaterPollFailed,
    #[fail(display = "failed to initialize LED devices: {}", error)]
    LedDeviceInitFailed { error: methods::MethodError },
}

impl From<methods::MethodError> for HyperionError {
    fn from(error: methods::MethodError) -> HyperionError {
        HyperionError::LedDeviceInitFailed { error }
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

        // Check intervals for devices to write to
        for device in self.devices.iter_mut() {
            let mut write_device = false;

            // Poll all events until NotReady
            while let Async::Ready(Some(_instant)) = device
                .updater
                .poll()
                .map_err(|_| HyperionError::UpdaterPollFailed)?
            {
                write_device = true;
            }

            // Write device if needed
            if write_device {
                device.method.write(&device.leds[..]);
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
