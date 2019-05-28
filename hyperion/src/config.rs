mod configuration;
pub use configuration::*;

mod device;
pub use device::*;

mod endpoint;
pub use endpoint::*;

mod idle_settings;
pub use idle_settings::*;

mod led;
pub use led::*;

mod scan_range;
pub use scan_range::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn sanitize_frequency() {
        let mut device = Device {
            name: "test".into(),
            endpoint: Endpoint::Stdout { bits: 8 },
            leds: Vec::new(),
            frequency: -2.0,
            idle: IdleSettings::default(),
        };

        device.sanitize();
        assert!(device.frequency >= 1.0f64 / 3600f64);
    }

    #[test]
    fn sanitize_idle() {
        let mut device = Device {
            name: "test".into(),
            endpoint: Endpoint::Stdout { bits: 8 },
            leds: Vec::new(),
            frequency: 1.0,
            idle: IdleSettings {
                delay: Duration::from_millis(5),
                .. Default::default()
            },
        };

        device.sanitize();
        assert!(device.idle.delay > Duration::from_millis((1_000f64 / device.frequency) as u64));
    }

    #[test]
    fn serialize_udp_endpoint() {
        let endpoint = Endpoint::Udp {
            address: "127.0.0.1:19446".into(),
        };
        println!(
            "udp endpoint: {}",
            serde_yaml::to_string(&endpoint).unwrap()
        );
    }
}
