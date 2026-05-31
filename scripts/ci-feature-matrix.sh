#!/bin/sh
set -eu

cargo check -p petkit-client --lib --no-default-features
cargo check -p petkit-client --lib --no-default-features --features async
cargo check -p petkit-client --lib --no-default-features --features blocking
cargo check -p petkit-client --lib --no-default-features --features async,blocking
cargo check -p petkit-client --examples --no-default-features --features async,reqwest-async
cargo check -p petkit-client --examples --no-default-features --features blocking,reqwest-blocking
cargo check -p petkit-client --examples --no-default-features --features blocking,ureq-blocking
cargo check -p petkit-client --examples --no-default-features --features async,blocking,reqwest-native
