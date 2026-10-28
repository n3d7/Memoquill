#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
# mutator's generic overlay omits dist and symlinks Rust crate entry files.
# Materialize only those links in its private worker so rustc resolves modules
# through the mutated source, and supply the already-built passive UI assets.
python3 - "$repo_root" <<'PY'
import os
import sys
from pathlib import Path
root = Path(sys.argv[1])
worker = Path.cwd()
if worker != root:
    worker.relative_to(root / 'target' / 'mutation-workers')
    for relative in ['src-tauri/Cargo.toml', 'src-tauri/src/lib.rs']:
        path = worker / relative
        if path.is_symlink():
            content = path.read_bytes()
            path.unlink()
            path.write_bytes(content)
    if not (worker / 'dist').exists():
        (worker / 'dist').symlink_to(root / 'dist', target_is_directory=True)
os.environ['CARGO_TARGET_DIR'] = str(root / 'src-tauri/target')
os.execvpe('bash', ['bash', str(root / 'tests/qualification/with-test-env.sh'), 'cargo', 'test', '--locked', '--offline', '--manifest-path', 'src-tauri/Cargo.toml', '--lib', '-j', '2'], os.environ)
PY
