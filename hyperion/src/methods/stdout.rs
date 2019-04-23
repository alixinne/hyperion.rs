use super::{LedInstance, Method};

/// Dummy LED device which outputs updates to the standard output
#[derive(Default)]
pub struct Stdout {}

impl Stdout {
    pub fn new() -> Self {
        Self {}
    }
}

impl Method for Stdout {
    fn write(&self, leds: &[LedInstance]) {
        for led in leds {
            debug!("LED{} write({:?})", led.spec.index, led.current_color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdout_method() {
        let method: Box<dyn Method> = Box::new(Stdout::new());
        let leds = vec![LedInstance::default()];

        method.write(&leds[..]);
    }
}
