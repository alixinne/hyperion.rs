use std::time::{Duration, Instant};

use async_trait::async_trait;
use thiserror::Error;
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Mutex,
};

use crate::{
    image::{RawImage, RawImageError},
    models::Color,
};

use super::EffectMessageKind;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlMessage {
    Abort,
}

struct InstanceMethodsData {
    crx: Receiver<ControlMessage>,
    aborted: bool,
}

pub struct InstanceMethods {
    tx: Sender<EffectMessageKind>,
    led_count: usize,
    deadline: Option<Instant>,
    data: Mutex<InstanceMethodsData>,
}

impl InstanceMethods {
    pub fn new(
        tx: Sender<EffectMessageKind>,
        crx: Receiver<ControlMessage>,
        led_count: usize,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            tx,
            led_count,
            deadline: duration.map(|d| Instant::now() + d),
            data: Mutex::new(InstanceMethodsData {
                crx: crx.into(),
                aborted: false.into(),
            }),
        }
    }

    fn completed(&self, data: &InstanceMethodsData) -> bool {
        data.aborted || self.deadline.map(|d| Instant::now() > d).unwrap_or(false)
    }

    /// Returns true if the should abort
    async fn poll_control(&self) -> Result<(), RuntimeMethodError> {
        let mut data = self.data.lock().await;
        match data.crx.try_recv() {
            Ok(m) => match m {
                ControlMessage::Abort => {
                    data.aborted = true;
                    return Err(RuntimeMethodError::EffectAborted);
                }
            },
            Err(err) => {
                match err {
                    tokio::sync::mpsc::error::TryRecvError::Empty => {
                        // No control messages pending
                    }
                    tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                        // We were disconnected
                        data.aborted = true;
                        return Err(RuntimeMethodError::EffectAborted);
                    }
                }
            }
        }

        if self.completed(&*data) {
            Err(RuntimeMethodError::EffectAborted)
        } else {
            Ok(())
        }
    }

    async fn wrap_result<T, E: Into<RuntimeMethodError>>(
        &self,
        res: Result<T, E>,
    ) -> Result<T, RuntimeMethodError> {
        match res {
            Ok(t) => Ok(t),
            Err(err) => {
                // TODO: Log error?
                self.data.lock().await.aborted = true;
                Err(err.into())
            }
        }
    }
}

#[async_trait]
impl RuntimeMethods for InstanceMethods {
    fn get_led_count(&self) -> usize {
        self.led_count
    }

    async fn abort(&self) -> bool {
        self.poll_control().await.is_err()
    }

    async fn set_color(&self, color: crate::models::Color) -> Result<(), RuntimeMethodError> {
        self.poll_control().await?;

        self.wrap_result(self.tx.send(EffectMessageKind::SetColor { color }).await)
            .await
    }

    async fn set_led_colors(
        &self,
        colors: Vec<crate::models::Color>,
    ) -> Result<(), RuntimeMethodError> {
        self.poll_control().await?;

        self.wrap_result(
            self.tx
                .send(EffectMessageKind::SetLedColors {
                    colors: colors.into(),
                })
                .await,
        )
        .await
    }

    async fn set_image(&self, image: RawImage) -> Result<(), RuntimeMethodError> {
        self.poll_control().await?;

        self.wrap_result(
            self.tx
                .send(EffectMessageKind::SetImage {
                    image: image.into(),
                })
                .await,
        )
        .await
    }
}

#[async_trait]
pub trait RuntimeMethods: Send {
    fn get_led_count(&self) -> usize;
    async fn abort(&self) -> bool;

    async fn set_color(&self, color: Color) -> Result<(), RuntimeMethodError>;
    async fn set_led_colors(&self, colors: Vec<Color>) -> Result<(), RuntimeMethodError>;
    async fn set_image(&self, image: RawImage) -> Result<(), RuntimeMethodError>;
}

#[derive(Debug, Error)]
pub enum RuntimeMethodError {
    #[cfg(feature = "python")]
    #[error("Invalid arguments to hyperion.{name}")]
    InvalidArguments { name: &'static str },
    #[cfg(feature = "python")]
    #[error("Length of bytearray argument should be 3*ledCount")]
    InvalidByteArray,
    #[error("Effect aborted")]
    EffectAborted,
    #[error(transparent)]
    InvalidImageData(#[from] RawImageError),
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for RuntimeMethodError {
    fn from(_: tokio::sync::mpsc::error::SendError<T>) -> Self {
        Self::EffectAborted
    }
}
