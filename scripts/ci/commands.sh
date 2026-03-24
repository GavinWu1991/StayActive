#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(CDPATH='' cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

run_frontend_checks() {
  cd "$ROOT_DIR"
  npm ci
  npm run build
}

run_rust_checks() {
  cd "$ROOT_DIR"
  cargo test --manifest-path src-tauri/Cargo.toml
  cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
}

run_quality_gate() {
  run_frontend_checks
  run_rust_checks
}

case "${1:-quality-gate}" in
  frontend)
    run_frontend_checks
    ;;
  rust)
    run_rust_checks
    ;;
  quality-gate)
    run_quality_gate
    ;;
  *)
    echo "Unknown command '$1'. Use one of: frontend | rust | quality-gate" >&2
    exit 1
    ;;
esac
