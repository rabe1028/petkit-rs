#!/bin/sh
set -eu

unset CDPATH
ROOT=$(cd -- "$(dirname -- "$0")/.." && pwd)

export PETKIT_RS_ROOT="$ROOT"
python3 - <<'PY'
import os
import shlex
import subprocess
import sys
from pathlib import Path


def read_dotenv(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.exists():
        return values
    for raw_line in path.read_text().splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key:
            continue
        if value and value[0] in "'\"" and value[-1:] == value[0]:
            value = shlex.split(value)[0]
        values[key] = value
    return values


root = Path(os.environ["PETKIT_RS_ROOT"])
env = read_dotenv(root / ".env")
env.update(os.environ)

email = env.get("PETKIT_EMAIL", "")
password = env.get("PETKIT_PASSWORD", "")
login_code = env.get("PETKIT_LOGIN_CODE", "")
if not email or email == "user@example.com":
    print("PETKIT_EMAIL is required; put it in .env or export it in your shell.", file=sys.stderr)
    sys.exit(2)
if not login_code and (not password or password == "password"):
    print("PETKIT_PASSWORD or PETKIT_LOGIN_CODE is required; put it in .env or export it in your shell.", file=sys.stderr)
    sys.exit(2)

print("== Cloud BLE relay probe ==")
subprocess.run(
    [
        "cargo",
        "run",
        "-p",
        "petkit-client",
        "--example",
        "reqwest_blocking_cloud_ble_probe",
        "--no-default-features",
        "--features",
        "blocking,reqwest-blocking",
    ],
    cwd=root,
    env=env,
    check=True,
)

if env.get("PETKIT_SMOKE_CAMERA", "0") == "1":
    print("== Camera live-feed probe ==")
    subprocess.run(
        [
            "cargo",
            "run",
            "-p",
            "petkit-client",
            "--example",
            "reqwest_async_camera_live_feed_probe",
            "--no-default-features",
            "--features",
            "async,reqwest-async",
        ],
        cwd=root,
        env=env,
        check=True,
    )
else:
    print("Skipping camera probe; set PETKIT_SMOKE_CAMERA=1 to run it.")
PY
