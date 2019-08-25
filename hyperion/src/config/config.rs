//! Definition of the Config type

use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use validator::Validate;

use super::{ConfigHandle, ConfigLoadError, Correction, Device};

/// Config for an Hyperion instance
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct Config {
    /// Path this config was loaded from
    #[serde(skip)]
    path: PathBuf,
    /// List of devices for this config
    #[validate]
    pub devices: Vec<Device>,
    /// Image color correction
    #[serde(default)]
    #[validate]
    pub color: Correction,
}

impl Config {
    /// Read the configuration from a file
    ///
    /// # Parameters
    ///
    /// * `path`: path to the configuration to load
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Config, ConfigLoadError> {
        let src_path = path.as_ref().to_path_buf();

        // Open file and create reader
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut config: Self = serde_yaml::from_reader(reader)?;
        config.path = src_path;
        config.validate()?;

        Ok(config)
    }

    /// Turn this config object into a shared handle
    pub fn into_handle(self) -> ConfigHandle {
        Arc::new(RwLock::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn deserialize_full_config() {
        let config: Config = serde_yaml::from_str(
            r#"
devices:
  - name: Stdout dummy
    endpoint:
      type: stdout
    leds:
      - hscan: { min: 0.0, max: 0.5 }
        vscan: { min: 0.0, max: 0.5 }
  - name: Remote UDP
    endpoint:
      type: udp
      address: 127.0.0.1:20446
    leds:
      - hscan: { min: 0.5, max: 1.0 }
        vscan: { min: 0.5, max: 1.0 }
        "#,
        )
        .unwrap();

        println!("{:?}", config);
    }
}
