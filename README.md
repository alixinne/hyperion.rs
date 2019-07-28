# [hyperion.rs](https://github.com/vtavernier/hyperion.rs)

[![Build Status](https://travis-ci.com/vtavernier/hyperion.rs.svg)](http://travis-ci.com/vtavernier/hyperion.rs)
[![GitHub release](https://img.shields.io/github/release/vtavernier/hyperion.rs)](https://github.com/vtavernier/hyperion.rs/releases)
[![Codacy Badge](https://api.codacy.com/project/badge/Grade/9a0bff1adfc84e1d8c72fcc136328629)](https://www.codacy.com/app/vtavernier/hyperion.rs?utm_source=github.com&amp;utm_medium=referral&amp;utm_content=vtavernier/hyperion.rs&amp;utm_campaign=Badge_Grade)
[![Documentation](https://img.shields.io/badge/docs-master-blue.svg)](https://vtavernier.github.io/hyperion.rs/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Libraries.io for GitHub](https://img.shields.io/librariesio/github/vtavernier/hyperion.rs.svg)](https://libraries.io/github/vtavernier/hyperion.rs)
[![Built with cargo-make](https://sagiegurari.github.io/cargo-make/assets/badges/cargo-make.svg)](https://sagiegurari.github.io/cargo-make)

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

```bash
git clone https://github.com/vtavernier/hyperion.rs.git
cd hyperion.rs
```

Assuming the following `config.yml`:

```yaml
devices:
  - name: Raw UDP device
    frequency: 30 # Hz
    idle:
      delay: 5s   # Consider device idle after 5s
      holds: true # This device holds the last command forever
      retries: 3  # Unreliable device, retry sending packets 3 times during idle updates
    endpoint:
      type: udp
      address: 192.168.0.27:19446 # Can also use hostnames, e.g. device.local:19446
    leds:
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
```

You can run the daemon using the following commands:

```bash
# Build and run
HYPERION_LOG=hyperion=debug cargo run -- -c config.yml s --bind 0.0.0.0

# Run release version (without cargo)
HYPERION_LOG=hyperion=debug ./target/release/hyperiond -c config.yml s --bind 0.0.0.0
```

The Android app can be used to send commands to this server, which should result
in updated colors in the output of the daemon.

If you have [cargo-make](https://github.com/sagiegurari/cargo-make) installed,
you can use this command to build a deployable archive of hyperion.rs:

```bash
# Build hyperion.rs-release.tar.gz
cargo make --no-workspace package-release
```

## Status

This is still early work: the crate interface will probably change a lot and the
configuration format might be updated in breaking ways.

Not implemented yet:

* Effects
* Other LED device types

### Supported devices

This list details which devices are supported currently for outputting color data.

* Raw 8-bit RGB UDP
* Stdout (for debugging)

Currently, both RGB and RGB+W LED arrangements are supported. The white component
is computed from the RGB commands sent to the daemon, based on the white point
(color temperature) of both the devices' RGB and W LEDs. This means that the W
LED will be used to produce as much as possible of its "target" white, while the
RGB LED will be used for colors, while also having its white point corrected.
This requires estimating (or getting from the specs) the color temperature of
the various LEDs in the device.

RGBCW (RGB + Cold white + Warm white) is not supported yet and only the RGB LED
will be used.

### Endpoint command status

This table summarizes the commands supported by the JSON and Protobuf interfaces
to the Hyperion server.

| Command     | JSON  | Protobuf |
| ----------- | ----- | -------- |
| Adjustment  | ❌     | N.A.     |
| Clear       | ✔     | ✔        |
| ClearAll    | ✔     | ✔        |
| Color       | ✔     | ✔        |
| Correction  | ❌     | N.A.     |
| Effect      | ❌     | N.A.     |
| Image       | ✔     | ✔        |
| ServerInfo  | ❌     | N.A.     |
| Temperature | ❌     | N.A.     |
| Transform   | ❌     | N.A.     |

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

The following temporal filters are available:

* `nearest`: no filtering. The latest image sample is used as current color.
  Lowest image to LED latency, but sensitive to noise.
* `linear`: linear filtering. The LED color reaches the latest image sample
  linearly in `1/f` seconds (where `f` is the filter frequency). Introduces
  latency proportional to the filter period.

### Grabbing

Grabbing is not planned yet for hyperion.rs.

### Web configuration

Web configuration of the hyperion.rs is outside of the scope of this project.

## Authors

* [Vincent Tavernier](https://github.com/vtavernier)

## License

This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)
