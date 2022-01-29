use slotmap::SlotMap;
use thiserror::Error;
use tokio::sync::mpsc;

use crate::{
    api::json::message::EffectRequest,
    effects::{self, EffectDefinitionError, EffectRunHandle, RunEffectError},
    global::Global,
    instance::muxer::MuxedMessageData,
};

use super::MuxedMessage;

#[derive(Debug, Error)]
pub enum StartEffectError {
    #[error(transparent)]
    Definition(#[from] EffectDefinitionError),
    #[error(transparent)]
    Run(#[from] RunEffectError),
    #[error("effect '{name}' not found")]
    NotFound { name: String },
}

slotmap::new_key_type! { pub struct RunningEffectKey; }

pub type EffectMessage = effects::EffectMessage<RunningEffectKey>;

#[derive(Debug, Clone)]
pub enum EffectRunnerUpdate {
    Message(MuxedMessage),
    Completed {
        key: RunningEffectKey,
        priority: i32,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct EffectRunnerConfig {
    pub led_count: usize,
}

pub struct EffectRunner {
    global: Global,
    effect_tx: mpsc::Sender<EffectMessage>,
    effect_rx: mpsc::Receiver<EffectMessage>,
    running_effects: SlotMap<RunningEffectKey, Option<EffectRunHandle>>,
    config: EffectRunnerConfig,
}

impl EffectRunner {
    pub fn new(global: Global, config: EffectRunnerConfig) -> Self {
        let (effect_tx, effect_rx) = mpsc::channel(4);

        Self {
            global,
            effect_tx,
            effect_rx,
            running_effects: Default::default(),
            config,
        }
    }

    pub async fn abort(&mut self, key: RunningEffectKey) {
        if let Some(Some(handle)) = self.running_effects.get_mut(key) {
            handle.abort().await;
        }
    }

    pub async fn clear_all(&mut self) -> bool {
        let mut cleared_effects = false;

        for effect in self.running_effects.values_mut() {
            if let Some(handle) = effect.as_mut() {
                cleared_effects = true;
                handle.abort().await;
            }
        }

        if cleared_effects {
            debug!("cleared all running effects");
        }

        cleared_effects
    }

    pub async fn clear(&mut self, priority: i32) -> bool {
        let mut cleared_effects = false;

        for effect in self.running_effects.values_mut() {
            if let Some(handle) = effect.as_mut() {
                if handle.priority == priority {
                    cleared_effects = true;
                    handle.abort().await;
                }
            }
        }

        if cleared_effects {
            debug!(priority, "cleared running effects");
        }

        cleared_effects
    }

    pub async fn start(
        &mut self,
        priority: i32,
        duration: Option<chrono::Duration>,
        effect: &EffectRequest,
    ) -> Result<RunningEffectKey, StartEffectError> {
        // TODO: Read per-instance effects
        self.global
            .clone()
            .read_effects(|effects| {
                // Find the effect definition
                let result = if let Some(handle) = effects.find_effect(&effect.name) {
                    let key = self.running_effects.insert(None);

                    match handle.run(
                        effect.args.clone().into(),
                        self.config.led_count,
                        duration,
                        priority,
                        self.effect_tx.clone(),
                        key,
                    ) {
                        Ok(handle) => {
                            *self.running_effects.get_mut(key).unwrap() = Some(handle);
                            info!(name = %effect.name, "started effect");
                            Ok(key)
                        }
                        Err(err) => {
                            self.running_effects.remove(key);
                            warn!(name = %effect.name, error = %err, "could not start effect");
                            Err(err.into())
                        }
                    }
                } else {
                    warn!(name = %effect.name, "effect not found");
                    Err(StartEffectError::NotFound {
                        name: effect.name.clone(),
                    })
                };

                async move {
                    if let Ok(key) = result {
                        // Clear existing effects with the same priority as the newly-started one
                        for (existing_key, handle) in self.running_effects.iter_mut() {
                            if existing_key == key {
                                continue;
                            }
                            if let Some(handle) = handle {
                                if priority == handle.priority {
                                    handle.abort().await;
                                }
                            }
                        }
                    }

                    result
                }
            })
            .await
            .await
    }

    pub async fn update(&mut self) -> Option<EffectRunnerUpdate> {
        let msg = self.effect_rx.recv().await?;

        // Log received message
        trace!(message = ?msg, "got effect message");

        let key = msg.extra;
        let running_effect = || {
            // expect: we only clear slots when an effect completes, so this one can't be None
            // expect: Self::update can only run when start has completed, thus the handle slot
            // can't be None either
            self.running_effects
                .get(key)
                .expect("invalid effect handle")
                .as_ref()
                .expect("handle shouldn't be null")
        };

        // Turn this into a MuxedMessage
        match msg.kind {
            effects::EffectMessageKind::SetColor { color } => Some(EffectRunnerUpdate::Message(
                MuxedMessage::new(MuxedMessageData::SolidColor {
                    priority: running_effect().priority,
                    duration: None,
                    color,
                }),
            )),

            effects::EffectMessageKind::SetImage { image } => Some(EffectRunnerUpdate::Message(
                MuxedMessage::new(MuxedMessageData::Image {
                    priority: running_effect().priority,
                    duration: None,
                    image: image.clone(),
                }),
            )),

            effects::EffectMessageKind::SetLedColors { colors } => Some(
                EffectRunnerUpdate::Message(MuxedMessage::new(MuxedMessageData::LedColors {
                    priority: running_effect().priority,
                    duration: None,
                    led_colors: colors.clone(),
                })),
            ),

            effects::EffectMessageKind::Completed { result } => {
                // The effect has completed, remove it from the running_effects list
                let priority = if let Some(mut effect) = self.running_effects.remove(key).flatten()
                {
                    effect.finish().await;
                    effect.priority
                } else {
                    panic!("unexpected null handle for completed effect");
                };

                // Log result
                match result {
                    Ok(_) => {
                        info!("effect completed");
                    }
                    Err(err) => {
                        error!(error = %err, "effect completed with errors");
                    }
                }

                Some(EffectRunnerUpdate::Completed { key, priority })
            }
        }
    }
}
