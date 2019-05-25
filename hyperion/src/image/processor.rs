use crate::hyperion::LedInstance;
use std::cmp::min;

#[derive(Default, Clone)]
struct Pixel {
    color: [f32; 3],
    count: usize,
}

impl Pixel {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn sample(&mut self, (r, g, b): (u8, u8, u8)) {
        self.color[0] += f32::from(r) / 255.0;
        self.color[1] += f32::from(g) / 255.0;
        self.color[2] += f32::from(b) / 255.0;
        self.count += 1;
    }

    pub fn mean(&self) -> palette::LinSrgb {
        palette::LinSrgb::from_components((
            self.color[0] / self.count as f32,
            self.color[1] / self.count as f32,
            self.color[2] / self.count as f32,
        ))
    }
}

/// Raw image data processor
pub struct Processor {
    pub width: usize,
    pub height: usize,
    led_map: Vec<Vec<(usize, usize, usize)>>,
    color_map: Vec<Pixel>,
}

impl Processor {
    /// Create a new image data processor
    ///
    /// # Parameters
    ///
    /// * `width`: width of the incoming images in pixels
    /// * `height`: height of the incoming images in pixels
    /// * `leds`: LED specification for target devices
    pub fn new<'a>(
        width: u32,
        height: u32,
        leds: impl Iterator<Item = (usize, &'a LedInstance, usize)>,
    ) -> Self {
        let width = width as usize;
        let height = height as usize;

        // Initialize led map data structure
        let mut led_map = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            led_map.push(Vec::new());
        }

        // Add leds whose area overlap the current pixel
        let mut count = 0;
        for (index, led) in leds.enumerate() {
            for j in min(
                height - 1,
                (led.1.spec.vscan.min * height as f32).floor() as usize,
            )
                ..min(
                    height,
                    (led.1.spec.vscan.max * height as f32).ceil() as usize,
                )
            {
                // Vertical scan range
                let y_min = j as f32 / height as f32;
                let y_max = (j + 1) as f32 / height as f32;

                for i in min(
                    width - 1,
                    (led.1.spec.hscan.min * width as f32).floor() as usize,
                )
                    ..min(width, (led.1.spec.hscan.max * width as f32).ceil() as usize)
                {
                    // Horizontal scan range
                    let x_min = i as f32 / width as f32;
                    let x_max = (i + 1) as f32 / width as f32;

                    let map_index = j * width + i;

                    if led.1.spec.hscan.min < x_max
                        && led.1.spec.hscan.max >= x_min
                        && led.1.spec.vscan.min < y_max
                        && led.1.spec.vscan.max >= y_min
                    {
                        led_map[map_index].push((index, led.0, led.2));
                    }
                }
            }

            count += 1;
        }

        // Color map for computation
        let color_map = vec![Pixel::default(); count];

        Self {
            width,
            height,
            led_map,
            color_map,
        }
    }

    /// Checks if this image processor is set up to process images of the given size
    ///
    /// # Parameters
    ///
    /// * `width`: width of the image data
    /// * `height`: height of the image data
    ///
    /// # Return value
    ///
    /// `true` if this image process supports this size, `false` otherwise
    pub fn matches(&self, width: u32, height: u32) -> bool {
        let width = width as usize;
        let height = height as usize;

        self.width == width && self.height == height
    }

    /// Process incoming image data into led colors
    ///
    /// # Parameters
    ///
    /// * `led_setter`: callback to update LED colors
    /// * `data`: raw 8-bit RGB image data
    /// * `width`: width of the image data
    /// * `height`: height of the image data
    pub fn process_image(
        &mut self,
        mut led_setter: impl FnMut((usize, usize), palette::LinSrgb),
        data: &[u8],
        width: u32,
        height: u32,
    ) {
        // Check that this processor has the right size
        assert!(self.matches(width, height));

        let width = width as usize;
        let height = height as usize;

        // Check that the image is the right size
        assert!(width * height * 3 == data.len());

        // Reset all colors
        for color in &mut self.color_map {
            color.reset();
        }

        // TODO: check image data ordering
        for j in 0..height {
            for i in 0..width {
                let map_idx = j * width + i;
                let image_idx = map_idx * 3;
                let rgb = (data[image_idx], data[image_idx + 1], data[image_idx + 2]);

                for (pixel_idx, _device_idx, _led_idx) in &self.led_map[map_idx] {
                    self.color_map[*pixel_idx].sample(rgb);
                }
            }
        }

        // Compute mean and assign to led instances
        for pixel in self.led_map.iter() {
            for (pixel_idx, device_idx, led_idx) in pixel.iter() {
                led_setter((*device_idx, *led_idx), self.color_map[*pixel_idx].mean());
            }
        }
    }
}
