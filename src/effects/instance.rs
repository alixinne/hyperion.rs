use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};

use tokio::sync::mpsc::{Receiver, Sender};

use crate::image::RawImage;

use super::{
    runtime::{RuntimeMethodError, RuntimeMethods},
    EffectMessage, EffectMessageKind,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlMessage {
    Abort,
}

pub struct InstanceMethods<X> {
    tx: Sender<EffectMessage<X>>,
    crx: RefCell<Receiver<ControlMessage>>,
    led_count: usize,
    deadline: Option<Instant>,
    aborted: Cell<bool>,
    extra: X,
}

impl<X> InstanceMethods<X> {
    pub fn new(
        tx: Sender<EffectMessage<X>>,
        crx: Receiver<ControlMessage>,
        led_count: usize,
        duration: Option<Duration>,
        extra: X,
    ) -> Self {
        Self {
            tx,
            crx: crx.into(),
            led_count,
            deadline: duration.map(|d| Instant::now() + d),
            aborted: false.into(),
            extra,
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

impl<X: std::fmt::Debug + Clone> RuntimeMethods for InstanceMethods<X> {
    fn get_led_count(&self) -> usize {
        self.led_count
    }

    fn abort(&self) -> bool {
        self.poll_control().is_err()
    }

    fn set_color(&self, color: crate::models::Color) -> Result<(), RuntimeMethodError> {
        self.poll_control()?;

        self.wrap_result(self.tx.blocking_send(EffectMessage {
            kind: EffectMessageKind::SetColor { color },
            extra: self.extra.clone(),
        }))
    }

    fn set_led_colors(&self, colors: Vec<crate::models::Color>) -> Result<(), RuntimeMethodError> {
        self.poll_control()?;

        self.wrap_result(self.tx.blocking_send(EffectMessage {
            kind: EffectMessageKind::SetLedColors {
                colors: colors.into(),
            },
            extra: self.extra.clone(),
        }))
    }

    fn set_image(&self, image: RawImage) -> Result<(), RuntimeMethodError> {
        self.poll_control()?;

        self.wrap_result(self.tx.blocking_send(EffectMessage {
            kind: EffectMessageKind::SetImage {
                image: image.into(),
            },
            extra: self.extra.clone(),
        }))
    }
}
