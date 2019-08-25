//! Definition of the Correction type

use validator::Validate;

use crate::color::ColorPoint;

/// Transform part of the color processing pipeline
#[derive(Debug, Validate, Serialize, Deserialize)]
#[serde(default)]
pub struct Transform {
    /// Saturation gain
    #[validate(range(min = 0.0))]
    pub saturation: f32,
    /// Luminance gain
    #[validate(range(min = 0.0))]
    pub lightness: f32,
    /// Luminance threshold
    #[validate(range(min = 0.0))]
    pub threshold: f32,
    /// RGB gamma
    // TODO: validate ColorPoint gamma values
    pub gamma: ColorPoint,
}

impl Transform {
    /// Apply color correction to the given color
    ///
    /// # Parameters
    ///
    /// * `color`: color to apply corrections to
    pub fn process(&self, mut color: ColorPoint) -> ColorPoint {
        color = color.sl_gain(self.saturation, self.lightness);
        color = color.l_threshold(self.threshold);
        color = color.rgb_gamma(self.gamma);

        color
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            saturation: 1.0,
            lightness: 1.0,
            threshold: 0.0,
            gamma: ColorPoint::from((1.0, 1.0, 1.0)),
        }
    }
}

/// Color correction settings
#[derive(Default, Validate, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Correction {
    /// Transform correction
    #[validate]
    pub transform: Transform,
}

impl Correction {
    /// Apply color correction to the given color
    ///
    /// # Parameters
    ///
    /// * `color`: color to apply corrections to
    pub fn process(&self, color: ColorPoint) -> ColorPoint {
        self.transform.process(color)
    }
}
