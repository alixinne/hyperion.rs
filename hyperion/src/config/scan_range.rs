//! Definition of the ScanRange type

/// Floating-point range in a picture
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanRange {
    /// Start fraction (>= 0) of this range
    pub min: f32,
    /// End fraction (<= 1) of this range
    pub max: f32,
}

impl Default for ScanRange {
    fn default() -> Self {
        Self { min: 0.0, max: 1.0 }
    }
}
