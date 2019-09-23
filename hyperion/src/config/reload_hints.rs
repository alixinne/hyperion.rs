//! ReloadHints type definition

bitflags! {
    /// Hyperion state reloading flags
    #[derive(Default)]
    pub struct ReloadHints: u32 {
        /// Generic information for a device changed
        const DEVICE_GENERIC   = 0b0000_0001;
        /// LED specifications for a device changed
        const DEVICE_LEDS      = 0b0000_0010;
        /// Idle settings for a device changed
        const DEVICE_IDLE      = 0b0000_0100;
        /// Endpoint details for a device changed
        const DEVICE_ENDPOINT  = 0b0000_1000;
        /// Filter details for a device changed
        const DEVICE_FILTER    = 0b0001_0000;
        /// Format details for a device changed
        const DEVICE_FORMAT    = 0b0010_0000;
        /// Frequency information for a device changed
        const DEVICE_FREQUENCY = 0b0100_0000;
        /// Latency information for a device changed
        const DEVICE_LATENCY   = 0b1000_0000;
    }
}
