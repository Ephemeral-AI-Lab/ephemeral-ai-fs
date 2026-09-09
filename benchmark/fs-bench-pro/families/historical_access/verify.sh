#!/bin/sh
set -eu
exec python3 "$(dirname "$0")/../../shared/runner.py" --family historical_access --mode verification "$@"
