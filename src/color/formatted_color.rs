//! FormattedColor type definition

use super::DeviceColor;

use crate::config::ColorFormat;

/// A device color in a given format
#[derive(Debug)]
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

    /// Get the number of components for this formatted color
    pub fn components(&self) -> usize {
        self.0.len()
    }

    /// Obtain an iterator the components of this formatted color
    pub fn iter(&self) -> impl Iterator<Item = &(f32, char)> {
        self.0.iter()
    }
}

impl IntoIterator for FormattedColor {
    type Item = (f32, char);
    type IntoIter = <Vec<(f32, char)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
