use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::fs;
use tracing::error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectDefinition {
    /// Friendly name of the effect
    pub name: String,
    /// Path to the effect definition file
    #[serde(skip)]
    pub file: PathBuf,
    /// Path to the effect script
    pub script: String,
    /// Arguments to the effect script
    #[serde(default)]
    pub args: serde_json::Value,
    /// Path this definition is located in
    #[serde(skip)]
    base_path: Arc<PathBuf>,
}

#[derive(Debug, Error)]
pub enum EffectDefinitionError {
    /// i/o error
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Invalid effect Definitionification path
    #[error("invalid effect Definitionification path")]
    InvalidPath,
    /// JSON error
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl EffectDefinition {
    pub async fn read_dir(path: impl AsRef<Path>) -> Result<Vec<Self>, EffectDefinitionError> {
        let base_path = Arc::new(path.as_ref().to_owned());
        let mut definitions = Vec::new();

        let mut read_dir = fs::read_dir(path.as_ref()).await?;
        loop {
            let result = read_dir.next_entry().await;
            match result {
                Ok(None) => {
                    break;
                }
                Ok(Some(entry)) => {
                    let path = entry.path();
                    if path.extension().and_then(std::ffi::OsStr::to_str) != Some("json") {
                        continue;
                    }

                    match Self::from_file(&path, base_path.clone()).await {
                        Ok(definition) => {
                            definitions.push(definition);
                        }
                        Err(err) => {
                            error!(path = %path.display(), error = %err, "error reading effect definition");
                        }
                    }
                }
                Err(err) => {
                    error!(error = %err, "error reading effect definition directory");
                }
            }
        }

        Ok(definitions)
    }

    pub async fn read_file(path: impl AsRef<Path>) -> Result<Self, EffectDefinitionError> {
        let path = path.as_ref();

        Self::from_file(
            path,
            path.parent()
                .ok_or(EffectDefinitionError::InvalidPath)?
                .to_owned()
                .into(),
        )
        .await
    }

    async fn from_file(
        path: &Path,
        base_path: Arc<PathBuf>,
    ) -> Result<Self, EffectDefinitionError> {
        // Read file contents
        let json = fs::read(path).await?;

        // Parse
        let mut this: Self = serde_json::from_slice(&json)?;

        // Set path
        this.file = path
            .strip_prefix(&*base_path)
            .map(|path| path.to_owned())
            .unwrap_or_else(|_| path.to_owned());

        // Set base path
        this.base_path = base_path;

        Ok(this)
    }

    pub fn script_path(&self) -> Result<PathBuf, EffectDefinitionError> {
        let mut result = (*self.base_path).clone();
        let subpath = PathBuf::from(&self.script);

        // Prevent traversal attacks
        for component in subpath.components() {
            match component {
                std::path::Component::CurDir => {
                    // Ignore
                }
                std::path::Component::Normal(component) => {
                    result.push(component);
                }
                _ => {
                    return Err(EffectDefinitionError::InvalidPath);
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn load_builtin_effects() {
        let paths = crate::global::Paths::new(None).expect("failed to load paths");
        let effects_root = paths.resolve_path("$SYSTEM/effects");

        // All JSON files in this directory should parse as valid effects
        let json_files = std::fs::read_dir(&effects_root)
            .expect("effects directory not found")
            .filter(|res| {
                res.as_ref()
                    .map(|entry| {
                        entry.path().extension().and_then(std::ffi::OsStr::to_str) == Some("json")
                    })
                    .unwrap_or(false)
            })
            .count();

        let parsed = EffectDefinition::read_dir(&effects_root)
            .await
            .expect("read_dir failed");

        eprintln!("{:#?}", parsed);
        assert_eq!(parsed.len(), json_files);
    }
}
