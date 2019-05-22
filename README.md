# hyperion.rs

[![Build Status](https://travis-ci.com/vtavernier/hyperion.rs.svg)](http://travis-ci.com/vtavernier/hyperion.rs) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT) [![Libraries.io for GitHub](https://img.shields.io/librariesio/github/vtavernier/hyperion.rs.svg)](https://libraries.io/vtavernier/hyperion.rs) [![Built with cargo-make](https://sagiegurari.github.io/cargo-make/assets/badges/cargo-make.svg)](https://sagiegurari.github.io/cargo-make)

`hyperion.rs` is a Rust implementation of the
[Hyperion](https://github.com/hyperion-project/hyperion) open-source ambient
lighting software.

* [`hyperiond/`](hyperiond): daemon program and usage instructions
* [`hyperion/`](hyperion): core functionality implementation

## Usage

Get the source for the project:

    git clone https://github.com/vtavernier/hyperion.rs.git
    cd hyperion.rs

Assuming the following `config.json`:

    {
        "devices": [
            {
                "name": "stdout script",
                "frequency": 1,
                "endpoint": {
                    "method": "stdout",
                    "target": {}
                },
                "leds": [
                    { "hscan" : { "minimum" : 0.8, "maximum" : 1.0 }, "vscan" : { "minimum" : 0.0, "maximum" : 1.0 } },
                    { "hscan" : { "minimum" : 0.5, "maximum" : 1.0 }, "vscan" : { "minimum" : 0.8, "maximum" : 1.0 } },
                    { "hscan" : { "minimum" : 0.0, "maximum" : 0.5 }, "vscan" : { "minimum" : 0.8, "maximum" : 1.0 } },
                    { "hscan" : { "minimum" : 0.0, "maximum" : 0.2 }, "vscan" : { "minimum" : 0.0, "maximum" : 1.0 } }
                ]
            }
        ]
    }

You can run the daemon using the following command:

    HYPERION_LOG=hyperion=debug cargo run -- -c config.json s --bind 0.0.0.0

The Android app can be used to send commands to this server, which should result
in updated colors in the output of the daemon.

## Status

This is still early work: the crate interface will probably change a lot, and no
binary releases will be provided until the core functionality is implemented.

Works:

* JSON server endpoint
* Protobuf server endpoint
* Commands:
  * Clear/ClearAll
  * Set solid color

Not implemented yet:

* Image to LED color (including black border)
* Effects
* Other LED device types
* LED color filtering

### Supported devices

This list details which devices are supported currently for outputting color data.

* Raw 8-bit RGB UDP
* Stdout (for debugging)

### Endpoint command status

This table summarizes the commands supported by the JSON and Protobuf interfaces
to the Hyperion server.

| Command     | JSON  | Protobuf |
| ----------- | ----- | -------- |
| Adjustment  | ❌     | N.A.     |
| Clear       | ✔ (1) | ✔ (1)    |
| ClearAll    | ✔     | ✔        |
| Correction  | ❌     | N.A.     |
| Effect      | ❌     | N.A.     |
| Image       | ❌     | ❌        |
| ServerInfo  | ❌     | N.A.     |
| Temperature | ❌     | N.A.     |
| Transform   | ❌     | N.A.     |

* (1): no support for the priority field
* N.A.: not defined for the protocol

## Authors

* [Vincent Tavernier](https://github.com/vtavernier)

## License

This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)
