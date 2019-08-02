//! Definition of the Devices type

use std::time::Instant;

use std::convert::TryFrom;

use futures::{Async, Future, Poll};

use crate::color;
use crate::config::{Correction, Device};
use crate::image::*;
use crate::methods;

use super::DeviceInstance;

/// A set of runtime devices
pub struct Devices {
    /// List of device instances
    devices: Vec<DeviceInstance>,
    /// Image color correction
    correction: Correction,
}

impl Devices {
    /// Set all LEDs of all devices to a new color immediately
    ///
    /// # Parameters
    ///
    /// * `time`: time of the color update
    /// * `color`: new color to apply immediately to all the LEDs of all devices
    /// * `immediate`: apply change immediately (skipping filtering)
    pub fn set_all_leds(&mut self, time: Instant, color: color::ColorPoint, immediate: bool) {
        for device in self.devices.iter_mut() {
            device.set_all_leds(time, color, immediate);
        }
    }

    /// Update the devices using the given image processor and input image
    ///
    /// # Parameters
    ///
    /// * `time`: time of the color update
    /// * `image_processor`: image processor instance
    /// * `raw_image`: raw RGB image
    /// * `immediate`: apply change immediately (skipping filtering)
    pub fn set_from_image(
        &mut self,
        time: Instant,
        image_processor: &mut Processor,
        raw_image: RawImage,
        immediate: bool,
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
            .process_image(raw_image);

        // Update LEDs with computed colors
        image_processor.update_leds(|(device_idx, led_idx), color| {
            // Should never fail, we only consider valid LEDs
            self.devices[device_idx]
                .set_led(time, led_idx, self.correction.process(color), immediate)
                .unwrap();
        });
    }
}

// Note: can't use a blanket implementation for IntoIterator<Item = Device>
// See #50133
impl TryFrom<(Vec<Device>, Correction)> for Devices {
    // Can't use TryFrom<Device>::Error, see #38078
    type Error = methods::MethodError;

    fn try_from((devices, correction): (Vec<Device>, Correction)) -> Result<Self, Self::Error> {
        Ok(Self {
            devices: devices
                .into_iter()
                .map(DeviceInstance::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            correction,
        })
    }
}

impl Future for Devices {
    type Item = ();
    type Error = tokio::timer::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Check intervals for devices to write to
        for device in self.devices.iter_mut() {
            while let Async::Ready(()) = device.poll()? {}
        }

        Ok(Async::NotReady)
    }
}
