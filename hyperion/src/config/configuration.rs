//! Definition of the Configuration type

use validator::Validate;

use super::{Correction, Device};

/// Configuration for an Hyperion instance
#[derive(Debug, Validate, Serialize, Deserialize)]
pub struct Configuration {
    /// List of devices for this configuration
    #[validate]
    pub devices: Vec<Device>,
    /// Image color correction
    #[serde(default)]
    #[validate]
    pub color: Correction,
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
