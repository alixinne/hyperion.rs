//! FormattedColor type definition

use super::DeviceColor;

use crate::config::ColorFormat;

/// A device color in a given format
pub struct FormattedColor(Vec<(f32, char)>);

impl FormattedColor {
    /// Create a new formatted device color
    ///
    /// # Parameters
    ///
    /// * `color`: device color to format
    /// * `format`: color format to apply
    pub fn new(color: &DeviceColor, format: &ColorFormat) -> Self {
        Self(
            format
                .order()
                .chars()
                .map(|ch| (color.get_component(ch), ch))
                .map(|(next_val, next_ch)| (next_val.unwrap_or(0.), next_ch))
                .collect(),
        )
    }
}

impl IntoIterator for FormattedColor {
    type Item = (f32, char);
    type IntoIter = <Vec<(f32, char)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
