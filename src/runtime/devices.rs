//! Definition of the Devices type

use std::time::Instant;

use futures::prelude::*;

use tokio::time::{delay_queue::Key, DelayQueue};

use crate::color;
use crate::config::*;
use crate::image::*;

use super::DeviceInstance;

/// A set of runtime devices
pub struct Devices<'conf> {
    /// Global configuration
    config: &'conf Config,
    /// List of device instances
    devices: Vec<DeviceInstance>,
    /// Device update DelayQueue
    dq: DelayQueue<usize>,
    /// DelayQueue keys for devices
    dq_keys: Vec<Option<Key>>,
}

impl<'conf> Devices<'conf> {
    /// Create a new runtime device host
    ///
    /// # Parameters
    ///
    /// * `config`: configuration for devices
    pub fn new(config: &'conf Config) -> Self {
        // Create device instances
        let devices: Vec<_> = config.devices.iter().map(DeviceInstance::new).collect();

        let mut dq = DelayQueue::new();
        let mut dq_keys = vec![None; devices.len()];

        // Insert devices in the DelayQueue
        for (idx, device) in devices.iter().enumerate() {
            if let Some(instant) = device.next_write() {
                dq_keys[idx] = Some(dq.insert_at(idx, instant.into()));
            } else {
                warn!(
                    "{}: device has no next_write, it will never be polled",
                    config.devices[idx].name
                );
            }
        }

        Self {
            config,
            devices,
            dq,
            dq_keys,
        }
    }

    fn update_all_delays(&mut self) {
        for (idx, _device) in self.devices.iter().enumerate() {
            if let Some(instant) = self.devices[idx].next_write() {
                // Sometimes next_write can be in the past, which would panic in the delay queue
                let instant = instant.max(Instant::now() + std::time::Duration::from_millis(1));

                if let Some(key) = &self.dq_keys[idx] {
                    self.dq.reset_at(key, instant.into());
                } else {
                    self.dq_keys[idx] = Some(self.dq.insert_at(idx, instant.into()));
                }
            } else {
                if let Some(key) = &self.dq_keys[idx] {
                    self.dq.remove(key);
                }

                self.dq_keys[idx] = None;
            }
        }
    }

    /// Update the next device in the queue
    pub async fn write_next(&mut self) {
        if self.dq.is_empty() {
            futures::future::pending::<()>().await;
        } else {
            // TODO: Really unwrap errors here?
            if let Some(idx) = self.dq.next().await.map(|res| res.unwrap().into_inner()) {
                // Invalidate key as soon as possible
                self.dq_keys[idx] = None;

                let device = &mut self.devices[idx];
                let device_config = &self.config.devices[idx];

                // Write device now
                // TODO: Write should be sent to the background using spawn?
                device.write(device_config.enabled).await;

                // Update key in DelayQueue
                if let Some(instant) = self.devices[idx].next_write() {
                    self.dq_keys[idx] = Some(self.dq.insert_at(idx, instant.into()));
                }
            }
        }
    }

    /// Set all LEDs of all devices to a new color immediately
    ///
    /// # Parameters
    ///
    /// * `time`: time of the color update
    /// * `color`: new color to apply immediately to all the LEDs of all devices
    /// * `immediate`: apply change immediately (skipping filtering)
    pub fn set_all_leds(&mut self, time: Instant, color: color::ColorPoint, immediate: bool) {
        for device in &mut self.devices {
            device.set_all_leds(time, color, immediate);
        }

        self.update_all_delays();
    }

    /// Update the devices using the given image processor and input image
    ///
    /// # Parameters
    ///
    /// * `time`: time of the color update
    /// * `processed`: processed RGB image
    /// * `immediate`: apply change immediately (skipping filtering)
    pub fn set_from_image(
        &mut self,
        time: Instant,
        processed_image: ProcessedImage,
        immediate: bool,
    ) {
        // Mutable reference to devices to prevent the closure exclusive access
        let devices = &mut self.devices;
        // Get reference to color config data
        let correction = &self.config.color;

        // Update LEDs with computed colors
        processed_image.update_leds(|(device_idx, led_idx), color| {
            // Should never fail, we only consider valid LEDs
            devices[device_idx]
                .set_led(time, led_idx, correction.process(color), immediate)
                .unwrap();
        });

        self.update_all_delays();
    }

    /// Set all LEDs of all devices to a new color immediately
    ///
    /// # Parameters
    ///
    /// * `time`: time of the color update
    /// * `leds`: color data for every device LED
    /// * `immediate`: apply change immediately (skipping filtering)
    pub fn set_leds(&mut self, time: Instant, leds: Vec<color::ColorPoint>, immediate: bool) {
        let mut current_idx = 0;

        for device in &mut self.devices {
            if current_idx >= leds.len() {
                warn!(
                    "not enough led data (only got {}, check led count)",
                    leds.len()
                );
                break;
            }

            for idx in 0..device.leds().len() {
                if current_idx >= leds.len() {
                    break;
                }

                device
                    .set_led(time, idx, leds[current_idx], immediate)
                    .unwrap();

                current_idx += 1;
            }
        }

        self.update_all_delays();
    }

    /// Get the total LED count for all devices
    pub fn get_led_count(&self) -> usize {
        self.devices
            .iter()
            .fold(0usize, |s, device| s + device.leds().len())
    }
}
