# hyperionc-udp

`hyperionc-udp` is an implementation of the raw UDP protocol for debugging.

Values received through the bound socket are forwarded to the standard
output in CSV format (tab separated, for gnuplot).

## Usage

   $ cargo run -- --help
    hyperionc-udp 0.1.0

    USAGE:
        hyperionc-udp [OPTIONS]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -b, --bind <bind>                Address to bind to [default: 0.0.0.0:19446]
        -c, --components <components>    Number of components per LED [default: 3]
        -l, --count <led-count>          Number of LEDs [default: 1]


## Authors

* [Vincent Tavernier](https://github.com/vtavernier)

## License

This source code is released under the [MIT-License](https://opensource.org/licenses/MIT)
