use std::{path::Path, sync::Arc};

use thiserror::Error;

use super::instance::RuntimeMethods;

#[cfg(feature = "python")]
mod python;

#[derive(Debug, Error)]
pub enum ProviderError {
    #[cfg(feature = "python")]
    #[error(transparent)]
    Python(#[from] python::Error),
}

/// Trait for effect providers.
///
/// An effect provider is able to load an effect script and run it when given input parameters.
pub trait Provider: std::fmt::Debug + Send + Sync {
    /// Returns if this provider supports the given effect
    ///
    /// # Parameters
    ///
    /// * `script_path`: path to the script file describing this effect. This is the `script` field
    /// in the effect definition JSON.
    ///
    /// # Returns
    ///
    /// `true` if this provider can handle this script file, `false` otherwise.
    fn supports(&self, script_path: &str) -> bool;

    /// Run the given effect to completion in a blocking fashion
    ///
    /// # Parameters
    ///
    /// * `full_script_path`: resolved script path
    /// * `args`: arguments to the effect
    /// * `methods`: instance interaction methods
    fn run(
        &self,
        full_script_path: &Path,
        args: serde_json::Value,
        methods: Arc<dyn RuntimeMethods>,
    ) -> Result<(), ProviderError>;
}

#[derive(Debug)]
pub struct Providers {
    providers: Vec<Arc<dyn Provider>>,
}

impl Providers {
    pub fn new() -> Self {
        Self {
            providers: vec![
                #[cfg(feature = "python")]
                Arc::new(python::PythonProvider::new()),
            ],
        }
    }

    pub fn get(&self, script_path: &str) -> Option<Arc<dyn Provider>> {
        self.providers
            .iter()
            .find(|provider| provider.supports(script_path))
            .cloned()
    }
}
