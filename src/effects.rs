use std::sync::Arc;

use thiserror::Error;
use tokio::{
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};

use crate::{global::InputSourceError, image::RawImage, models::Color};

mod definition;
pub use definition::*;

mod providers;
pub use providers::Providers;

mod instance;
use instance::*;

use self::providers::{Provider, ProviderError};

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
    Completed { result: Result<(), ProviderError> },
}

#[derive(Default, Debug, Clone)]
pub struct EffectRegistry {
    effects: Vec<EffectHandle>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &EffectDefinition> {
        self.effects.iter().map(|handle| &handle.definition)
    }

    pub fn find_effect(&self, name: &str) -> Option<&EffectHandle> {
        self.effects.iter().find(|e| e.definition.name == name)
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    /// Add definitions to this registry
    ///
    /// # Parameters
    ///
    /// * `providers`: effect providers
    /// * `definitions`: effect definitions to register
    ///
    /// # Returns
    ///
    /// Effect definitions that are not supported by any provider.
    pub fn add_definitions(
        &mut self,
        providers: &Providers,
        definitions: Vec<EffectDefinition>,
    ) -> Vec<EffectDefinition> {
        let mut remaining = vec![];

        for definition in definitions {
            if let Some(provider) = providers.get(&definition.script) {
                debug!(provider=?provider, effect=%definition.name, "assigned provider to effect");

                self.effects.push(EffectHandle {
                    definition,
                    provider,
                });
            } else {
                debug!(effect=%definition.name, "no provider for effect");

                remaining.push(definition);
            }
        }

        remaining
    }
}

#[derive(Debug, Clone)]
pub struct EffectHandle {
    pub definition: EffectDefinition,
    provider: Arc<dyn Provider>,
}

impl EffectHandle {
    pub fn run<X: std::fmt::Debug + Clone + Send + 'static>(
        &self,
        args: serde_json::Value,
        led_count: usize,
        duration: Option<chrono::Duration>,
        priority: i32,
        tx: Sender<EffectMessage<X>>,
        extra: X,
    ) -> Result<EffectRunHandle, RunEffectError> {
        // Resolve path
        let full_path = self.definition.script_path()?;

        // Clone provider arc
        let provider = self.provider.clone();

        // Create control channel
        let (ctx, crx) = channel(1);

        // Create channel to wrap data
        let (etx, mut erx) = channel(1);

        // Create instance methods
        let methods =
            InstanceMethods::new(etx, crx, led_count, duration.and_then(|d| d.to_std().ok()));

        // Run effect
        let join_handle = tokio::task::spawn(async move {
            // Create the blocking task
            let mut run_effect =
                tokio::task::spawn_blocking(move || provider.run(&full_path, args, methods));

            // Join the blocking task while forwarding the effect messages
            let result = loop {
                tokio::select! {
                    kind = erx.recv() => {
                        if let Some(kind) = kind {
                            // Add the extra marker to the message and forward it to the instance
                            let msg = EffectMessage { kind, extra: extra.clone() };

                            if let Err(err) = tx.send(msg).await {
                                // This would happen if the effect is running and the instance has
                                // already shutdown.
                                error!(err=%err, "failed to forward effect message");
                                return;
                            }
                        }
                    }
                    result = &mut run_effect => {
                        // Unwrap blocking result
                        break result.expect("failed to await blocking task");
                    }
                }
            };

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
}
