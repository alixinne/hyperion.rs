//! Definition of the ScanRange type

use validator::{Validate, ValidationError};

/// Floating-point range in a picture
#[derive(Debug, Validate, Serialize, Deserialize, Clone)]
#[validate(schema(function = "validate_scan_range", message = "invalid range"))]
pub struct ScanRange {
    /// Start fraction (>= 0) of this range
    #[validate(range(min = 0.0, max = 1.0))]
    pub min: f32,
    /// End fraction (<= 1) of this range
    #[validate(range(min = 0.0, max = 1.0))]
    pub max: f32,
}

/// Validate the bounds of a scan range
fn validate_scan_range(scan_range: &ScanRange) -> Result<(), ValidationError> {
    if scan_range.min > scan_range.max {
        return Err(ValidationError::new("invalid_range"));
    }

    Ok(())
}

impl Default for ScanRange {
    fn default() -> Self {
        Self { min: 0.0, max: 1.0 }
    }
}
