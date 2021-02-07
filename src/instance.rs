use thiserror::Error;
use tokio::select;
use tokio::sync::broadcast;

use crate::{
    global::{Global, Message, MuxedMessage, MuxedMessageData},
    image::RawImage,
    models::{self, DeviceConfig, InstanceConfig},
};

mod device;
use device::*;

mod black_border_detector;
use black_border_detector::*;

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
    config: InstanceConfig,
    device: Device,
    receiver: broadcast::Receiver<MuxedMessage>,
    led_data: Vec<models::Color>,
    black_border_detector: BlackBorderDetector,
}

impl Instance {
    pub async fn new(global: Global, config: InstanceConfig) -> Result<Self, InstanceError> {
        let device = Device::new(&config.instance.friendly_name, config.device.clone()).await?;
        let led_count = config.device.hardware_led_count();
        let black_border_detector = BlackBorderDetector::new(config.black_border_detector.clone());

        Ok(Self {
            config,
            device,
            receiver: global.subscribe_muxed().await,
            led_data: vec![models::Color::default(); led_count],
            black_border_detector,
        })
    }

    fn handle_image(&mut self, image: &RawImage) -> Result<(), InstanceError> {
        // TODO: Do all the image processing needed

        // Update the black border
        self.black_border_detector.process(image);
        let black_border = self.black_border_detector.current_border();

        // Update all the leds according to their range
        // TODO: Fixed point arithmetic
        let ((xmin, xmax), (ymin, ymax)) = black_border.get_ranges(image.width(), image.height());
        let width = (xmax - xmin) as f32;
        let height = (ymax - ymin) as f32;
        for (spec, value) in self.config.leds.leds.iter().zip(self.led_data.iter_mut()) {
            let mut r_acc = 0u64;
            let mut r_cnt = 0u64;
            let mut g_acc = 0u64;
            let mut g_cnt = 0u64;
            let mut b_acc = 0u64;
            let mut b_cnt = 0u64;

            let lxmin = spec.hmin * width + xmin as f32;
            let lxmax = spec.hmax * width + xmin as f32;
            let lymin = spec.vmin * height + ymin as f32;
            let lymax = spec.vmax * height + ymin as f32;

            for y in lymin.floor() as u32..=(lymax.ceil() as u32).min(image.height() - 1) {
                for x in lxmin.floor() as u32..=(lxmax.ceil() as u32).min(image.width() - 1) {
                    if let Some(rgb) = image.color_at(x, y) {
                        let x_area = if (x as f32) < lxmin {
                            (255. * (1. - lxmin.fract())) as u64
                        } else if (x + 1) as f32 > lxmax {
                            (255. * lxmax.fract()) as u64
                        } else {
                            255
                        };

                        let y_area = if (y as f32) < lymin {
                            (255. * (1. - lymin.fract())) as u64
                        } else if (y + 1) as f32 > lymax {
                            (255. * lymax.fract()) as u64
                        } else {
                            255
                        };

                        let area = x_area * y_area / 255;

                        let (r, g, b) = rgb.into_components();
                        r_acc += (r as u64) * area;
                        r_cnt += area;
                        g_acc += (g as u64) * area;
                        g_cnt += area;
                        b_acc += (b as u64) * area;
                        b_cnt += area;
                    }
                }
            }

            *value = models::Color::from_components((
                (r_acc / r_cnt.max(1)).max(0).min(255) as u8,
                (g_acc / g_cnt.max(1)).max(0).min(255) as u8,
                (b_acc / b_cnt.max(1)).max(0).min(255) as u8,
            ));
        }

        Ok(())
    }

    async fn handle_message(&mut self, message: MuxedMessage) -> Result<(), InstanceError> {
        match message.data() {
            MuxedMessageData::SolidColor { color, .. } => {
                // TODO: Replace with fill once it's stabilized
                self.led_data.iter_mut().map(|x| *x = *color).count();
            }
            MuxedMessageData::Image { image, .. } => {
                self.handle_image(&*image)?;
            }
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
