//! Configuration model definitions

mod color_format;
pub use color_format::*;

mod config;
pub use config::*;

mod config_handle;
pub use config_handle::*;

mod config_load_error;
pub use config_load_error::*;

mod correction;
pub use correction::*;

mod device;
pub use device::*;

mod endpoint;
pub use endpoint::*;

mod filter;
pub use filter::*;

mod idle_settings;
pub use idle_settings::*;

mod led;
pub use led::*;

mod reload_hints;
pub use reload_hints::*;

mod scan_range;
pub use scan_range::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use validator::Validate;

    #[test]
    fn sanitize_frequency() {
        let device = Device {
            enabled: true,
            name: "test".into(),
            format: ColorFormat::default(),
            endpoint: Endpoint::Stdout { bits: 8 },
            leds: Vec::new(),
            frequency: -2.0,
            idle: IdleSettings::default(),
            filter: Filter::default(),
        };

        assert!(device.validate().is_err());
    }

    #[test]
    fn sanitize_idle() {
        let device = Device {
            enabled: true,
            name: "test".into(),
            format: ColorFormat::default(),
            endpoint: Endpoint::Stdout { bits: 8 },
            leds: Vec::new(),
            frequency: 1.0,
            idle: IdleSettings {
                delay: Duration::from_millis(5),
                ..Default::default()
            },
            filter: Filter::default(),
        };

        assert!(device.validate().is_err());
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
