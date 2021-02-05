use std::sync::Arc;

use tokio::sync::broadcast;
use tokio::sync::RwLock;

use crate::image::RawImage;
use crate::models::Color;

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

pub type Global = Arc<RwLock<GlobalData>>;

pub struct GlobalData {
    pub input_tx: broadcast::Sender<InputMessage>,
}

impl GlobalData {
    pub fn new() -> Self {
        let (input_tx, _) = broadcast::channel(4);
        Self { input_tx }
    }

    pub fn wrap(self) -> Global {
        Arc::new(RwLock::new(self))
    }
}
