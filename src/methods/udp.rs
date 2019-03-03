use super::{Method, Led};

/// LED device that forwards raw RGBW data as UDP packets
pub struct Udp {
}

impl Udp {
    pub fn new(address: String) -> Self {
        Self {}
    }
}

impl Method for Udp {
    fn write(&self, leds: &[Led]) {
        for led in leds {
            println!("LED{} UDP write({:?})", led.index(), led.current_color());
        }
    }
}
