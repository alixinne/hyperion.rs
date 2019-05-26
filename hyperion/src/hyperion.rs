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

use crate::image::Processor;

/// State update messages for the Hyperion service
#[derive(Debug, Clone)]
pub enum StateUpdate {
    ClearAll,
    SolidColor {
        color: palette::LinSrgb,
    },
    Image {
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
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
        // Clamp frequency to 1/hour Hz
        let mut freq = 1.0f64 / 3600f64;
        if device.frequency > freq {
            freq = device.frequency;
        }

        Ok(DeviceInstance {
            method: methods::from_endpoint(&device.endpoint)?,
            updater: Interval::new_interval(Duration::from_nanos((1_000_000_000f64 / freq) as u64)),
            leds: device
                .leds
                .iter()
                .map(|led| LedInstance::new(led))
                .collect(),
        })
    }
}

/// Messages sent to the debug monitor
pub enum DebugMessage {
    /// A state update forwarded from one of the sources
    StateUpdate(StateUpdate),
    /// The hyperion instance is terminating
    Terminating,
}

/// Hyperion service state
pub struct Hyperion {
    /// Receiver for update messages
    receiver: mpsc::UnboundedReceiver<StateUpdate>,
    /// Device runtime data
    devices: Vec<DeviceInstance>,
    /// Image processor
    image_processor: Processor,
    /// Debug listener
    debug_listener: Option<std::sync::mpsc::Sender<DebugMessage>>,
}

impl Hyperion {
    pub fn new(
        configuration: Configuration,
        disable_devices: Option<Regex>,
        debug_listener: Option<std::sync::mpsc::Sender<DebugMessage>>,
    ) -> Result<(Self, mpsc::UnboundedSender<StateUpdate>), HyperionError> {
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
                image_processor: Default::default(),
                debug_listener,
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
        // Forward state update to the debug listener if we have one
        if let Some(debug_listener) = self.debug_listener.as_ref() {
            debug_listener
                .send(DebugMessage::StateUpdate(update.clone()))
                .unwrap_or_else(|e| {
                    error!("failed to forward state update to listener: {:?}", e);
                    self.debug_listener = None;
                });
        }

        match update {
            StateUpdate::ClearAll => {
                debug!("clearing all leds");
                self.set_all_leds(palette::LinSrgb::default());
            }
            StateUpdate::SolidColor { color } => {
                debug!("setting all leds to {:?}", color);
                self.set_all_leds(color);
            }
            StateUpdate::Image {
                data,
                width,
                height,
            } => {
                debug!("incoming {}x{} image", width, height);

                // Update stored image
                self.image_processor
                    .with_devices(self.devices.iter().enumerate().flat_map(
                        |(device_idx, device)| {
                            device
                                .leds
                                .iter()
                                .enumerate()
                                .map(move |(led_idx, led)| (device_idx, led, led_idx))
                        },
                    ))
                    .process_image(&data[..], width, height);

                // Update LEDs with computed colors
                let devices_mut = &mut self.devices;
                self.image_processor
                    .update_leds(|(device_idx, led_idx), color| {
                        devices_mut[device_idx].leds[led_idx].current_color = color;
                    });
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
      method: stdout
      target: {}
    leds:
      - hscan: { min: 0.0, max: 0.5 }
        vscan: { min: 0.0, max: 0.5 }
  - name: Remote UDP
    endpoint:
      method: udp
      target:
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
