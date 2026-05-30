#!/bin/sh
set -eu

MUTATION_FILTER="${PETKIT_RS_MUTATION_FILTER:-request_login_code|family_list|iot_device_info_v2}"
MUTATION_OUTPUT="${PETKIT_RS_MUTANTS_OUTPUT:-mutants.out}"
MUTATION_PROFILE="${PETKIT_RS_MUTANTS_PROFILE:-mutants}"

cargo mutants \
  --workspace \
  --all-features \
  --in-place \
  --baseline skip \
  --profile "${MUTATION_PROFILE}" \
  --cap-lints true \
  --output "${MUTATION_OUTPUT}" \
  --timeout 120 \
  -F "${MUTATION_FILTER}"
