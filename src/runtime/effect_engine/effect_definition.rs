//! Definition of the EffectDefinition type

use std::path::PathBuf;

/// Definition details of an effect
#[derive(Debug, Deserialize)]
pub struct EffectDefinition {
    /// Definition object
    definition: crate::servers::json::EffectDefinition,
    /// Full path to the script to run
    script: PathBuf,
}

impl EffectDefinition {
    /// Create a new effect definition
    ///
    /// # Parameters
    ///
    /// * `definition`: JSON definition of the effect
    /// * `base_path`: base path to resolve the full script path
    pub fn new(definition: crate::servers::json::EffectDefinition, mut base_path: PathBuf) -> Self {
        base_path.push(&definition.script);

        Self {
            definition,
            script: base_path,
        }
    }

    /// Get the name of this effect
    pub fn get_name(&self) -> &str {
        &self.definition.name
    }

    /// Get the path to the script for this effect
    pub fn get_script(&self) -> &PathBuf {
        &self.script
    }

    /// Get the default arguments for this effect
    pub fn get_args(&self) -> &serde_json::Value {
        &self.definition.args
    }

    /// Get the definition object for this effect
    pub fn get_definition(&self) -> &crate::servers::json::EffectDefinition {
        &self.definition
    }
}
