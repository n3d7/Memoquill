#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
test_state="$repo_root/.qualification/test-state"
mkdir -p "$test_state/data" "$test_state/config" "$test_state/cache" "$test_state/runtime"
chmod 700 "$test_state/runtime"
export XDG_DATA_HOME="$test_state/data"
export XDG_CONFIG_HOME="$test_state/config"
export XDG_CACHE_HOME="$test_state/cache"
export XDG_RUNTIME_DIR="$test_state/runtime"
exec "$@"
