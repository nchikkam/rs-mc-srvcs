#!/bin/sh

export RUST_LOG=trace
cargo run -- --address 0.0.0.0:2345

# cargo run -- --help
