//! Definition of the Configuration type

use super::{Correction, Device};

/// Configuration for an Hyperion instance
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    /// List of devices for this configuration
    pub devices: Vec<Device>,
    /// Image color correction
    #[serde(default)]
    pub color: Correction,
}

impl Configuration {
    /// Ensures the configuration is well-formed
    ///
    /// This will issue warnings for fields that need to be changed.
    pub fn sanitize(&mut self) {
        for device in &mut self.devices {
            device.sanitize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn deserialize_full_config() {
        let config: Configuration = serde_yaml::from_str(
            r#"
devices:
  - name: Stdout dummy
    endpoint:
      type: stdout
    leds:
      - hscan: { min: 0.0, max: 0.5 }
        vscan: { min: 0.0, max: 0.5 }
  - name: Remote UDP
    endpoint:
      type: udp
      address: 127.0.0.1:20446
    leds:
      - hscan: { min: 0.5, max: 1.0 }
        vscan: { min: 0.5, max: 1.0 }
        "#,
        )
        .unwrap();

        println!("{:?}", config);
    }
}
