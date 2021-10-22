use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::pin::Pin;
use std::time::Instant;

use futures::Future;
use tokio::select;

use crate::{
    api::types::PriorityInfo,
    component::ComponentName,
    global::{Global, InputMessage, InputMessageData, Message},
    models::Color,
};

mod effect_runner;
pub use effect_runner::StartEffectError;
use effect_runner::*;

mod muxed_message;
pub use muxed_message::*;

#[derive(Debug, Clone, Copy)]
pub struct MuxerConfig {
    pub led_count: usize,
}

impl From<MuxerConfig> for EffectRunnerConfig {
    fn from(MuxerConfig { led_count }: MuxerConfig) -> Self {
        Self { led_count }
    }
}

#[derive(Debug)]
struct InputEntry {
    input_id: usize,
    message: InputMessage,
    expires: Option<Instant>,
    effect_key: Option<RunningEffectKey>,
}

pub struct PriorityMuxer {
    global: Global,
    inputs: BTreeMap<i32, InputEntry>,
    input_id: usize,
    timeouts: HashMap<
        usize,
        Box<dyn Fn() -> Pin<Box<dyn Future<Output = (usize, i32)> + Send + Sync>> + Send + Sync>,
    >,
    effect_runner: EffectRunner,
}

pub const MAX_PRIORITY: i32 = 256;
const MUXER_ID: usize = 0;

impl PriorityMuxer {
    pub async fn new(global: Global, config: MuxerConfig) -> Self {
        let mut this = Self {
            global: global.clone(),
            inputs: Default::default(),
            timeouts: Default::default(),
            input_id: 0,
            effect_runner: EffectRunner::new(global, config.into()),
        };

        // Start by clearing all outputs
        this.clear_all().await;

        this
    }

    fn current_priority(&self) -> i32 {
        *self.inputs.keys().next().unwrap()
    }

    fn notify_output_change(&mut self) -> Option<MuxedMessage> {
        let target = self.inputs.values().next()?;
        Some(MuxedMessage::new(
            target.message.data().clone().try_into().ok()?,
        ))
    }

    fn insert_input(
        &mut self,
        priority: i32,
        input: InputMessage,
        effect_key: Option<RunningEffectKey>,
    ) -> Option<InputEntry> {
        // Get the duration of this input
        let expires = input
            .data()
            .duration()
            .map(|duration| Instant::now() + duration.to_std().unwrap());

        // Insert the input, replacing the old one
        let before = self.inputs.insert(
            priority,
            InputEntry {
                input_id: self.input_id,
                message: input,
                expires,
                effect_key,
            },
        );

        // Drop the future for the previous input
        if let Some(InputEntry { input_id, .. }) = before {
            self.timeouts.remove(&input_id);
        }

        // Add the future for the current input
        if let Some(expires) = expires {
            let id = self.input_id;

            self.timeouts.insert(
                self.input_id,
                Box::new(move || {
                    Box::pin(async move {
                        tokio::time::sleep_until(expires.into()).await;
                        (id, priority)
                    })
                }),
            );
        }

        // Increment id
        self.input_id += 1;

        before
    }

    fn clear_inputs(&mut self) {
        self.inputs.clear();
        self.timeouts.clear();
    }

    fn clear_input(&mut self, priority: i32) -> bool {
        if let Some(InputEntry { input_id, .. }) = self.inputs.remove(&priority) {
            self.timeouts.remove(&input_id);
            true
        } else {
            false
        }
    }

    async fn clear_all(&mut self) -> Option<MuxedMessage> {
        self.clear_inputs();
        debug!("cleared all inputs");

        // Clear all running effects
        self.effect_runner.clear_all().await;

        self.insert_input(
            MAX_PRIORITY,
            InputMessage::new(
                MUXER_ID,
                ComponentName::All,
                InputMessageData::SolidColor {
                    priority: MAX_PRIORITY,
                    duration: None,
                    color: Color::from_components((0, 0, 0)),
                },
            ),
            None,
        );

        debug!(priority = %self.current_priority(), "current priority changed");
        self.notify_output_change()
    }

    async fn clear(&mut self, priority: i32) -> Option<MuxedMessage> {
        assert!(priority < MAX_PRIORITY);
        // We should notify if we're clearing the current priority
        let mut notify = self.current_priority() == priority;

        // Clear running effect on that priority, this notifies if an effect is running in the
        // clearing priority
        notify = self.effect_runner.clear(priority).await || notify;

        notify = self.clear_input(priority) && notify;
        debug!(priority = %priority, "cleared priority");

        if notify {
            debug!(priority = %self.current_priority(), "current priority changed");
            self.notify_output_change()
        } else {
            None
        }
    }

    async fn handle_input(&mut self, input: InputMessage) -> Option<MuxedMessage> {
        let priority = input.data().priority().unwrap();
        let is_new = priority < self.current_priority();
        let notify = priority <= self.current_priority();

        let before = self.insert_input(priority, input.clone(), None);
        trace!(
            priority = %priority,
            after = ?input,
            before = ?before,
            "new command for priority level",
        );

        if let Some(key) = before.and_then(|entry| entry.effect_key) {
            self.effect_runner.abort(key).await;
        }

        if is_new {
            debug!(priority = %priority, "current priority changed");
        }

        if notify {
            self.notify_output_change()
        } else {
            None
        }
    }

    async fn handle_timeout(&mut self, (id, priority): (usize, i32)) -> Option<MuxedMessage> {
        let current_priority = self.current_priority();

        // Check if the input for the target priority is still the one mentioned in the future
        if let Some(input) = self.inputs.get(&priority) {
            if input.input_id == id {
                if let Some(removed) = self.inputs.remove(&priority) {
                    debug!(input = ?removed, "input timeout");
                }
            } else {
                warn!(id = %id, "unexpected timeout for input");
            }
        }

        // Remove the future
        self.timeouts.remove(&id);

        // If the timeout priority is <=, then it was the current input
        if current_priority >= priority {
            debug!(priority = %current_priority, "current priority changed");
            self.notify_output_change()
        } else {
            None
        }
    }

    pub async fn handle_message(&mut self, input: InputMessage) -> Option<MuxedMessage> {
        trace!(input = ?input, "got input");

        // Check if this will change the output
        match input.data() {
            InputMessageData::ClearAll => self.clear_all().await,
            InputMessageData::Clear { priority } => self.clear(*priority).await,
            InputMessageData::Effect {
                priority,
                duration,
                effect,
                response,
            } => {
                let result = self.effect_runner.start(*priority, *duration, effect).await;
                let response = response.clone();

                match result {
                    Ok(ref key) => {
                        // Register this input to keep track of it
                        self.insert_input(*priority, input, Some(*key));
                    }
                    Err(_) => {}
                }

                if let Some(tx) = (*response.lock().await).take() {
                    // We ignore send errors, this means the caller doesn't care for the response
                    tx.send(result.map(|_| ())).ok();
                } else {
                    // TODO: Remove this when effect requests are properly forwarded to only one
                    // instance
                    warn!("effect request already answered");
                }

                // No MuxedMessage results from this, the effect will publish updates later
                None
            }
            _ => self.handle_input(input).await,
        }
    }

    pub async fn current_priorities(&self) -> Vec<PriorityInfo> {
        self.global
            .read_input_sources(|sources| {
                // Inputs are sorted by priority, so i == 0 denotes the
                // current (active) entry
                self.inputs
                    .values()
                    .enumerate()
                    .map(|(i, entry)| {
                        PriorityInfo::new(
                            &entry.message,
                            sources
                                .get(&entry.message.source_id())
                                .map(|source| source.name().to_string())
                                .unwrap_or_else(String::new),
                            entry.expires,
                            i == 0,
                        )
                    })
                    .collect()
            })
            .await
    }

    async fn handle_effect_message(
        &mut self,
        msg: Option<EffectRunnerUpdate>,
    ) -> Option<MuxedMessage> {
        match msg {
            Some(msg) => {
                match msg {
                    EffectRunnerUpdate::Message(msg) => {
                        (msg.priority() <= self.current_priority()).then(|| msg)
                    }
                    EffectRunnerUpdate::Completed { key, priority } => {
                        let notify = self.current_priority() == priority;

                        // Remove corresponding input entry
                        let entry = self.inputs.entry(priority);
                        match entry {
                            std::collections::btree_map::Entry::Vacant(_) => {
                                // Effect was already removed by a clear call or similar
                            }
                            std::collections::btree_map::Entry::Occupied(entry) => {
                                // Remove the input entry if it's the one that triggered the effect
                                if entry.get().effect_key == Some(key) {
                                    entry.remove();
                                }
                            }
                        }

                        // Notify of the priority change, if any
                        if notify {
                            self.notify_output_change()
                        } else {
                            None
                        }
                    }
                }
            }
            None => {
                // No message
                None
            }
        }
    }

    pub async fn update(&mut self) -> Option<MuxedMessage> {
        // Check for input timeouts
        if self.timeouts.len() > 0 {
            select! {
                id = futures::future::select_all(self.timeouts.values().map(|f| f())) => {
                    self.handle_timeout(id.0).await
                },
                msg = self.effect_runner.update() => {
                    self.handle_effect_message(msg).await
                }
            }
        } else {
            let msg = self.effect_runner.update().await;
            self.handle_effect_message(msg).await
        }
    }
}
