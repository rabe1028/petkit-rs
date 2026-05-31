#!/bin/sh
set -eu

MSRV="${MSRV:-1.88.0}"
NIGHTLY="${NIGHTLY:-nightly}"
LOCKFILE_BACKUP_DIR="$(mktemp -d)"
LOCKFILE_BACKUP="${LOCKFILE_BACKUP_DIR}/Cargo.lock"
HAD_LOCKFILE=0

cleanup() {
  rm -f Cargo.lock
  if [ "${HAD_LOCKFILE}" -eq 1 ]; then
    mv "${LOCKFILE_BACKUP}" Cargo.lock
  fi
  rmdir "${LOCKFILE_BACKUP_DIR}"
}
trap cleanup EXIT

if [ -f Cargo.lock ]; then
  HAD_LOCKFILE=1
  mv Cargo.lock "${LOCKFILE_BACKUP}"
fi

cargo "+${NIGHTLY}" generate-lockfile -Z direct-minimal-versions
cargo "+${MSRV}" test --workspace --all-targets --all-features --locked
cargo "+${MSRV}" check -p petkit-types --no-default-features --locked
cargo "+${MSRV}" check -p petkit-protocol --no-default-features --locked
