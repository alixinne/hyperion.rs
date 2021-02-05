use std::convert::TryFrom;

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum RawImageError {
    #[error("invalid data ({data} bytes) for the given dimensions ({width} x {height} x {channels} = {expected})")]
    InvalidData {
        data: usize,
        width: usize,
        height: usize,
        channels: usize,
        expected: usize,
    },
}

#[derive(Debug, Clone)]
pub struct RawImage {
    data: Vec<u8>,
    width: usize,
    height: usize,
    channels: usize,
}

impl TryFrom<(Vec<u8>, u32, u32)> for RawImage {
    type Error = RawImageError;

    fn try_from((data, width, height): (Vec<u8>, u32, u32)) -> Result<Self, Self::Error> {
        let channels = 3;
        let width = width as usize;
        let height = height as usize;
        let expected = width * height * channels;

        if data.len() != channels * width * height {
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
