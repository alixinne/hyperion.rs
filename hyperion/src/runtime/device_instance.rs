//! Definition of the DevinceInstance type

use std::time::{Duration, Instant};

use super::{IdleTracker, LedInstance};
use crate::color;
use crate::config::{self, ColorFormat};
use crate::filters::ColorFilter;
use crate::methods::{self, Method};

/// Type to hold computed device colors for a LED
pub struct LedData {
    /// Device led index
    pub index: usize,
    /// Color formatted according to the device format
    pub formatted: color::FormattedColor,
}

/// Runtime data for a given device
///
/// This type is constructed from the configuration details in the configuration file.
pub struct DeviceInstance {
    /// Device name
    name: String,
    /// Filter instance
    filter: ColorFilter,
    /// List of LED data
    leds: Vec<LedInstance>,
    /// Writing method
    method: Box<dyn Method + Send>,
    /// Device latency
    latency: Duration,
    /// Color format
    format: ColorFormat,
    /// Idle tracker
    idle_tracker: IdleTracker,
    /// Formatted LED data cache
    led_data: Vec<LedData>,
}

#[allow(missing_docs)]
mod errors {
    use error_chain::error_chain;

    error_chain! {
        types {
            DeviceError, DeviceErrorKind, ResultExt;
        }

        errors {
            OutOfBoundsLedIndex(i: usize) {
                description("out of bounds led index")
                display("no such LED at index {}", i)
            }
        }
    }
}

pub use errors::*;

impl DeviceInstance {
    /// Initialize a new device instance from a device configuration object
    ///
    /// # Parameters
    ///
    /// * `device`: device configuration node
    pub fn new(device: &config::Device) -> Self {
        let filter: ColorFilter = device.filter.clone().into();
        let capacity = filter.capacity(device.frequency as f32);

        let leds: Vec<_> = device
            .leds
            .iter()
            .map(|led| LedInstance::new(led.clone(), capacity))
            .collect();

        let led_count = leds.len();

        Self {
            name: device.name.clone(),
            filter,
            method: methods::from_endpoint(&device.endpoint),
            leds,
            latency: device.latency,
            format: device.format.clone(),
            idle_tracker: IdleTracker::new(device.idle.clone(), device.frequency),
            led_data: Vec::with_capacity(led_count),
        }
    }

    /// Get the LED instance details
    pub fn leds(&self) -> &Vec<LedInstance> {
        &self.leds
    }

    fn update_write(&mut self, time: Instant) {
        if self.idle_tracker.update_write(time) {
            debug!("activating device {}", self.name);
        }
    }

    /// Set all LEDs of this device to a new color
    ///
    /// # Parameters
    ///
    /// * `time`: time of the color update
    /// * `color`: new color to apply to all the LEDs of this device
    /// * `immediate`: apply change immediately (skipping filtering)
    pub fn set_all_leds(&mut self, time: Instant, color: color::ColorPoint, immediate: bool) {
        for led in self.leds.iter_mut() {
            // Change LED color
            led.update_color(time, color, immediate);
        }

        self.update_write(time);
    }

    /// Set a specific LED to the given color by its index
    ///
    /// # Parameters
    ///
    /// * `time`: time of the color update
    /// * `led_idx`: 0-based index of the LED to set
    /// * `color`: new color to apply immediately to all the LEDs of this device
    /// * `immediate`: apply change immediately (skipping filtering)
    pub fn set_led(
        &mut self,
        time: Instant,
        led_idx: usize,
        color: color::ColorPoint,
        immediate: bool,
    ) -> Result<(), DeviceError> {
        if led_idx >= self.leds.len() {
            return Err(DeviceErrorKind::OutOfBoundsLedIndex(led_idx).into());
        }

        let led = &mut self.leds[led_idx];

        // Change LED color
        led.update_color(time, color, immediate);

        self.update_write(time);

        Ok(())
    }

    /// Perform a writing pass to the target device
    ///
    /// This updates the filtered values using the current input values (set through calls to
    /// `set_led` and `set_all_leds`), then this initiates sending them to the physical device.
    ///
    /// # Parameters
    ///
    /// * `write_to_device`: true if the updated LED data should be written to the physical device.
    ///
    /// # Returns
    ///
    /// Future that completes when the write pass is over.
    pub async fn write(&mut self, write_to_device: bool) {
        // Get the current time
        let now = Instant::now();

        // The time when the color data will reach the device
        //
        // This is in the future since the packets (in case of UDP)
        // take some time to get to the device. This reduces the
        // latency introduced by the filter.
        let time = now + self.latency;

        // Update tracker
        let mut pass = self.idle_tracker.start_pass(&self.name);

        // Compute new colors
        for led in self.leds.iter_mut() {
            led.next_value(time, &self.filter, &mut pass);
        }

        // Format colors
        let format = &self.format;

        // Get buffer ownership
        let led_data = &mut self.led_data;

        // Clear current values
        led_data.clear();

        // Fill with new LED data
        led_data.extend(self.leds.iter().enumerate().map(|(index, led)| {
            let device_color = led.current_color().to_device(&format);
            let formatted = device_color.format(&format);

            LedData { index, formatted }
        }));

        if write_to_device {
            // Write to device and take back buffer
            pass.complete(self.method.write(led_data).await);
        }
    }

    /// Get the time at which the scheduler should write to this device again
    pub fn next_write(&self) -> Option<Instant> {
        self.idle_tracker.next_write()
    }
}
