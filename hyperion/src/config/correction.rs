//! Definition of the Correction type

use crate::color::ColorPoint;

/// Default saturation gain
fn default_saturation() -> f32 {
    1.0
}

/// Default lightness gain
fn default_lightness() -> f32 {
    1.0
}

/// Default lightness threshold
fn default_threshold() -> f32 {
    0.0
}

/// Default RGB gamma
fn default_gamma() -> ColorPoint {
    ColorPoint::from((1.0, 1.0, 1.0))
}

/// Transform part of the color processing pipeline
#[derive(Debug, Serialize, Deserialize)]
pub struct Transform {
    /// Saturation gain
    #[serde(default = "default_saturation")]
    pub saturation: f32,
    /// Luminance gain
    #[serde(default = "default_lightness")]
    pub lightness: f32,
    /// Luminance threshold
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    /// RGB gamma
    #[serde(default = "default_gamma")]
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
            saturation: default_saturation(),
            lightness: default_lightness(),
            threshold: default_threshold(),
            gamma: default_gamma(),
        }
    }
}

/// Color correction settings
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Correction {
    /// Transform correction
    #[serde(default)]
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
