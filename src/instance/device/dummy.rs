use std::fmt::Write;

use async_trait::async_trait;

use super::{common::*, DeviceError};
use crate::models;

pub type DummyDevice = Rewriter<DummyDeviceImpl>;

pub struct DummyDeviceImpl {
    leds: Vec<models::Color>,
    mode: models::DummyDeviceMode,
    ansi_buf: String,
}

#[async_trait]
impl WritingDevice for DummyDeviceImpl {
    type Config = models::Dummy;

    fn new(config: &Self::Config) -> Result<Self, DeviceError> {
        Ok(Self {
            leds: vec![Default::default(); config.hardware_led_count as _],
            mode: config.mode,
            ansi_buf: String::new(),
        })
    }

    async fn set_let_data(
        &mut self,
        _config: &Self::Config,
        led_data: &[models::Color],
    ) -> Result<(), DeviceError> {
        self.leds.copy_from_slice(led_data);
        Ok(())
    }

    async fn write(&mut self) -> Result<(), DeviceError> {
        // Write to log when we get new data
        match self.mode {
            models::DummyDeviceMode::Text => {
                for (i, led) in self.leds.iter().enumerate() {
                    info!(
                        led = %format_args!("{:3}", i),
                        red = %format_args!("{:3}", led.red),
                        green = %format_args!("{:3}", led.green),
                        blue = %format_args!("{:3}", led.blue),
                    );
                }
            }

            models::DummyDeviceMode::Ansi => {
                // Build a truecolor ANSI sequence for all LEDs
                self.ansi_buf.clear();

                // Push LED colors
                for led in self.leds.iter() {
                    write!(
                        &mut self.ansi_buf,
                        "\x1B[38;2;{red};{green};{blue}m█",
                        red = led.red,
                        green = led.green,
                        blue = led.blue
                    )
                    .expect("failed to format escape sequence");
                }

                // Reset
                write!(&mut self.ansi_buf, "\x1B[0m").expect("failed to format escape sequence");

                // Output
                info!("{}", &self.ansi_buf);
            }
        }

        Ok(())
    }
}
