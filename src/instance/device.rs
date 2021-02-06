use async_trait::async_trait;
use thiserror::Error;

use crate::models::{self, DeviceConfig};

mod dummy;

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("device not supported: {0}")]
    NotSupported(&'static str),
    #[error("invalid led data")]
    InvalidLedData,
}

#[async_trait]
pub trait DeviceImpl: Send {
    async fn set_led_data(&mut self, led_data: &[models::Color]) -> Result<(), DeviceError>;
    async fn update(&mut self) -> Result<(), DeviceError>;
}

pub struct Device {
    inner: Box<dyn DeviceImpl>,
    led_data: Vec<models::Color>,
}

impl Device {
    pub async fn new(name: &str, config: models::Device) -> Result<Self, DeviceError> {
        let led_count = config.hardware_led_count();

        let inner = match config {
            models::Device::Dummy(dummy) => {
                Box::new(dummy::DummyDevice::new(name.to_owned(), dummy))
            }
            other => return Err(DeviceError::NotSupported(other.into())),
        };

        Ok(Self {
            inner,
            led_data: vec![Default::default(); led_count],
        })
    }

    pub async fn set_led_data(&mut self, led_data: &[models::Color]) -> Result<(), DeviceError> {
        if led_data.len() != self.led_data.len() {
            return Err(DeviceError::InvalidLedData);
        }

        // Store the LED data for updates
        self.led_data.copy_from_slice(led_data);

        // Notify device of new write: some devices write immediately
        self.inner.set_led_data(&self.led_data).await
    }

    pub async fn update(&mut self) -> Result<(), DeviceError> {
        Ok(self.inner.update().await?)
    }
}
