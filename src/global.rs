use std::collections::HashMap;
use std::sync::Arc;

use parse_display::Display;
use thiserror::Error;
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

#[derive(Display)]
#[display("`{name}` (id = {id}, priority = {priority:?}")]
pub struct InputSource {
    id: usize,
    name: String,
    priority: Option<i32>,
    input_tx: broadcast::Sender<InputMessage>,
}

impl InputSource {
    pub fn priority(&self) -> Option<i32> {
        self.priority
    }

    pub fn send(
        &self,
        message: InputMessage,
    ) -> Result<usize, broadcast::error::SendError<InputMessage>> {
        self.input_tx.send(message)
    }
}

pub struct InputSourceHandle {
    input_source: Arc<InputSource>,
    global: Global,
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

#[derive(Clone)]
pub struct Global(Arc<RwLock<GlobalData>>);

impl Global {
    pub async fn register_source(
        &self,
        name: String,
        priority: Option<i32>,
    ) -> Result<InputSourceHandle, InputSourceError> {
        let priority = if let Some(priority) = priority {
            if priority < 0 || priority > 255 {
                return Err(InputSourceError::InvalidPriority(priority));
            }

            Some(priority)
        } else {
            // TODO: Default value?
            None
        };

        Ok(InputSourceHandle {
            input_source: self.0.write().await.register_source(name, priority),
            global: self.clone(),
        })
    }

    pub async fn subscribe_input(&self) -> broadcast::Receiver<InputMessage> {
        self.0.read().await.input_tx.subscribe()
    }
}

pub struct GlobalData {
    input_tx: broadcast::Sender<InputMessage>,
    sources: HashMap<usize, Arc<InputSource>>,
    next_source_id: usize,
}

impl GlobalData {
    pub fn new() -> Self {
        let (input_tx, _) = broadcast::channel(4);
        Self {
            input_tx,
            sources: Default::default(),
            next_source_id: 1,
        }
    }

    pub fn wrap(self) -> Global {
        Global(Arc::new(RwLock::new(self)))
    }

    fn register_source(&mut self, name: String, priority: Option<i32>) -> Arc<InputSource> {
        let id = self.next_source_id;
        self.next_source_id += 1;

        let input_source = Arc::new(InputSource {
            id,
            name,
            priority,
            input_tx: self.input_tx.clone(),
        });

        info!("registered new source {}", *input_source);

        self.sources.insert(id, input_source.clone());

        input_source
    }

    fn unregister_source(&mut self, source: &InputSource) {
        if let Some(is) = self.sources.remove(&source.id) {
            info!("unregistered source {}", *is);
        }
    }
}
