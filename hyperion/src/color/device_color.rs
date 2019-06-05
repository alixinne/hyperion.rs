//! DeviceColor type definition

use super::FormattedColor;

use crate::config;

/// Physical device color
pub enum DeviceColor {
    /// RGB LEDs
    Rgb {
        /// Red
        r: f32,
        /// Green
        g: f32,
        /// Blue
        b: f32,
    },
    /// RGB+White LEDs
    Rgbw {
        /// Red
        r: f32,
        /// Green
        g: f32,
        /// Blue
        b: f32,
        /// White
        w: f32,
    },
    /// RGB+Cold white+Warm white LEDs
    Rgbcw {
        /// Red
        r: f32,
        /// Green
        g: f32,
        /// Blue
        b: f32,
        /// Cold white
        c: f32,
        /// Warm white
        w: f32,
    },
}

impl DeviceColor {
    /// Return a component by its name
    ///
    /// # Parameters
    ///
    /// * `component_name`: name of the component to return
    pub fn get_component(&self, component_name: char) -> Option<f32> {
        match component_name {
            'r' | 'R' => match *self {
                DeviceColor::Rgb { r, .. } => Some(r),
                DeviceColor::Rgbw { r, .. } => Some(r),
                DeviceColor::Rgbcw { r, .. } => Some(r),
            },
            'g' | 'G' => match *self {
                DeviceColor::Rgb { g, .. } => Some(g),
                DeviceColor::Rgbw { g, .. } => Some(g),
                DeviceColor::Rgbcw { g, .. } => Some(g),
            },
            'b' | 'B' => match *self {
                DeviceColor::Rgb { b, .. } => Some(b),
                DeviceColor::Rgbw { b, .. } => Some(b),
                DeviceColor::Rgbcw { b, .. } => Some(b),
            },
            'w' | 'W' => match *self {
                DeviceColor::Rgbw { w, .. } => Some(w),
                DeviceColor::Rgbcw { w, .. } => Some(w),
                _ => None,
            },
            'c' | 'C' => match *self {
                DeviceColor::Rgbcw { c, .. } => Some(c),
                _ => None,
            },
            _ => None,
        }
    }

    /// Format this device color
    ///
    /// # Parameters
    ///
    /// * `format`: color format to use
    pub fn format<'a>(&'a self, format: &'a config::ColorFormat) -> FormattedColor<'a> {
        FormattedColor::new(self, format)
    }
}
