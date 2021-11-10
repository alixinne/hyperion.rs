use crate::models::{Color16, Led};

use super::Image;

#[derive(Debug, Default)]
pub struct Reducer {
    spec: Vec<LedSpec>,
    spec_width: u16,
    spec_height: u16,
}

#[derive(Debug)]
struct LedSpec {
    lxmin: u16,
    lxmax: u16,
    lymin: u16,
    lymax: u16,
}

impl LedSpec {
    pub fn new(spec: &Led, width: u16, height: u16, fwidth: f32, fheight: f32) -> Self {
        let lxmin = spec.hmin * fwidth;
        let lxmax = spec.hmax * fwidth;
        let lymin = spec.vmin * fheight;
        let lymax = spec.vmax * fheight;

        Self {
            lxmin: lxmin.floor() as u16,
            lxmax: (lxmax.ceil() as u16).min(width - 1),
            lymin: lymin.floor() as u16,
            lymax: (lymax.ceil() as u16).min(height - 1),
        }
    }
}

impl Reducer {
    pub fn reset(&mut self, width: u16, height: u16, leds: &[Led]) {
        self.spec_width = width;
        self.spec_height = height;

        let fwidth = width as f32;
        let fheight = height as f32;

        self.spec.clear();
        self.spec.reserve(leds.len());

        for spec in leds.iter() {
            self.spec
                .push(LedSpec::new(spec, width, height, fwidth, fheight));
        }
    }

    pub fn reduce(&mut self, image: &impl Image, leds: &[Led], color_data: &mut [Color16]) {
        let width = image.width();
        let height = image.height();

        if self.spec_width != width || self.spec_height != height || self.spec.len() != leds.len() {
            self.reset(width, height, leds);
        }

        for (spec, value) in self.spec.iter().zip(color_data.iter_mut()) {
            let mut r_acc = 0u64;
            let mut g_acc = 0u64;
            let mut b_acc = 0u64;
            let mut cnt = 0u64;

            for y in spec.lymin..=spec.lymax {
                for x in spec.lxmin..=spec.lxmax {
                    // Safety: x (resp. y) are necessarily in 0..width (resp. 0..height)
                    let rgb = unsafe { image.color_at_unchecked(x as _, y as _) };
                    let area = 255;

                    let (r, g, b) = rgb.into_components();
                    r_acc += (r as u64 * 255) * area;
                    g_acc += (g as u64 * 255) * area;
                    b_acc += (b as u64 * 255) * area;
                    cnt += area;
                }
            }

            *value = Color16::new(
                ((r_acc / cnt.max(1)) * 65535 / (255 * 255)).min(u16::MAX as _) as u16,
                ((g_acc / cnt.max(1)) * 65535 / (255 * 255)).min(u16::MAX as _) as u16,
                ((b_acc / cnt.max(1)) * 65535 / (255 * 255)).min(u16::MAX as _) as u16,
            );
        }
    }
}
