//! Definition of the Host type

use std::convert::TryFrom;
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
    devices: Option<Mutex<Devices>>,
    /// Effect engine
    effect_engine: Option<Mutex<EffectEngine>>,
    /// Priority muxer
    priority_muxer: Option<Mutex<PriorityMuxer>>,
    /// Image processor
    image_processor: Mutex<Processor<f32>>,
    /// Service input sender
    service_input_sender: ServiceInputSender,
    /// Configuration handle
    config: ConfigHandle,
}

/// Handle to Hyperion components
pub type HostHandle = Arc<Host>;

impl Host {
    /// Build a new component host for Hyperion
    pub fn new(
        service_input_receiver: ServiceInputReceiver,
        service_input_sender: ServiceInputSender,
        config: ConfigHandle,
    ) -> Result<HostHandle> {
        let host = Arc::new(Self {
            devices: None,
            effect_engine: None,
            priority_muxer: None,
            image_processor: Mutex::new(Default::default()),
            service_input_sender,
            config,
        });

        let devices_host = host.clone();
        let priority_muxer_host = host.clone();

        // TODO: Remove this horrible hack
        //
        // We don't want to lock the entire host structure which would
        // complicate access and have worse performance, but we still
        // need to initialize its members with the created components.
        unsafe {
            // Initialize devices
            std::ptr::write(
                std::mem::transmute(&host.devices),
                Some(Mutex::new(Devices::try_from(devices_host)?)),
            );

            // Initialize effect engine
            // TODO: Introduce parameter for effect path
            std::ptr::write(
                std::mem::transmute(&host.effect_engine),
                Some(Mutex::new(EffectEngine::new(vec!["effects/".into()]))),
            );

            // Initialize priority muxer
            std::ptr::write(
                std::mem::transmute(&host.priority_muxer),
                Some(Mutex::new(PriorityMuxer::new(
                    service_input_receiver,
                    priority_muxer_host,
                ))),
            );
        }

        Ok(host)
    }

    /// Acquire reference to the effect engine
    pub fn get_effect_engine(&self) -> MutexGuard<EffectEngine> {
        self.effect_engine.as_ref().unwrap().lock().unwrap()
    }

    /// Acquire reference to the device runtime data
    pub fn get_devices(&self) -> MutexGuard<Devices> {
        self.devices.as_ref().unwrap().lock().unwrap()
    }

    /// Acquire reference to the priority muxer
    pub fn get_priority_muxer(&self) -> MutexGuard<PriorityMuxer> {
        self.priority_muxer.as_ref().unwrap().lock().unwrap()
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
