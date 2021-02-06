use async_trait::async_trait;

use super::{DeviceError, DeviceImpl};
use crate::models;

pub struct DummyDevice {
    name: String,
    led_count: usize,
}

impl DummyDevice {
    pub fn new(name: String, config: models::Dummy) -> Self {
        Self {
            name,
            led_count: config.hardware_led_count as _,
        }
    }
}

#[async_trait]
impl DeviceImpl for DummyDevice {
    async fn set_led_data(&mut self, led_data: &[models::Color]) -> Result<(), DeviceError> {
        if led_data.len() != self.led_count {
            return Err(DeviceError::InvalidLedData);
        }

        // Write to log when we get new data
        for (i, led) in led_data.iter().enumerate() {
            info!(
                "{}: LED {}: {:3}, {:3}, {:3}",
                self.name, i, led.red, led.green, led.blue
            );
        }

        Ok(())
    }

    async fn update(&mut self) -> Result<(), super::DeviceError> {
        // No regular updates
        Ok(())
    }
}
