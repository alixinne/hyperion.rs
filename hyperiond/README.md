# hyperiond

`hyperiond` is the Rust implementation of the
[Hyperion](https://github.com/hyperion-project/hyperion) ambient lighting software. It is
written from scratch both as an experiment and as a way to add more features.

## Usage

For now, the CLI is only able to start the hyperion server implementation:

    $ cargo run -- server --help
    hyperiond-server 0.1.1
    Starts the server daemon

    USAGE:
        hyperiond --config <FILE> server --bind <ADDRESS> --json-port <PORT> --proto-port <PORT> --web-port <PORT>

    FLAGS:
        -h, --help    Prints help information

    OPTIONS:
            --bind <ADDRESS>       IP address to bind the servers to [default: 127.0.0.1]
            --json-port <PORT>     TCP port for the JSON server [default: 19444]
            --proto-port <PORT>    TCP port for the Protobuf server [default: 19445]
            --web-port <PORT>      TCP port for the Web interface [default: 19080]

Logging is set using the HYPERION_LOG environment variable, which can be set to the desired
logging level (trace, debug, info, warn, error). Note that this will affect logging of all
crates, and if only hyperion logging is required, it should be filtered as such:
`HYPERION_LOG=hyperion=level`. See the [env_logger crate docs](https://docs.rs/env_logger/0.6.1/env_logger/)
for more details.

## Development

The source code in this folder is only responsible for the command-line interface and starting
the server code, which is implemented in the [core crate](../hyperion)

## Authors

* [Vincent Tavernier](https://github.com/vtavernier)

## License

This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)
