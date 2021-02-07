# [hyperion.rs](https://github.com/vtavernier/hyperion.rs)

[![Build](https://github.com/vtavernier/hyperion.rs/workflows/build/badge.svg)](https://github.com/vtavernier/hyperion.rs/actions)
[![Docs](https://img.shields.io/badge/docs-master-blue)](https://vtavernier.github.io/hyperion.rs/hyperion/)
[![GitHub](https://img.shields.io/github/license/vtavernier/hyperion.rs)](LICENSE)

hyperion.rs is a rewrite of
[hyperion.ng](https://github.com/hyperion-project/hyperion.ng) in the Rust
Programming Language. This version features:

* Very low resource requirements (can be run on a single thread, useful for the
  Raspberry Pi Zero for example)
* No memory leaks (most allocations are static anyways)
* Easy to compile *and* cross-compile

***Disclaimer: this is an early work-in-progress***:
* A lot of internals may still change, in case you would like to contribute
  please contact me beforehand.
* Only a small subset of the original project's features are currently
  supported. This means that unless you're using it in the exact same context as
  I am (Android TV Grabber + Raspberry Pi Zero W + WS2815 LED strip), this might
  not be for you.

## Compatibility

Currently implemented features:

* Loading settings from the hyperion.ng database
* Basic WS2812SPI device (no invert, no latch time)
* JSON, Protobuf and Flatbuffers server

## Migrating your settings

Due to the ORM being used in this project, we need the settings database to have
primary keys setup. This doesn't change the schema, but to ensure compatibility,
it is best to use a separate database for hyperion.ng and hyperion.rs.

This process will be more streamlined in the future, but for testing purposes
you can follow these instructions: assuming you have the sqlite3 client
installed, and you are in your home directory, with hyperion.rs being a clone of
this repository:

```bash
# Create config directory for hyperion.rs
$ mkdir -p .config/hyperion.rs/

# (if not done already) Install the ORM client diesel
$ cargo install --force diesel_cli

# Setup the database
$ (cd hyperion.rs && DATABASE_URL=$HOME/.config/hyperion.rs/hyperion.db diesel setup)

# Open sqlite3
$ sqlite3
> ATTACH ".hyperion/db/hyperion.db" AS db1;        -- Attach the original hyperion.ng database
> ATTACH ".config/hyperion.rs/hyperion.db" AS db2; -- Attach the new hyperion.rs database
> -- Copy the tables
> BEGIN TRANSACTION;
> 	INSERT INTO db2.instances SELECT * FROM db1.instances;
> 	INSERT INTO db2.auth SELECT * FROM db1.auth;
> 	INSERT INTO db2.meta SELECT * FROM db1.meta;
> 	INSERT INTO db2.settings SELECT * FROM db1.settings;
> COMMIT;
```

## Running hyperion.rs

Once your settings database has been migrated, you can run hyperion.rs using
`cargo`:

```bash
$ cargo run
```

If running from a release archive, invoke the `hyperiond-rs` binary directly.

## Cross-compiling

Cross-compiling is done using [cross](https://github.com/rustembedded/cross). As
hyperion.rs has native dependencies, we first need to prepare a Docker image
with the required dependencies, and then build using the resulting image.

*Note: these images will be published on dockerhub as the project stabilizes.*

Let's say we are building for the Raspberry Pi Zero, which corresponds to the
Rust target arm-unknown-linux-gnueabihf.

```bash
$ export TARGET=arm-unknown-linux-gnueabihf

# Build the Docker image
$ (cd docker && docker build . -f Dockerfile.$TARGET -t vtavernier/cross-hyperion:$TARGET)

# (if not done already) Install cross
$ cargo install --force cross

# Build the project
$ cross build --release --target $TARGET

# The resulting binaries will be in target/$TARGET/release
```

## License

This work is licensed under the [MIT License](LICENSE).

## Author

Vincent Tavernier <vince.tavernier@gmail.com>. Original project and protocol
source files by [hyperion-project](https://github.com/hyperion-project).
