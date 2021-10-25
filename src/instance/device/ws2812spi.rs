use async_trait::async_trait;
use spidev::{SpiModeFlags, Spidev, SpidevOptions, SpidevTransfer};

use super::{common::*, DeviceError};
use crate::models;

pub type Ws2812SpiDevice = Rewriter<Ws2812SpiImpl>;

pub struct Ws2812SpiImpl {
    dev: ImplState,
    notified_error: bool,
    buf: Vec<u8>,
}

const SPI_BYTES_PER_LED: usize = 3 * SPI_BYTES_PER_COLOUR;
const SPI_BYTES_PER_COLOUR: usize = 4;
const SPI_FRAME_END_LATCH_BYTES: usize = 116;
const BITPAIR_TO_BYTE: [u8; 4] = [0b10001000, 0b10001100, 0b11001000, 0b11001100];

enum ImplState {
    Pending(models::Ws2812Spi),
    Ready(Spidev),
}

impl ImplState {
    fn as_dev(&self) -> Option<&Spidev> {
        match self {
            ImplState::Ready(dev) => Some(dev),
            _ => None,
        }
    }

    fn try_init(&mut self) -> Result<&Spidev, DeviceError> {
        match self {
            ImplState::Pending(config) => {
                // Initialize SPI device
                let mut dev = Spidev::open(&config.output)?;
                let options = SpidevOptions::new()
                    .bits_per_word(8)
                    .max_speed_hz(config.rate as _)
                    .mode(SpiModeFlags::SPI_MODE_0)
                    .build();
                dev.configure(&options)?;

                info!(path = %config.output, "initialized SPI device");

                *self = Self::from(dev);
                Ok(self.as_dev().unwrap())
            }

            ImplState::Ready(dev) => Ok(dev),
        }
    }
}

impl From<&models::Ws2812Spi> for ImplState {
    fn from(value: &models::Ws2812Spi) -> Self {
        Self::Pending(value.clone())
    }
}

impl From<Spidev> for ImplState {
    fn from(value: Spidev) -> Self {
        Self::Ready(value)
    }
}

#[async_trait]
impl WritingDevice for Ws2812SpiImpl {
    type Config = models::Ws2812Spi;

    fn new(config: &models::Ws2812Spi) -> Result<Self, DeviceError> {
        // Buffer for SPI tranfers
        let buf = vec![
            0;
            config.hardware_led_count as usize * SPI_BYTES_PER_LED
                + SPI_FRAME_END_LATCH_BYTES
        ];

        let mut dev = ImplState::from(config);

        // Try to open the device early
        if let Err(error) = dev.try_init() {
            warn!(%error, path = %config.output, "failed to initialize SPI device, will try again later");
        }

        Ok(Self {
            dev,
            notified_error: false,
            buf,
        })
    }

    async fn set_let_data(
        &mut self,
        config: &Self::Config,
        led_data: &[models::Color],
    ) -> Result<(), DeviceError> {
        // Update buffer
        let mut ptr = 0;
        for led in led_data {
            let (r, g, b) = config.color_order.reorder_from_rgb(*led).into_components();
            let mut color_bits = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);

            for j in (0..SPI_BYTES_PER_LED).rev() {
                self.buf[ptr + j] = BITPAIR_TO_BYTE[(color_bits & 0x3) as usize];
                color_bits >>= 2;
            }

            ptr += SPI_BYTES_PER_LED;
        }

        for dst in self.buf.iter_mut().skip(ptr) {
            *dst = 0;
        }

        if config.invert {
            for byte in &mut self.buf {
                *byte = !*byte;
            }
        }

        Ok(())
    }

    async fn write(&mut self) -> Result<(), DeviceError> {
        // Perform SPI transfer
        let mut transfer = SpidevTransfer::write(&self.buf);

        // Try writing to the device
        match self.dev.try_init() {
            Ok(dev) => {
                self.notified_error = false;
                dev.transfer(&mut transfer)?;
            }
            Err(err) => {
                if !self.notified_error {
                    self.notified_error = true;
                    error!(error = %err, "failed to initialize SPI device");
                }
            }
        }

        Ok(())
    }
}
