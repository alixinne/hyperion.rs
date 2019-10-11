//! Definition of the EffectEngine type

use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

use futures::{Async, Stream};

use crate::hyperion::{ServiceCommand, ServiceInputSender};
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

/// Effect engine
///
/// Hosts the Python interpreter to run effects on this hyperion instance.
pub struct EffectEngine {
    /// List of known effects
    effects: HashMap<String, EffectDefinition>,
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

        let effects = effects
            .into_iter()
            .map(|spec| (spec.get_name().to_owned(), spec))
            .collect();

        Self {
            effects,
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
        sender: ServiceInputSender,
        led_count: usize,
    ) -> Result<(), EffectError> {
        if let Some(effect) = self.effects.get(&effect_name.name) {
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
    pub fn get_definitions(&self) -> Vec<crate::servers::json::EffectDefinition> {
        let mut effects: Vec<_> = self
            .effects
            .iter()
            .map(|(_k, v)| (*v.get_definition()).clone())
            .collect();

        effects.sort_by(|a, b| a.name.cmp(&b.name));
        effects
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
    type Error = ();

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        let mut result = Ok(Async::NotReady);
        let mut remove_idx = None;

        for (i, effect) in self.terminating_effects.iter_mut().enumerate() {
            let terminated = effect
                .as_ref()
                .map(|effect| effect.effect_terminated.load(Ordering::SeqCst))
                .unwrap_or(false);

            if terminated {
                let mut effect = effect.take().unwrap();
                result = Ok(Async::Ready(Some(ServiceCommand::EffectCompleted {
                    name: effect.name,
                    result: effect.thread.take().unwrap().join().unwrap(),
                })));
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
                result = Ok(Async::Ready(Some(ServiceCommand::EffectCompleted {
                    name: effect.name,
                    result: effect.thread.take().unwrap().join().unwrap(),
                })));
            }
        }

        result
    }
}
