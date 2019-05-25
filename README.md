# hyperion.rs

[![Build Status](https://travis-ci.com/vtavernier/hyperion.rs.svg)](http://travis-ci.com/vtavernier/hyperion.rs) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT) [![Libraries.io for GitHub](https://img.shields.io/librariesio/github/vtavernier/hyperion.rs.svg)](https://libraries.io/github/vtavernier/hyperion.rs) [![Built with cargo-make](https://sagiegurari.github.io/cargo-make/assets/badges/cargo-make.svg)](https://sagiegurari.github.io/cargo-make)

`hyperion.rs` is a Rust implementation of the
[Hyperion](https://github.com/hyperion-project/hyperion) open-source ambient
lighting software.

* [`hyperiond/`](hyperiond): daemon program and usage instructions
* [`hyperion/`](hyperion): core functionality implementation

## Requirements

* [Rust stable](https://rustup.rs/)
* [protobuf compiler](https://github.com/protocolbuffers/protobuf/releases), `protobuf-compiler` on Debian distributions

## Usage

Get the source for the project:

    git clone https://github.com/vtavernier/hyperion.rs.git
    cd hyperion.rs

Assuming the following `config.yml`:

    devices:
      - name: stdout script
        frequency: 1 # Hz
        endpoint:
          method: stdout
          target:
            path: scripts/methods/stdout.lua
        leds: &1
          # When facing the screen
          #  * hscan ranges from 0 (left) to 1 (right)
          #  * vscan ranges from 0 (top) to 1 (bottom)
          - hscan: { min: 0.8, max: 1.0 }
            vscan: { min: 0.0, max: 1.0 }
          - hscan: { min: 0.5, max: 1.0 }
            vscan: { min: 0.0, max: 0.2 }
          - hscan: { min: 0.0, max: 0.5 }
            vscan: { min: 0.0, max: 0.2 }
          - hscan: { min: 0.0, max: 0.2 }
            vscan: { min: 0.0, max: 1.0 }

You can run the daemon using the following command:

    HYPERION_LOG=hyperion=debug cargo run -- -c config.yml s --bind 0.0.0.0

The Android app can be used to send commands to this server, which should result
in updated colors in the output of the daemon.

## Status

This is still early work: the crate interface will probably change a lot, and no
binary releases will be provided until the core functionality is implemented.

Not implemented yet:

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
| Color       | ✔ (2) | ✔ (2)    |
| Correction  | ❌     | N.A.     |
| Effect      | ❌     | N.A.     |
| Image       | ✔ (2) | ✔ (2)    |
| ServerInfo  | ❌     | N.A.     |
| Temperature | ❌     | N.A.     |
| Transform   | ❌     | N.A.     |

* (1): no support for the priority field
* (2): no support for the priority and duration fields
* N.A.: not defined for the protocol

### Image processing

The black border detector has not been implemented yet.

### Color processing

The Hyperion daemon is responsible for processing the incoming image data into
LED colors. This requires processing and transforming colors. The following parts
of the color processing pipeline have been implemented:

_(none)_

The following parts have to be implemented (they are listed in order of application
in the computation):

* Transform (saturation gain, luminance gain, luminance minimum + RGB per-channel
  threshold and gamma)
* Adjustment (RGB -> RGB mapping matrix)
* Temperature (RGB per-channel multiplication)

### Effects

Effects in hyperion.rs will be implemented as Lua scripts with a specific API to
interact with hyperiond. This project uses the Lua interpreter as it is lighter
to embed than the Python interpreter, and the language differences should not
matter much for writing effect code.

Effect support is under development.

### Smoothing

Temporal smoothing is not implemented yet.

### Grabbing

Grabbing is not planned yet for hyperion.rs.

### Web configuration

Web configuration of the hyperion.rs is outside of the scope of this project.

## Authors

* [Vincent Tavernier](https://github.com/vtavernier)

## License

This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)
