#!/bin/sh
set -eu

cargo check -p petkit-client --lib --no-default-features
cargo check -p petkit-client --lib --no-default-features --features async
if rustup target list --installed | grep -qx wasm32-wasip2; then
  cargo check -p petkit-client --lib --target wasm32-wasip2 --no-default-features --features async
else
  echo "skipping wasm32-wasip2 petkit-client async check; target is not installed" >&2
fi
cargo check -p petkit-client --lib --no-default-features --features blocking
cargo check -p petkit-client --lib --no-default-features --features async,blocking
cargo check -p petkit-client --lib --no-default-features --features action-adapter
cargo check -p petkit-client --example host_callback_async --no-default-features --features async
cargo check -p petkit-client --examples --no-default-features --features async,reqwest-async
cargo check -p petkit-client --examples --no-default-features --features blocking,reqwest-blocking
cargo check -p petkit-client --examples --no-default-features --features blocking,ureq-blocking
cargo check -p petkit-client --examples --no-default-features --features async,blocking,reqwest-native
