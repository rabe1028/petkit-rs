#!/bin/sh
set -eu

MSRV="${MSRV:-1.88.0}"

cargo "+${MSRV}" test --workspace --all-targets --all-features
cargo "+${MSRV}" check -p petkit-types --no-default-features
cargo "+${MSRV}" check -p petkit-protocol --no-default-features
RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links" \
  cargo "+${MSRV}" doc --workspace --all-features --no-deps
