//! Definition of the LedInstance type

use crate::config::Led;

/// Instance of a LED at runtime
///
/// Combines the specification of the LED coverage of the screen area plus
/// its current state.
#[derive(Debug, Default)]
pub struct LedInstance {
    pub spec: Led,
    pub current_color: palette::LinSrgb,
}

impl From<Led> for LedInstance {
    /// Create a new LedInstance from a LED specification
    /// 
    /// # Parameters
    /// 
    /// * `led`: LED specification
    fn from(led: Led) -> Self {
        Self {
            spec: led,
            current_color: Default::default()
        }
    }
}
