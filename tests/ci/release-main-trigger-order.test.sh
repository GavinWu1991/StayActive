#!/usr/bin/env bash
set -euo pipefail

FILE=".github/workflows/release-main.yml"

rg "^name: Main Release Pipeline" "$FILE" >/dev/null
rg "push:" "$FILE" >/dev/null
rg "branches: \[main\]" "$FILE" >/dev/null
rg "needs: \[quality-gate\]" "$FILE" >/dev/null
rg "needs: \[quality-gate, build-installers\]" "$FILE" >/dev/null

echo "PASS: release-main trigger and order checks"
