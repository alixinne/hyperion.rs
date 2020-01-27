//! Definition of the ByteRgb type

/// Packed RGB data from Python interface
#[repr(packed)]
pub struct ByteRgb {
    /// Red component
    pub r: u8,
    /// Green component
    pub g: u8,
    /// Blue component
    pub b: u8,
}
