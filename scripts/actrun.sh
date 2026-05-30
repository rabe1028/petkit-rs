#!/bin/sh
set -eu

ACTRUN_NIX_REF="${ACTRUN_NIX_REF:-github:mizchi/actrun/v0.29.0}"

if command -v nix >/dev/null 2>&1; then
  exec nix run "${ACTRUN_NIX_REF}" -- "$@"
fi

if command -v actrun >/dev/null 2>&1; then
  exec actrun "$@"
fi

echo "actrun is required. Prefer nix: nix run ${ACTRUN_NIX_REF} -- <args>" >&2
exit 127
