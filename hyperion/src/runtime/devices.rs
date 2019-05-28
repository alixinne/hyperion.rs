//! Definition of the Devices type

use std::convert::TryFrom;

use futures::{Async, Future, Poll};

use crate::config::Device;
use crate::image::Processor;
use crate::methods;

use super::DeviceInstance;

pub struct Devices {
    devices: Vec<DeviceInstance>,
}

impl Devices {
    /// Set all LEDs of all devices to a new color immediately
    ///
    /// # Parameters
    ///
    /// * `color`: new color to apply immediately to all the LEDs of all devices
    pub fn set_all_leds(&mut self, color: palette::LinSrgb) {
        for device in self.devices.iter_mut() {
            device.set_all_leds(color);
        }
    }

    /// Update the devices using the given image processor and input image
    pub fn set_from_image(
        &mut self,
        image_processor: &mut Processor,
        data: Vec<u8>,
        width: u32,
        height: u32,
    ) {
        // Update stored image
        image_processor
            .with_devices(
                self.devices
                    .iter()
                    .enumerate()
                    .flat_map(|(device_idx, device)| {
                        device
                            .iter_leds()
                            .map(move |(led_idx, led)| (device_idx, led, led_idx))
                    }),
            )
            .process_image(&data[..], width, height);

        // Update LEDs with computed colors
        let devices_mut = &mut self.devices;
        for device in devices_mut.iter_mut() {
            device.start_pass();
        }

        image_processor.update_leds(|(device_idx, led_idx), color| {
            // Should never fail, we only consider valid LEDs
            devices_mut[device_idx].set_led(led_idx, color).unwrap();
        });

        for device in devices_mut.iter_mut() {
            device.end_pass(false);
        }
    }
}

// Note: can't use a blanket implementation for IntoIterator<Item = Device>
// See #50133
impl TryFrom<Vec<Device>> for Devices {
    // Can't use TryFrom<Device>::Error, see #38078
    type Error = methods::MethodError;

    fn try_from(devices: Vec<Device>) -> Result<Self, Self::Error> {
        Ok(Self {
            devices: devices
                .into_iter()
                .map(DeviceInstance::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl Future for Devices {
    type Item = ();
    type Error = tokio::timer::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Check intervals for devices to write to
        for device in self.devices.iter_mut() {
            try_ready!(device.poll());
        }

        Ok(Async::NotReady)
    }
}
