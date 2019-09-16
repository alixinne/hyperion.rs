//! Definition of the Stdout method

use std::time::Instant;

use std::io::Result;

use crate::config::ColorFormat;
use crate::filters::ColorFilter;
use crate::methods::Method;
use crate::runtime::{IdleTracker, LedInstance};

/// LED device that outputs RGB data to hyperion log
pub struct Stdout {
    /// Number of bits per output channel
    bits: i32,
}

impl Stdout {
    /// Create a new Stdout device method
    ///
    /// # Parameters
    ///
    /// * `bits`: number of bits per output channel
    pub fn new(bits: i32) -> Result<Self> {
        Ok(Self { bits })
    }
}

impl Method for Stdout {
    fn write(
        &mut self,
        time: Instant,
        filter: &ColorFilter,
        leds: &mut [LedInstance],
        idle_tracker: &mut IdleTracker,
        format: &ColorFormat,
    ) {
        // Number of components per LED
        let components = format.components();

        // Print LED data
        for (i, led) in leds.iter_mut().enumerate() {
            let current_color = led.next_value(time, &filter, idle_tracker);
            let device_color = current_color.to_device(format);
            let formatted = device_color.format(format);

            let mut output = "LED".to_owned();
            output.push_str(&i.to_string());
            output.push_str(": [");

            for (idx, comp) in formatted.into_iter().enumerate() {
                output.push_str(&((((1 << self.bits) - 1) as f32 * comp) as i32).to_string());
                if idx < components - 1 {
                    output.push_str(", ");
                } else {
                    output.push_str("]");
                }
            }

            info!("{}", output);
        }
    }
}
