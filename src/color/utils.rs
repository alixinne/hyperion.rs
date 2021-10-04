//! Color utilities

use palette::LinSrgb;

use crate::models::{Color, Color16};

const FACTOR: u16 = 65535 / 255;

pub fn color_to8(color: Color16) -> Color {
    let (r, g, b) = color.into_components();
    Color::new((r / FACTOR) as u8, (g / FACTOR) as u8, (b / FACTOR) as u8)
}

pub fn color_to16(color: Color) -> Color16 {
    let (r, g, b) = color.into_components();
    Color16::new(
        (r as u16) * FACTOR,
        (g as u16) * FACTOR,
        (b as u16) * FACTOR,
    )
}
