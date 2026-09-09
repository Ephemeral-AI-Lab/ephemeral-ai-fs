#!/usr/bin/env bash
set -euo pipefail
here=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
exec python3 "$here/shared/runner.py" --family small_file_delta_smoke "$@"
