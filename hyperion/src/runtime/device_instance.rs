//! Definition of the DevinceInstance type

use std::convert::TryFrom;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use futures::{Async, Future, Poll, Stream};

use tokio::timer::Interval;

use crate::color;

use crate::methods;
use crate::methods::Method;

use crate::config::{DeviceConfigHandle, ReloadHints};

use super::{IdleTracker, LedInstance};
use crate::filters::ColorFilter;

/// Type to hold LED data properties
pub struct LedStats {
    /// Total number of leds for the current device
    pub led_count: usize,
    /// Number of formatted components per led
    pub components: usize,
}

/// Type to hold computed device colors for a LED
pub struct LedData {
    /// Device led index
    pub index: usize,
    /// Color formatted according to the device format
    pub formatted: color::FormattedColor,
}

/// LED and filter state data of a device instance
pub struct DeviceInstanceData {
    /// Configuration handle
    config: DeviceConfigHandle,
    /// Filter instance
    filter: ColorFilter,
    /// List of LED data
    leds: Vec<LedInstance>,
    /// Change tracker for idle detection
    idle_tracker: IdleTracker,
}

impl DeviceInstanceData {
    /// Create a new DeviceInstanceData
    ///
    /// # Parameters
    ///
    /// * `config`: configuration handle
    /// * `filter`: filter instance
    /// * `leds`: list of LED data
    /// * `idle_tracker`: change tracker for idle detection
    pub fn new(
        config: DeviceConfigHandle,
        filter: ColorFilter,
        leds: Vec<LedInstance>,
        idle_tracker: IdleTracker,
    ) -> Self {
        Self {
            config,
            filter,
            leds,
            idle_tracker,
        }
    }

    /// Get the device configuration
    pub fn get_config(
        &self,
    ) -> Result<
        crate::config::DeviceConfigGuard<'_>,
        std::sync::PoisonError<std::sync::RwLockReadGuard<'_, crate::config::Config>>,
    > {
        self.config.read()
    }

    /// Get the LED instance details
    pub fn leds(&self) -> &Vec<LedInstance> {
        &self.leds
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
            return Err(DeviceErrorKind::OutOfBoundsLedIndex(led_idx).into());
        }

        let led = &mut self.leds[led_idx];

        // Change LED color
        led.update_color(time, color, immediate);

        // Notify color change to tracker
        self.idle_tracker.notify_changed();

        Ok(())
    }

    /// In case of a configuration update, reload cached settings from the configuration
    ///
    /// # Parameters
    ///
    /// * `reload_hints`: details about which parts to reload
    fn reload(&mut self, reload_hints: ReloadHints) -> Result<(), methods::MethodError> {
        let device = self.config.read().unwrap();

        if reload_hints.contains(ReloadHints::DEVICE_IDLE) {
            // Preserve idle tracker state on load
            self.idle_tracker.reload(device.idle.clone());
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

        Ok(())
    }

    /// Perform a device writing pass
    ///
    /// # Parameters
    ///
    /// * `f`: function that receives (LedStats, Iterator<Item = LedData>) describing the number of
    /// leds and their components, as well as an iterator to computed device LED colors.
    pub fn pass<T, F: FnOnce(LedStats, &'_ mut dyn Iterator<Item = LedData>) -> T>(
        &mut self,
        f: F,
    ) -> Option<T> {
        let config = self.config.read().unwrap();

        if self.idle_tracker.start_pass(&config.name) {
            // Get the current time
            let time = Instant::now() + config.latency;

            // Compute new colors
            for led in self.leds.iter_mut() {
                led.next_value(time, &self.filter, &mut self.idle_tracker);
            }

            // Invoke callback
            let result = f(
                LedStats {
                    led_count: self.leds.len(),
                    components: config.format.components(),
                },
                &mut self.leds.iter_mut().enumerate().map(|(index, led)| {
                    let device_color = led.current_color().to_device(&config.format);
                    let formatted = device_color.format(&config.format);

                    LedData { index, formatted }
                }),
            );

            self.idle_tracker.end_pass();
            Some(result)
        } else {
            None
        }
    }
}

/// Handle to device instance data
pub type DeviceInstanceDataHandle = Arc<RwLock<DeviceInstanceData>>;

/// Runtime data for a given device
///
/// This type is constructed from the configuration details in the configuration file.
pub struct DeviceInstance {
    /// Configuration handle
    config: DeviceConfigHandle,
    /// Communication method
    method: Box<dyn Method + Send>,
    /// Updater future
    updater: Interval,
    /// Device data
    data: DeviceInstanceDataHandle,
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

        // Device data
        let data = Arc::new(RwLock::new(DeviceInstanceData::new(
            config.clone(),
            filter,
            leds,
            idle_tracker,
        )));

        Ok(Self {
            config,
            method,
            updater,
            data,
        })
    }
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
    /// Get reference to device data
    pub fn get_data(&self) -> DeviceInstanceDataHandle {
        self.data.clone()
    }

    /// Set all LEDs of this device to a new color
    ///
    /// # Parameters
    ///
    /// * `time`: time of the color update
    /// * `color`: new color to apply to all the LEDs of this device
    /// * `immediate`: apply change immediately (skipping filtering)
    pub fn set_all_leds(&mut self, time: Instant, color: color::ColorPoint, immediate: bool) {
        self.data
            .write()
            .unwrap()
            .set_all_leds(time, color, immediate)
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
        self.data
            .write()
            .unwrap()
            .set_led(time, led_idx, color, immediate)
    }

    /// In case of a configuration update, reload cached settings from the configuration
    ///
    /// # Parameters
    ///
    /// * `reload_hints`: details about which parts to reload
    pub fn reload(&mut self, reload_hints: ReloadHints) -> Result<(), methods::MethodError> {
        let device = self.config.read().unwrap();

        if reload_hints.contains(ReloadHints::DEVICE_FREQUENCY) {
            // Updater and filter are state-less
            let (_update_duration, updater) = updater_for(device.frequency);
            self.updater = updater;
        }

        self.data.write().unwrap().reload(reload_hints)?;

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

        // Poll all events until NotReady
        while let Async::Ready(Some(_instant)) = self.updater.poll()? {
            write_device = true;
        }

        // Read config
        let device = self.config.read().unwrap();

        // Write device if needed
        if write_device && device.enabled {
            // We don't need the device config anymore
            drop(device);

            // Write to the device using the current data
            self.method.write(self.data.clone());
        }

        Ok(Async::NotReady)
    }
}
