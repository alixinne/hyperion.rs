//! Definition of the EffectEngine type

use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Instant;

use futures::prelude::*;
use futures::task::{Context, Poll};

use tokio::sync::mpsc;

use crate::hyperion::{Input, ServiceCommand};
use crate::image::RawImage;
use crate::servers::json::Effect as EffectName;

mod byte_rgb;
use byte_rgb::*;

mod effect;
use effect::*;

mod effect_definition;
use effect_definition::*;

mod effect_error;
use effect_error::*;

mod effect_listener;
use effect_listener::*;

mod hyperion_listener;
use hyperion_listener::*;

mod serde_ext;
use serde_ext::*;

/// Handle to the effect definitions storage
pub type EffectDefinitionsHandle = Arc<Mutex<EffectDefinitions>>;

/// Type to hold effect definitions
pub struct EffectDefinitions {
    /// List of known effects
    effects: HashMap<String, EffectDefinition>,
}

impl std::ops::Deref for EffectDefinitions {
    type Target = HashMap<String, EffectDefinition>;

    fn deref(&self) -> &Self::Target {
        &self.effects
    }
}

/// Effect engine
///
/// Hosts the Python interpreter to run effects on this hyperion instance.
pub struct EffectEngine {
    /// Effect definition storage
    effect_definitions: Arc<Mutex<EffectDefinitions>>,
    /// Effect running currently
    current_effect: Option<RunningEffect>,
    /// Effects being terminated
    terminating_effects: Vec<Option<RunningEffect>>,
}

/// Currently running effect
struct RunningEffect {
    /// Abort flag
    join_requested: Arc<AtomicBool>,
    /// Abort completed flag
    effect_terminated: Arc<AtomicBool>,
    /// Thread object running the effect
    thread: Option<std::thread::JoinHandle<Result<(), String>>>,
    /// Effect name
    name: String,
}

impl EffectEngine {
    /// Return a new effect engine
    ///
    /// # Parameters
    ///
    /// * `effects_path`: list of paths to load effect definitions from
    pub fn new(effects_path: Vec<PathBuf>) -> Self {
        let mut effects: Vec<EffectDefinition> = Vec::new();

        // TODO: Log errors
        for dir_path in effects_path {
            if let Ok(entries) = fs::read_dir(dir_path.clone()) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        let extension = path.extension();
                        if let Some(ext) = extension {
                            if ext == OsStr::new("json") {
                                // Possible JSON effect file
                                File::open(path)
                                    .map_err(|_| ())
                                    .and_then(|f| serde_json::from_reader(f).map_err(|_| ()))
                                    .map(|spec: crate::servers::json::EffectDefinition| {
                                        debug!("parsed effect definition for {}", spec.name);
                                        effects.push(EffectDefinition::new(spec, dir_path.clone()));
                                    })
                                    .ok();
                            }
                        }
                    }
                }
            }
        }

        let effects: HashMap<_, _> = effects
            .into_iter()
            .map(|spec| (spec.get_name().to_owned(), spec))
            .collect();

        Self {
            effect_definitions: Arc::new(Mutex::new(EffectDefinitions { effects })),
            current_effect: None,
            terminating_effects: Vec::new(),
        }
    }

    /// Launch an effect on this engine
    ///
    /// # Parameters
    ///
    /// * `effect_name`: name and parameters of the effect
    /// * `deadline`: instant at which the effect should time out
    /// * `sender`: channel to send updates to
    /// * `led_count`: number of LEDs currently managed
    pub fn launch(
        &mut self,
        effect_name: EffectName,
        deadline: Option<Instant>,
        sender: mpsc::Sender<Input>,
        led_count: usize,
    ) -> Result<(), EffectError> {
        let effects = self.effect_definitions.lock().unwrap();

        if let Some(effect) = effects.get(&effect_name.name) {
            // Construct args object
            let args = Some(
                effect_name
                    .args
                    .unwrap_or_else(|| effect.get_args().clone()),
            );
            // Read effect code
            let code = fs::read_to_string(effect.get_script().clone())?;

            // Spawn effect thread
            let join_flag = Arc::new(AtomicBool::new(false));
            let terminated_flag = Arc::new(AtomicBool::new(false));

            let running_effect = RunningEffect {
                name: effect_name.name.clone(),
                join_requested: join_flag.clone(),
                effect_terminated: terminated_flag.clone(),
                thread: Some(std::thread::spawn(move || {
                    let res = Effect::run(
                        &code,
                        Box::new(HyperionListener::new(sender)),
                        led_count,
                        args,
                        deadline,
                        join_flag,
                    );

                    // Notify effect finished running
                    terminated_flag.store(true, Ordering::SeqCst);
                    res
                })),
            };

            // Release borrow
            drop(effects);

            // Add to running effects
            if let Some(previous_effect) = self.current_effect.replace(running_effect) {
                self.terminate(previous_effect);
            }

            Ok(())
        } else {
            Err(EffectErrorKind::NotFound(effect_name.name).into())
        }
    }

    /// Build and return a list of known effect definitions
    pub fn get_definitions(&self) -> EffectDefinitionsHandle {
        self.effect_definitions.clone()
    }

    /// Add the given effect to the termination list
    ///
    /// # Parameters
    ///
    /// * `effect`: effect to terminate
    fn terminate(&mut self, effect: RunningEffect) {
        // Request effect termination
        effect.join_requested.store(true, Ordering::SeqCst);

        // Add to termination list
        self.terminating_effects.push(Some(effect));
    }

    /// Abort all effects
    pub fn clear_all(&mut self) {
        if let Some(previous_effect) = self.current_effect.take() {
            self.terminate(previous_effect);
        }
    }
}

impl Stream for EffectEngine {
    type Item = ServiceCommand;

    fn poll_next(mut self: Pin<&mut Self>, _ctx: &mut Context) -> Poll<Option<Self::Item>> {
        let mut result = Poll::Pending;
        let mut remove_idx = None;

        for (i, effect) in self.terminating_effects.iter_mut().enumerate() {
            let terminated = effect
                .as_ref()
                .map(|effect| effect.effect_terminated.load(Ordering::SeqCst))
                .unwrap_or(false);

            if terminated {
                let mut effect = effect.take().unwrap();
                result = Poll::Ready(Some(ServiceCommand::EffectCompleted {
                    name: effect.name,
                    result: effect.thread.take().unwrap().join().unwrap(),
                }));
                remove_idx = Some(i);
            }
        }

        if let Some(idx) = remove_idx {
            self.terminating_effects.remove(idx);
        } else {
            // Check if the current effect finished early
            if self
                .current_effect
                .as_ref()
                .map(|e| e.effect_terminated.load(Ordering::SeqCst))
                .unwrap_or(false)
            {
                let mut effect = self.current_effect.take().unwrap();
                result = Poll::Ready(Some(ServiceCommand::EffectCompleted {
                    name: effect.name,
                    result: effect.thread.take().unwrap().join().unwrap(),
                }));
            }
        }

        result
    }
}
