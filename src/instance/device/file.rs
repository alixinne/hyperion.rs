use async_trait::async_trait;
use chrono::Utc;
use std::{fmt::Write, time};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::models;

use super::{common::*, DeviceError};

pub type FileDevice = Rewriter<FileDeviceImpl>;

pub struct FileDeviceImpl {
    leds: Vec<models::Color>,
    print_timestamp: bool,
    file_handle: File,
    last_write_time: time::Instant,
    str_buf: String,
}

#[async_trait]
impl WritingDevice for FileDeviceImpl {
    type Config = models::File;

    fn new(config: &Self::Config) -> Result<Self, DeviceError> {
        let file_handle = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&config.output)?;

        Ok(Self {
            leds: vec![Default::default(); config.hardware_led_count as _],
            print_timestamp: config.print_time_stamp,
            file_handle: File::from_std(file_handle),
            last_write_time: time::Instant::now(),
            str_buf: String::new(),
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
        self.str_buf.clear();

        if self.print_timestamp {
            // Prepend timestamp
            let now = Utc::now();
            let elapsed_time_ms = self.last_write_time.elapsed().as_millis();
            self.last_write_time = time::Instant::now();

            write!(self.str_buf, "{} | +{}", now, elapsed_time_ms)?;
        }

        write!(self.str_buf, " [")?;
        for led in &self.leds {
            write!(self.str_buf, "{{{},{},{}}}", led.red, led.green, led.blue)?;
        }
        writeln!(self.str_buf, "]")?;

        self.file_handle.write(&self.str_buf.as_bytes()).await?;
        self.file_handle.flush().await?;

        Ok(())
    }
}
