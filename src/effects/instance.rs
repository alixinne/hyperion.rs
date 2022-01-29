use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};

use thiserror::Error;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{
    image::{RawImage, RawImageError},
    models::Color,
};

use super::EffectMessageKind;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlMessage {
    Abort,
}

pub struct InstanceMethods {
    tx: Sender<EffectMessageKind>,
    crx: RefCell<Receiver<ControlMessage>>,
    led_count: usize,
    deadline: Option<Instant>,
    aborted: Cell<bool>,
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
            crx: crx.into(),
            led_count,
            deadline: duration.map(|d| Instant::now() + d),
            aborted: false.into(),
        }
    }

    fn completed(&self) -> bool {
        self.aborted.get() || self.deadline.map(|d| Instant::now() > d).unwrap_or(false)
    }

    /// Returns true if the should abort
    fn poll_control(&self) -> Result<(), RuntimeMethodError> {
        match self.crx.borrow_mut().try_recv() {
            Ok(m) => match m {
                ControlMessage::Abort => {
                    self.aborted.set(true);
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
                        self.aborted.set(true);
                        return Err(RuntimeMethodError::EffectAborted);
                    }
                }
            }
        }

        if self.completed() {
            Err(RuntimeMethodError::EffectAborted)
        } else {
            Ok(())
        }
    }

    fn wrap_result<T, E: Into<RuntimeMethodError>>(
        &self,
        res: Result<T, E>,
    ) -> Result<T, RuntimeMethodError> {
        match res {
            Ok(t) => Ok(t),
            Err(err) => {
                // TODO: Log error?
                self.aborted.set(true);
                Err(err.into())
            }
        }
    }
}

impl RuntimeMethods for InstanceMethods {
    fn get_led_count(&self) -> usize {
        self.led_count
    }

    fn abort(&self) -> bool {
        self.poll_control().is_err()
    }

    fn set_color(&self, color: crate::models::Color) -> Result<(), RuntimeMethodError> {
        self.poll_control()?;

        self.wrap_result(self.tx.blocking_send(EffectMessageKind::SetColor { color }))
    }

    fn set_led_colors(&self, colors: Vec<crate::models::Color>) -> Result<(), RuntimeMethodError> {
        self.poll_control()?;

        self.wrap_result(self.tx.blocking_send(EffectMessageKind::SetLedColors {
            colors: colors.into(),
        }))
    }

    fn set_image(&self, image: RawImage) -> Result<(), RuntimeMethodError> {
        self.poll_control()?;

        self.wrap_result(self.tx.blocking_send(EffectMessageKind::SetImage {
            image: image.into(),
        }))
    }
}

pub trait RuntimeMethods {
    fn get_led_count(&self) -> usize;
    fn abort(&self) -> bool;

    fn set_color(&self, color: Color) -> Result<(), RuntimeMethodError>;
    fn set_led_colors(&self, colors: Vec<Color>) -> Result<(), RuntimeMethodError>;
    fn set_image(&self, image: RawImage) -> Result<(), RuntimeMethodError>;
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
