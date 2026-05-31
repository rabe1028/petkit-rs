#!/bin/sh
set -eu

MSRV="${MSRV:-1.88.0}"
NIGHTLY="${NIGHTLY:-nightly}"

cleanup() {
  rm -f Cargo.lock
}
trap cleanup EXIT

cargo "+${NIGHTLY}" generate-lockfile -Z direct-minimal-versions
cargo "+${MSRV}" test --workspace --all-targets --all-features --locked
cargo "+${MSRV}" check -p petkit-types --no-default-features --locked
cargo "+${MSRV}" check -p petkit-protocol --no-default-features --locked
