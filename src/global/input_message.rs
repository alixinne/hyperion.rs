use std::sync::Arc;

use tokio::sync::{oneshot, Mutex};

use crate::{
    api::json::message::EffectRequest, component::ComponentName, image::RawImage,
    instance::StartEffectError, models::Color,
};

use super::Message;

#[derive(Debug, Clone)]
pub struct InputMessage {
    source_id: usize,
    component: ComponentName,
    data: InputMessageData,
}

impl Message for InputMessage {
    type Data = InputMessageData;

    fn new(source_id: usize, component: ComponentName, data: Self::Data) -> Self {
        Self {
            source_id,
            component,
            data,
        }
    }

    fn source_id(&self) -> usize {
        self.source_id
    }

    fn component(&self) -> ComponentName {
        self.component
    }

    fn data(&self) -> &Self::Data {
        &self.data
    }

    fn unregister_source(global: &mut super::GlobalData, input_source: &super::InputSource<Self>) {
        global.unregister_input_source(input_source);
    }
}

pub type StartEffectResponseCallback = Mutex<Option<oneshot::Sender<Result<(), StartEffectError>>>>;

#[derive(Debug, Clone)]
pub enum InputMessageData {
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
    LedColors {
        priority: i32,
        duration: Option<chrono::Duration>,
        led_colors: Arc<Vec<Color>>,
    },
    Effect {
        priority: i32,
        duration: Option<chrono::Duration>,
        effect: Arc<EffectRequest>,
        response: Arc<StartEffectResponseCallback>,
    },
}

impl InputMessageData {
    pub fn priority(&self) -> Option<i32> {
        match self {
            InputMessageData::ClearAll => None,
            InputMessageData::Clear { priority } => Some(*priority),
            InputMessageData::SolidColor { priority, .. } => Some(*priority),
            InputMessageData::Image { priority, .. } => Some(*priority),
            InputMessageData::LedColors { priority, .. } => Some(*priority),
            InputMessageData::Effect { priority, .. } => Some(*priority),
        }
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        match self {
            InputMessageData::ClearAll => None,
            InputMessageData::Clear { .. } => None,
            InputMessageData::SolidColor { duration, .. } => *duration,
            InputMessageData::Image { duration, .. } => *duration,
            InputMessageData::LedColors { duration, .. } => *duration,
            InputMessageData::Effect { duration, .. } => *duration,
        }
    }
}
