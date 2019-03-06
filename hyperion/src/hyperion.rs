//! Definition of the Hyperion data model

/// Definition of the Led type
mod led;
pub use led::*;

/// Definition of the Device type
mod device;
pub use device::*;

/// A configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    devices: Vec<Device>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn deserialize_full_config() {
        let config: Configuration = serde_json::from_str(r#"
{
    "devices": [
        {
            "name": "Stdout dummy",
            "endpoint": {
                "method": "stdout"
            },
            "leds": [ 
                { "index": 0, "hscan": { "minimum": 0.0, "maximum": 0.5 },
                              "vscan": { "minimum": 0.0, "maximum": 0.5 } }
            ]
        },
        {
            "name": "Remote UDP",
            "endpoint": {
                "method": "udp",
                "target": {
                    "address": "127.0.0.1:20446"
                }
            },
            "leds": [ 
                { "index": 0, "hscan": { "minimum": 0.5, "maximum": 1.0 },
                              "vscan": { "minimum": 0.5, "maximum": 1.0 } }
            ]
        }
    ]
}
        "#).unwrap();

        println!("{:?}", config);
    }
}


