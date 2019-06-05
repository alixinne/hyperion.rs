//! RawImage type definition

use std::convert::TryFrom;

/// Represents a raw 8-bit linear RGB image
#[derive(Debug, Clone)]
pub struct RawImage {
    /// Raw 8-bit RGB data
    data: Vec<u8>,
    /// Width of the image in `data`
    width: u32,
    /// Height of the image in `data`
    height: u32,
}

/// Raw image data error
#[derive(Debug, Fail)]
pub enum RawImageError {
    /// Invalid width and height for given buffer
    #[fail(display = "invalid dimensions")]
    InvalidDimensions,
}

impl RawImage {
    /// Get the width and height of the image
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get a pixel at a given location
    ///
    /// # Parameters
    ///
    /// * `x`: horizontal coordinate of the pixel
    /// * `y`: vertical coordinate of the pixel
    pub fn get_pixel(&self, x: u32, y: u32) -> (u8, u8, u8) {
        assert!(x < self.width);
        assert!(y < self.height);

        let idx = ((y * self.width + x) * 3) as usize;
        (self.data[idx], self.data[idx + 1], self.data[idx + 2])
    }

    /// Return the raw components of the image
    pub fn into_raw(self) -> (Vec<u8>, u32, u32) {
        (self.data, self.width, self.height)
    }
}

impl TryFrom<(Vec<u8>, u32, u32)> for RawImage {
    type Error = RawImageError;

    fn try_from(value: (Vec<u8>, u32, u32)) -> Result<Self, Self::Error> {
        if value.0.len() != (value.1 * value.2 * 3) as usize {
            return Err(RawImageError::InvalidDimensions);
        }

        if value.1 == 0 || value.2 == 0 {
            return Err(RawImageError::InvalidDimensions);
        }

        Ok(RawImage {
            data: value.0,
            width: value.1,
            height: value.2,
        })
    }
}
