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

use crate::hyperion::ServiceInputSender;
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
    /// List of running effects
    running_effects: Vec<RunningEffect>,
}

/// Currently running effect
struct RunningEffect {
    /// Abort flag
    join_requested: Arc<AtomicBool>,
    /// Thread object running the effect
    thread: Option<std::thread::JoinHandle<Result<(), String>>>,
}

impl Drop for RunningEffect {
    fn drop(&mut self) {
        self.join_requested.store(true, Ordering::SeqCst);

        if let Err(error) = self.thread.take().unwrap().join().unwrap() {
            warn!("python effect error: {}", error);
        }
    }
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
            running_effects: Vec::new(),
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
            let flag = Arc::new(AtomicBool::new(false));
            let running_effect = RunningEffect {
                join_requested: flag.clone(),
                thread: Some(std::thread::spawn(move || {
                    let res = Effect::run(
                        &code,
                        Box::new(HyperionListener::new(sender)),
                        led_count,
                        args,
                        deadline,
                        flag,
                    );

                    if res.is_err() {
                        warn!("effect error: {:?}", res);
                    }

                    res
                })),
            };

            // Add to running effects
            self.running_effects.push(running_effect);

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

    /// Abort all effects
    pub fn clear_all(&mut self) {
        self.running_effects.clear();
    }
}
