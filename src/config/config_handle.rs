//! ConfigHandle type definition

use std::ops::Deref;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use super::{Config, Device};

/// Handle to the shared config object
pub type ConfigHandle = Arc<RwLock<Config>>;

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
