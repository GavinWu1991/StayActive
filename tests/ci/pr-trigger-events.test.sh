#!/usr/bin/env bash
set -euo pipefail

FILE=".github/workflows/ci-pr.yml"

rg "^name: PR Quality Gate" "$FILE" >/dev/null
rg "pull_request:" "$FILE" >/dev/null
rg "branches: \[main\]" "$FILE" >/dev/null
rg "types: \[opened, synchronize, reopened\]" "$FILE" >/dev/null
rg "name: quality-gate" "$FILE" >/dev/null

echo "PASS: PR trigger event checks"
