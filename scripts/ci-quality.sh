#!/bin/sh
set -eu

make quality
make deny
make machete
make typos

