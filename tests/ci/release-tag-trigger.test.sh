#!/usr/bin/env bash
set -euo pipefail

FILE=".github/workflows/release-tag.yml"

rg "^name: Tag Release Pipeline" "$FILE" >/dev/null
rg "tags:" "$FILE" >/dev/null
rg '"v\*"' "$FILE" >/dev/null
rg "needs: \[quality-gate\]" "$FILE" >/dev/null
rg "generateReleaseNotes: true" "$FILE" >/dev/null
rg "tauri-apps/tauri-action@v0" "$FILE" >/dev/null
rg "validate-release-tag.sh" "$FILE" >/dev/null

echo "PASS: release-tag trigger, order, and notes checks"
