//! Definition of the Method type

use crate::runtime::DeviceInstanceDataHandle;

/// A method for communicating with a device
pub trait Method {
    /// Write the current LED status to the target device
    ///
    /// # Parameters
    ///
    /// * `data`: handle to the device instance data
    fn write(&mut self, data: DeviceInstanceDataHandle);
}
