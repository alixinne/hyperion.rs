//! FormattedColor type definition

use super::DeviceColor;

use crate::config::ColorFormat;

/// A device color in a given format
pub struct FormattedColor<'a> {
    /// Color format
    format: &'a ColorFormat,
    /// Device color
    color: &'a DeviceColor,
}

impl<'a> FormattedColor<'a> {
    /// Create a new formatted device color
    ///
    /// # Parameters
    ///
    /// * `color`: device color to format
    /// * `format`: color format to apply
    pub fn new(color: &'a DeviceColor, format: &'a ColorFormat) -> Self {
        Self { format, color }
    }
}

impl<'a> IntoIterator for FormattedColor<'a> {
    type Item = f32;
    type IntoIter = FormattedColorIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FormattedColorIter {
            formatted_color: self,
            pos: 0,
        }
    }
}

/// Formatted color component iterator
pub struct FormattedColorIter<'a> {
    formatted_color: FormattedColor<'a>,
    pos: usize,
}

impl<'a> Iterator for FormattedColorIter<'a> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let components = self.formatted_color.format.components();

        // Have we exhausted all expected components?
        if self.pos == components {
            return None;
        }

        // Get index and increment
        let pos = self.pos;
        self.pos += 1;

        // Get the next component
        Some(
            self.formatted_color
                .format
                .order()
                .chars()
                .skip(pos)
                .next()
                .map(|ch| self.formatted_color.color.get_component(ch))
                .unwrap_or(None) // If string too short
                .unwrap_or(0.), // If component not found
        )
    }
}
