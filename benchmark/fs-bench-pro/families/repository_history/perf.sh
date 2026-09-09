#!/bin/sh
set -eu
exec python3 "$(dirname "$0")/../../shared/runner.py" --family repository_history "$@"
