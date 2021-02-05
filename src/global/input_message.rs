use std::sync::Arc;

use crate::{image::RawImage, models::Color};

#[derive(Debug, Clone)]
pub enum InputMessage {
    ClearAll,
    Clear {
        priority: i32,
    },
    SolidColor {
        priority: i32,
        duration: Option<chrono::Duration>,
        color: Color,
    },
    Image {
        priority: i32,
        duration: Option<chrono::Duration>,
        image: Arc<RawImage>,
    },
}
