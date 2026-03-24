#!/usr/bin/env bash
set -euo pipefail

FILE=".github/workflows/ci-pr.yml"

rg "^permissions:" "$FILE" >/dev/null
rg "contents: read" "$FILE" >/dev/null

if rg "tauri-apps/tauri-action|upload-artifact|release" "$FILE" >/dev/null; then
  echo "FAIL: PR workflow contains publish-oriented steps" >&2
  exit 1
fi

echo "PASS: PR fork-safety policy checks"
