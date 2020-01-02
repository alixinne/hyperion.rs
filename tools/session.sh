#!/bin/bash

COMMANDS=commands.txt
DELAY=1.0
while getopts ":d:c:" opt; do
  case "$opt" in
    c)
      COMMANDS="$OPTARG"
      ;;
    d)
      DELAY="$OPTARG"
      ;;
  esac
done

export HYPERION_LOG=trace
export RUST_BACKTRACE=1

cargo run -q -p hyperiond -- -h &>/dev/null
cargo run -q -p hyperionc-udp -- -h &>/dev/null

for T in nearest linear; do
  cargo run -q -p hyperionc-udp -- -l 4 -c 4 > hyperionc-udp/$T.csv &
  CUDP=$!
  cargo run -q -p hyperiond -- -b 0.0.0.0 -c config-$T.yml &
  DAEM=$!
  sleep 1

  cleanup () {
    kill -9 $CUDP $DAEM $NC
  }

  trap cleanup INT

  while IFS= read -r line; do
    echo "$line"
    sleep $DELAY
  done <$COMMANDS | ( nc 127.0.0.1 19444 & )
  NC=$!

  cleanup
done
