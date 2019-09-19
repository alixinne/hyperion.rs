//! Definition of the Host type

use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::config::ConfigHandle;
use crate::hyperion::{ServiceInputReceiver, ServiceInputSender};
use crate::image::Processor;
use crate::runtime::{Devices, EffectEngine, PriorityMuxer};

#[allow(missing_docs)]
mod errors {
    use error_chain::error_chain;

    error_chain! {
        links {
            Devices(crate::methods::MethodError, crate::methods::MethodErrorKind);
        }
    }
}

pub use errors::*;

/// Hyperion components host
pub struct Host {
    /// Device runtime data
    devices: Mutex<Devices>,
    /// Effect engine
    effect_engine: Mutex<EffectEngine>,
    /// Priority muxer
    priority_muxer: Mutex<PriorityMuxer>,
    /// Image processor
    image_processor: Mutex<Processor<f32>>,
    /// Service input sender
    service_input_sender: ServiceInputSender,
    /// Configuration handle
    config: ConfigHandle,
}

/// Handle to Hyperion components
#[derive(Clone)]
pub struct HostHandle(Option<Arc<Host>>);

impl HostHandle {
    /// Create a new host handle
    ///
    /// This handle will initially be empty, and should be later
    /// resolved by the Host when all components have been created.
    pub fn new() -> Self {
        Self(None)
    }

    /// Load the actual host handle
    ///
    /// # Parameters
    ///
    /// * `value`: allocated host handle
    fn replace(&mut self, value: Arc<Host>) {
        self.0.replace(value);
    }
}

impl Deref for HostHandle {
    type Target = Host;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("uninitialized host reference")
    }
}

impl From<Arc<Host>> for HostHandle {
    fn from(host: Arc<Host>) -> Self {
        Self(Some(host))
    }
}

impl Host {
    /// Build a new component host for Hyperion
    pub fn new(
        service_input_receiver: ServiceInputReceiver,
        service_input_sender: ServiceInputSender,
        config: ConfigHandle,
    ) -> Result<HostHandle> {
        // Create individual components
        let devices = Devices::new(config.clone())?;
        let effect_engine = EffectEngine::new(vec!["effects/".into()]);
        let priority_muxer = PriorityMuxer::new(service_input_receiver);

        // Create host object
        let host = Arc::new(Self {
            devices: Mutex::new(devices),
            effect_engine: Mutex::new(effect_engine),
            priority_muxer: Mutex::new(priority_muxer),
            image_processor: Mutex::new(Default::default()),
            service_input_sender,
            config,
        });

        // Update components with host object reference
        host.devices
            .lock()
            .unwrap()
            .get_host_mut()
            .replace(host.clone());

        host.priority_muxer
            .lock()
            .unwrap()
            .get_host_mut()
            .replace(host.clone());

        Ok(host.into())
    }

    /// Acquire reference to the effect engine
    pub fn get_effect_engine(&self) -> MutexGuard<EffectEngine> {
        self.effect_engine.lock().unwrap()
    }

    /// Acquire reference to the device runtime data
    pub fn get_devices(&self) -> MutexGuard<Devices> {
        self.devices.lock().unwrap()
    }

    /// Acquire reference to the priority muxer
    pub fn get_priority_muxer(&self) -> MutexGuard<PriorityMuxer> {
        self.priority_muxer.lock().unwrap()
    }

    /// Acquire reference to the image processor
    pub fn get_image_processor(&self) -> MutexGuard<Processor<f32>> {
        self.image_processor.lock().unwrap()
    }

    /// Get service input sender
    pub fn get_service_input_sender(&self) -> ServiceInputSender {
        self.service_input_sender.clone()
    }

    /// Get config handle
    pub fn get_config(&self) -> ConfigHandle {
        self.config.clone()
    }
}
