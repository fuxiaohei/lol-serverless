#!/usr/bin/env bash

cargo build -p hello-wasm --target wasm32-wasip1 --release
# cargo build -p wait-until --target wasm32-wasi --release
cargo test -p land-wasm-host -- --nocapture