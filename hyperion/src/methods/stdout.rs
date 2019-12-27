//! Definition of the Stdout method

use super::{Method, WriteResult};
use crate::runtime::LedData;

/// LED device that outputs RGB data to hyperion log
pub struct Stdout {
    /// Number of bits per output channel
    bits: i32,
    /// Device name
    name: String,
}

impl Stdout {
    /// Create a new Stdout device method
    ///
    /// # Parameters
    ///
    /// * `bits`: number of bits per output channel
    /// * `name`: output name
    pub fn new(bits: i32, name: String) -> Self {
        Self { bits, name }
    }
}

#[async_trait]
impl Method for Stdout {
    async fn write(&mut self, led_data: &Vec<LedData>) -> WriteResult {
        let bits = self.bits;
        let name = self.name.clone();

        tokio::task::block_in_place(|| {
            // Print LED data
            for led in led_data {
                let mut output = format!("{} LED{}: [", name, led.index);

                for (idx, (comp, ch)) in led.formatted.iter().enumerate() {
                    output.push_str(&format!("{}: ", ch));
                    output.push_str(&((((1 << bits) - 1) as f32 * comp) as i32).to_string());
                    if idx < led.formatted.components() - 1 {
                        output.push_str(", ");
                    } else {
                        output.push_str("]");
                    }
                }

                info!("{}", output);
            }
        });

        Ok(())
    }
}
