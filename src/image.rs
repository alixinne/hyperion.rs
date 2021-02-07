use std::convert::TryFrom;

use thiserror::Error;

use crate::models::Color;

#[derive(Debug, Clone, Error)]
pub enum RawImageError {
    #[error("invalid data ({data} bytes) for the given dimensions ({width} x {height} x {channels} = {expected})")]
    InvalidData {
        data: usize,
        width: u32,
        height: u32,
        channels: u32,
        expected: usize,
    },
}

#[derive(Clone)]
pub struct RawImage {
    data: Vec<u8>,
    width: u32,
    height: u32,
    channels: u32,
}

impl RawImage {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn channels(&self) -> u32 {
        self.channels
    }

    pub fn color_at(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height && self.channels >= 3 {
            unsafe {
                Some(Color::from_components((
                    *self
                        .data
                        .get_unchecked(((y * self.width + x) * self.channels) as usize),
                    *self
                        .data
                        .get_unchecked(((y * self.width + x) * self.channels + 1) as usize),
                    *self
                        .data
                        .get_unchecked(((y * self.width + x) * self.channels + 2) as usize),
                )))
            }
        } else {
            None
        }
    }
}

impl std::fmt::Debug for RawImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("RawImage");
        f.field("width", &self.width);
        f.field("height", &self.height);
        f.field("channels", &self.channels);

        if self.data.len() > 32 {
            f.field("data", &format!("[{} bytes]", self.data.len()));
        } else {
            f.field("data", &self.data);
        }

        f.finish()
    }
}

impl TryFrom<(Vec<u8>, u32, u32)> for RawImage {
    type Error = RawImageError;

    fn try_from((data, width, height): (Vec<u8>, u32, u32)) -> Result<Self, Self::Error> {
        let channels = 3;
        let expected = (width * height * channels) as usize;

        if data.len() != expected {
            return Err(RawImageError::InvalidData {
                data: data.len(),
                width,
                height,
                channels,
                expected,
            });
        }

        Ok(Self {
            data,
            width,
            height,
            channels,
        })
    }
}
