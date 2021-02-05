use std::sync::Arc;

use parse_display::Display;
use thiserror::Error;
use tokio::sync::broadcast;

use crate::global::{Global, InputMessage, InputMessageData};

#[derive(Display)]
#[display("`{name}` (id = {id}, priority = {priority:?})")]
pub struct InputSource {
    pub(super) id: usize,
    pub(super) name: String,
    pub(super) priority: Option<i32>,
    pub(super) input_tx: broadcast::Sender<InputMessage>,
}

impl InputSource {
    pub fn priority(&self) -> Option<i32> {
        self.priority
    }

    pub fn send(
        &self,
        message: InputMessageData,
    ) -> Result<usize, broadcast::error::SendError<InputMessage>> {
        self.input_tx.send(InputMessage {
            source_id: self.id,
            data: message,
        })
    }
}

pub struct InputSourceHandle {
    pub(super) input_source: Arc<InputSource>,
    pub(super) global: Global,
}

impl std::ops::Deref for InputSourceHandle {
    type Target = InputSource;

    fn deref(&self) -> &Self::Target {
        &*self.input_source
    }
}

impl Drop for InputSourceHandle {
    fn drop(&mut self) {
        // TODO: Can this block?
        futures::executor::block_on(async {
            self.global
                .0
                .write()
                .await
                .unregister_source(&*self.input_source)
        });
    }
}

#[derive(Debug, Error)]
pub enum InputSourceError {
    #[error("invalid priority: {0}")]
    InvalidPriority(i32),
}
