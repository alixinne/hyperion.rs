//! RawImage type definition

use std::convert::TryFrom;

/// Represents a raw 8-bit linear RGB image
#[derive(Debug, Clone)]
pub struct RawImage {
    /// Raw 8-bit RGB data, as ABGR tuples
    data: Vec<u32>,
    /// Width of the image in `data`
    width: usize,
    /// Height of the image in `data`
    height: usize,
}

#[allow(missing_docs)]
mod errors {
    use error_chain::error_chain;

    error_chain! {
        types {
            RawImageError, RawImageErrorKind, ResultExt;
        }

        errors {
            InvalidDimensions {
                description("invalid dimensions")
            }
        }
    }
}

pub use errors::*;

impl RawImage {
    /// Get the width and height of the image
    pub fn get_dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Get a pixel at a given location
    ///
    /// # Parameters
    ///
    /// * `x`: horizontal coordinate of the pixel
    /// * `y`: vertical coordinate of the pixel
    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        assert!(x < self.width);
        assert!(y < self.height);

        let idx = y * self.width + x;
        let val = self.data[idx];
        (
            (val & 0xFFu32) as u8,
            (val & 0xFF00u32 >> 8) as u8,
            (val & 0xFF0000u32 >> 16) as u8,
        )
    }
}

impl TryFrom<(Vec<u8>, u32, u32)> for RawImage {
    type Error = RawImageError;

    fn try_from(value: (Vec<u8>, u32, u32)) -> Result<Self, Self::Error> {
        let rgb_data = value.0;
        let width = value.1 as usize;
        let height = value.2 as usize;

        if rgb_data.len() != width * height * 3 {
            return Err(RawImageErrorKind::InvalidDimensions.into());
        }

        if width == 0 || height == 0 {
            return Err(RawImageErrorKind::InvalidDimensions.into());
        }

        // u32 vector
        let mut data: Vec<u32> = Vec::with_capacity((width * height) as usize);

        // Make tuples
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 3usize;
                data.push(
                    (rgb_data[idx] as u32)
                        | (rgb_data[idx] as u32) << 8
                        | (rgb_data[idx] as u32) << 16,
                );
            }
        }

        Ok(RawImage {
            data,
            width,
            height,
        })
    }
}
