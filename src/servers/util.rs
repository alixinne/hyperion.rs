use thiserror::Error;

use crate::{global::InputMessage, image::RawImageError};

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("invalid width")]
    InvalidWidth,
    #[error("invalid height")]
    InvalidHeight,
    #[error("image error: {0}")]
    Image(#[from] RawImageError),
    #[error("error broadcasting update: {0}")]
    Broadcast(#[from] tokio::sync::broadcast::error::SendError<InputMessage>),
    #[error("raw image data missing")]
    RawImageMissing,
}

pub fn i32_to_duration(d: Option<i32>) -> Option<chrono::Duration> {
    if let Some(d) = d {
        if d <= 0 {
            None
        } else {
            Some(chrono::Duration::milliseconds(d as _))
        }
    } else {
        None
    }
}
