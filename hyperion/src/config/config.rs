//! Definition of the Config type

use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use validator::Validate;

use super::{Correction, Device};

/// Config loading error
#[derive(Debug, Fail)]
pub enum ConfigLoadError {
    /// I/O error
    #[fail(display = "an i/o error occurred: {}", 0)]
    IoError(std::io::Error),
    /// Deserialization error
    #[fail(display = "invalid syntax: {}", 0)]
    InvalidSyntax(serde_yaml::Error),
    /// Validator error
    #[fail(display = "failed to validate config: {}", 0)]
    InvalidConfig(validator::ValidationErrors),
}

impl From<std::io::Error> for ConfigLoadError {
    fn from(error: std::io::Error) -> Self {
        ConfigLoadError::IoError(error)
    }
}

impl From<serde_yaml::Error> for ConfigLoadError {
    fn from(error: serde_yaml::Error) -> Self {
        ConfigLoadError::InvalidSyntax(error)
    }
}

impl From<validator::ValidationErrors> for ConfigLoadError {
    fn from(error: validator::ValidationErrors) -> Self {
        ConfigLoadError::InvalidConfig(error)
    }
}

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

/// Handle to the shared config object
pub type ConfigHandle = Arc<RwLock<Config>>;

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

/// Sub-configuration handle
#[derive(Clone)]
pub struct DeviceConfigHandle {
    /// Root configuration handle
    config: ConfigHandle,
    /// Device index
    device_index: usize,
}

impl DeviceConfigHandle {
    /// Create a new device configuration handle
    ///
    /// # Parameters
    ///
    /// * `config`: configuration to derive this config from
    /// * `device_index`: index to the device
    pub fn new(config: ConfigHandle, device_index: usize) -> Self {
        Self {
            config,
            device_index,
        }
    }

    /// Obtain a read handle to the target config
    pub fn read(
        &self,
    ) -> Result<DeviceConfigGuard<'_>, std::sync::PoisonError<std::sync::RwLockReadGuard<'_, Config>>>
    {
        self.config.read().map(|lock_guard| DeviceConfigGuard {
            lock_guard,
            handle: &self,
        })
    }
}

/// Device config read lock guard
pub struct DeviceConfigGuard<'a> {
    /// Lock guard
    lock_guard: RwLockReadGuard<'a, Config>,
    /// Source handle
    handle: &'a DeviceConfigHandle,
}

impl<'a> Deref for DeviceConfigGuard<'a> {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.lock_guard.devices[self.handle.device_index]
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
