#!/usr/bin/env bash
set -euo pipefail

FILE=".github/workflows/release-main.yml"

rg "actions/upload-artifact@v4" "$FILE" >/dev/null
rg "GITHUB_STEP_SUMMARY" "$FILE" >/dev/null
rg "workflow_run_url" "$FILE" >/dev/null

echo "PASS: pipeline visibility checks"
