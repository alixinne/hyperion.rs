//! Definition of the Stdout method

use crate::methods::Method;
use crate::runtime::DeviceInstanceDataHandle;

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
    pub fn new(bits: i32) -> Self {
        Self { bits }
    }
}

impl Method for Stdout {
    fn write(&mut self, data: DeviceInstanceDataHandle) {
        data.write().unwrap().pass(|stats, leds| {
            // Print LED data
            for led in leds {
                let mut output = "LED".to_owned();
                output.push_str(&led.index.to_string());
                output.push_str(": [");

                for (idx, (comp, ch)) in led.formatted.into_iter().enumerate() {
                    output.push_str(&format!("{}: ", ch));
                    output.push_str(&((((1 << self.bits) - 1) as f32 * comp) as i32).to_string());
                    if idx < stats.components - 1 {
                        output.push_str(", ");
                    } else {
                        output.push_str("]");
                    }
                }

                info!("{}", output);
            }
        });
    }
}
