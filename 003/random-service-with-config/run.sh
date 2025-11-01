#!/bin/sh

export RUST_LOG=trace
cargo run -- --config microservice.toml

# cargo run -- --help
# cargo run -- --config microservice.toml --address 127.0.0.1:9876
# cargo run -- --address 127.0.0.1:9876