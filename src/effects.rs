use std::sync::Arc;

use thiserror::Error;
use tokio::{
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};

use crate::{global::InputSourceError, image::RawImage, models::Color};

mod definition;
pub use definition::*;

mod runtime;

mod instance;
use instance::*;

pub struct EffectRunHandle {
    ctx: Sender<ControlMessage>,
    join_handle: Option<JoinHandle<()>>,

    pub priority: i32,
}

impl EffectRunHandle {
    pub async fn abort(&mut self) {
        self.ctx
            .send(ControlMessage::Abort)
            .await
            .expect("failed to send message");
    }

    pub async fn finish(&mut self) {
        if let Some(jh) = self.join_handle.take() {
            jh.await.expect("failed to join task");
        }
    }
}

impl Drop for EffectRunHandle {
    fn drop(&mut self) {
        if self.join_handle.is_some() {
            let ctx = self.ctx.clone();
            tokio::task::spawn(async move {
                // This handle has been discarded, try to abort the running script as best effort
                ctx.send(ControlMessage::Abort).await.ok();
            });
        }
    }
}

#[derive(Debug, Error)]
pub enum RunEffectError {
    #[error(transparent)]
    InputSource(#[from] InputSourceError),
    #[error(transparent)]
    EffectDefinition(#[from] EffectDefinitionError),
}

#[derive(Debug)]
pub struct EffectMessage<X> {
    pub kind: EffectMessageKind,
    pub extra: X,
}

#[derive(Debug)]
pub enum EffectMessageKind {
    SetColor { color: Color },
    SetImage { image: Arc<RawImage> },
    SetLedColors { colors: Arc<Vec<Color>> },
    Completed { result: Result<(), pyo3::PyErr> },
}

pub fn run<X: std::fmt::Debug + Clone + Send + 'static>(
    effect: &EffectDefinition,
    args: serde_json::Value,
    led_count: usize,
    duration: Option<chrono::Duration>,
    priority: i32,
    tx: Sender<EffectMessage<X>>,
    extra: X,
) -> Result<EffectRunHandle, RunEffectError> {
    // Resolve path
    let full_path = effect.script_path()?;

    // Create control channel
    let (ctx, crx) = channel(1);

    // Create instance methods
    let methods = InstanceMethods::new(
        tx.clone(),
        crx,
        led_count,
        duration.and_then(|d| d.to_std().ok()),
        extra.clone(),
    );

    // Run effect
    let join_handle = tokio::task::spawn(async move {
        // Run the blocking task
        let result = tokio::task::spawn_blocking(move || runtime::run(&full_path, args, methods))
            .await
            .expect("failed to await blocking task");

        // Send the completion, ignoring failures in case we're shutting down
        tx.send(EffectMessage {
            kind: EffectMessageKind::Completed { result },
            extra,
        })
        .await
        .ok();
    });

    Ok(EffectRunHandle {
        ctx,
        join_handle: join_handle.into(),
        priority,
    })
}
