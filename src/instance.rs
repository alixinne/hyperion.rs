use thiserror::Error;
use tokio::select;
use tokio::sync::broadcast;

use crate::{
    global::{Global, Message, MuxedMessage, MuxedMessageData},
    models::{self, DeviceConfig, InstanceConfig},
};

mod device;
use device::*;

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("device error: {0}")]
    Device(#[from] DeviceError),
    #[error("recv error: {0}")]
    Recv(#[from] broadcast::error::RecvError),
    #[error("command not supported: {0:?}")]
    NotSupported(MuxedMessageData),
}

pub struct Instance {
    device: Device,
    receiver: broadcast::Receiver<MuxedMessage>,
    led_data: Vec<models::Color>,
}

impl Instance {
    pub async fn new(global: Global, config: InstanceConfig) -> Result<Self, InstanceError> {
        let device = Device::new(&config.instance.friendly_name, config.device.clone()).await?;
        let led_count = config.device.hardware_led_count();

        Ok(Self {
            device,
            receiver: global.subscribe_muxed().await,
            led_data: vec![models::Color::default(); led_count],
        })
    }

    async fn handle_message(&mut self, message: MuxedMessage) -> Result<(), InstanceError> {
        // TODO: Do all the image processing needed
        // TODO: Handle image updates

        match message.data() {
            MuxedMessageData::SolidColor { color, .. } => {
                // TODO: Replace with fill once it's stabilized
                self.led_data.iter_mut().map(|x| *x = *color).count();
            }
            other => return Err(InstanceError::NotSupported(other.clone())),
        }

        // The message was handled, notify the device
        self.device.set_led_data(&self.led_data).await?;

        Ok(())
    }

    pub async fn run(mut self) -> Result<(), InstanceError> {
        loop {
            select! {
                _ = self.device.update() => {
                    // Device update completed
                },
                message = self.receiver.recv() => {
                    self.handle_message(message?).await?;
                }
            }
        }
    }
}
