//! Definition of the DevinceInstance type

use std::convert::TryFrom;
use std::time::{Duration, Instant};

use futures::{Async, Future, Poll, Stream};

use tokio::timer::Interval;

use crate::color;

use crate::methods;
use crate::methods::Method;

use crate::config::{DeviceConfigHandle, ReloadHints};

use super::{IdleTracker, LedInstance};
use crate::filters::ColorFilter;

/// Runtime data for a given device
///
/// This type is constructed from the configuration details in the configuration file.
pub struct DeviceInstance {
    /// Communication method
    method: Box<dyn Method + Send>,
    /// Updater future
    updater: Interval,
    /// List of LED data
    leds: Vec<LedInstance>,
    /// Change tracker for idle detection
    idle_tracker: IdleTracker,
    /// Filter instance
    filter: ColorFilter,
    /// Configuration handle
    config: DeviceConfigHandle,
}

/// Build an updater interval from a frequency
///
/// # Parameters
///
/// * `frequency`: update frequency in Hz
fn updater_for(frequency: f64) -> (Duration, Interval) {
    // Compute interval from frequency
    let update_duration = Duration::from_nanos((1_000_000_000f64 / frequency) as u64);
    (update_duration, Interval::new_interval(update_duration))
}

impl TryFrom<DeviceConfigHandle> for DeviceInstance {
    type Error = methods::MethodError;

    /// Try to instantiate the device corresponding to a specification
    ///
    /// # Parameters
    ///
    /// * `device`: device configuration to instantiate
    ///
    /// # Errors
    ///
    /// When the device method cannot be initialized from the configuration (for example, if the
    /// UDP address is already in use).
    fn try_from(config: DeviceConfigHandle) -> Result<Self, Self::Error> {
        let device = config.read().unwrap();

        let (update_duration, updater) = updater_for(device.frequency);

        // Log initialized device
        info!(
            "initialized device '{}': update {}, idle {}, {} leds",
            device.name,
            humantime::Duration::from(update_duration),
            device.idle,
            device.leds.len()
        );

        let filter = ColorFilter::from(device.filter.clone());
        let capacity = filter.capacity(device.frequency as f32);

        let method = methods::from_endpoint(&device.endpoint)?;

        let leds = device
            .leds
            .iter()
            .map(|led| LedInstance::new(led.clone(), capacity))
            .collect();

        let idle_tracker = IdleTracker::from(device.idle.clone());

        // device not used here, frees config from borrow
        drop(device);

        Ok(Self {
            method,
            updater,
            leds,
            idle_tracker,
            filter,
            config,
        })
    }
}

/// Device operation error type
#[derive(Debug, Fail)]
pub enum DeviceError {
    /// The LED index was greater than the total number of LED in the device
    #[fail(display = "no such LED at index {}", 0)]
    OutOfBoundsLedIndex(usize),
}

impl DeviceInstance {
    /// Iterate LEDs
    pub fn iter_leds(&self) -> impl Iterator<Item = (usize, &LedInstance)> {
        self.leds.iter().enumerate()
    }

    /// Get the LED count for this device
    pub fn get_led_count(&self) -> usize {
        self.leds.len()
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

        // Notify color change to tracker
        self.idle_tracker.notify_changed();
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
            return Err(DeviceError::OutOfBoundsLedIndex(led_idx));
        }

        let led = &mut self.leds[led_idx];

        // Change LED color
        led.update_color(time, color, immediate);

        // Notify color change to tracker
        self.idle_tracker.notify_changed();

        Ok(())
    }

    /// In case of a configuration update, reload cached settings
    /// from the configuration
    ///
    /// # Parameters
    ///
    /// * `reload_hints`: details about which parts to reload
    pub fn reload(&mut self, reload_hints: ReloadHints) -> Result<(), methods::MethodError> {
        let device = self.config.read().unwrap();

        if reload_hints.contains(ReloadHints::DEVICE_IDLE) {
            // Preserve idle tracker state on load
            self.idle_tracker.reload(device.idle.clone());
        }

        if reload_hints.contains(ReloadHints::DEVICE_FREQUENCY) {
            // Updater and filter are state-less
            let (_update_duration, updater) = updater_for(device.frequency);
            self.updater = updater;
        }

        if reload_hints.contains(ReloadHints::DEVICE_FILTER) {
            self.filter = ColorFilter::from(device.filter.clone());
        }

        if reload_hints.contains(ReloadHints::DEVICE_LEDS) {
            let capacity = self.filter.capacity(device.frequency as f32);

            if self.leds.len() == device.leds.len() {
                // Same amount of LEDs, preserve value states
                for (i, new_led) in device.leds.iter().enumerate() {
                    self.leds[i].reload(new_led.clone(), capacity);
                }
            } else {
                // LED count changed, just reload everything
                self.leds = device
                    .leds
                    .iter()
                    .map(|led| LedInstance::new(led.clone(), capacity))
                    .collect();
            }
        }

        if reload_hints.contains(ReloadHints::DEVICE_ENDPOINT) {
            self.method = methods::from_endpoint(&device.endpoint)?;
        }

        // Log initialized device
        info!("reloaded device '{}': {:?}", device.name, reload_hints);

        Ok(())
    }
}

impl Future for DeviceInstance {
    type Item = ();
    type Error = tokio::timer::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut write_device = false;

        let now = Instant::now();

        // Poll all events until NotReady
        while let Async::Ready(Some(_instant)) = self.updater.poll()? {
            write_device = true;
        }

        // Write device if needed
        if write_device && self.config.read().unwrap().enabled {
            // The interval told us to check the device, but now
            // check the change tracker to see if it's actually useful
            let (changed, state) = self.idle_tracker.update_state();

            let device = self.config.read().unwrap();

            // Notify log of state changes
            if changed {
                debug!("device '{}' is now {}", device.name, state);
            }

            // Write only if we need to
            if state.should_write() {
                self.idle_tracker.start_pass();
                self.method.write(
                    now,
                    &self.filter,
                    &mut self.leds[..],
                    &mut self.idle_tracker,
                    &device.format,
                );
                self.idle_tracker.end_pass();
            }
        }

        Ok(Async::NotReady)
    }
}
