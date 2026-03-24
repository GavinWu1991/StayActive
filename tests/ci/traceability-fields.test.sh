#!/usr/bin/env bash
set -euo pipefail

OUT="tmp-release-metadata.json"
bash scripts/ci/generate-release-metadata.sh "$OUT" >/dev/null

rg "\"source_revision\"" "$OUT" >/dev/null
rg "\"pipeline_run_id\"" "$OUT" >/dev/null
rg "\"workflow_run_url\"" "$OUT" >/dev/null
rg "\"artifacts\"" "$OUT" >/dev/null
rg "\"download_uri\"" "$OUT" >/dev/null

rm -f "$OUT"
echo "PASS: traceability fields present"
