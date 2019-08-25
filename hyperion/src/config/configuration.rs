//! Definition of the Configuration type

use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use validator::Validate;

use super::{Correction, Device};

/// Configuration for an Hyperion instance
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct Configuration {
    /// List of devices for this configuration
    #[validate]
    pub devices: Vec<Device>,
    /// Image color correction
    #[serde(default)]
    #[validate]
    pub color: Correction,
}

/// Handle to the shared configuration object
pub type ConfigurationHandle = Arc<RwLock<Configuration>>;

impl Configuration {
    /// Turn this configuration object into a shared handle
    pub fn into_handle(self) -> ConfigurationHandle {
        Arc::new(RwLock::new(self))
    }
}

/// Sub-configuration handle
#[derive(Clone)]
pub struct DeviceConfigurationHandle {
    /// Root configuration handle
    configuration: ConfigurationHandle,
    /// Device index
    device_index: usize,
}

impl DeviceConfigurationHandle {
    /// Create a new device configuration handle
    ///
    /// # Parameters
    ///
    /// * `configuration`: configuration to derive this config from
    /// * `device_index`: index to the device
    pub fn new(configuration: ConfigurationHandle, device_index: usize) -> Self {
        Self {
            configuration,
            device_index,
        }
    }

    /// Obtain a read handle to the target configuration
    pub fn read(
        &self,
    ) -> Result<
        DeviceConfigurationGuard<'_>,
        std::sync::PoisonError<std::sync::RwLockReadGuard<'_, Configuration>>,
    > {
        self.configuration
            .read()
            .map(|lock_guard| DeviceConfigurationGuard {
                lock_guard,
                handle: &self,
            })
    }
}

/// Device configuration read lock guard
pub struct DeviceConfigurationGuard<'a> {
    /// Lock guard
    lock_guard: RwLockReadGuard<'a, Configuration>,
    /// Source handle
    handle: &'a DeviceConfigurationHandle,
}

impl<'a> Deref for DeviceConfigurationGuard<'a> {
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
        let config: Configuration = serde_yaml::from_str(
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
