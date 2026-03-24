#!/usr/bin/env bash
set -euo pipefail

SCHEMA="specs/005-github-actions-pipeline/contracts/pipeline-metadata.schema.json"
SCRIPT="scripts/ci/generate-release-metadata.sh"
OUT="tmp-release-metadata.json"

bash "$SCRIPT" "$OUT" >/dev/null

rg "\"schema_version\"" "$OUT" >/dev/null
rg "\"source_revision\"" "$OUT" >/dev/null
rg "\"pipeline_run_id\"" "$OUT" >/dev/null
rg "\"workflow_name\"" "$OUT" >/dev/null
rg "\"test_outcome_aggregate\"" "$OUT" >/dev/null
rg "\"artifacts\"" "$OUT" >/dev/null
rg "\"title\": \"PipelineReleaseMetadata\"" "$SCHEMA" >/dev/null

rm -f "$OUT"
echo "PASS: release metadata fields present"
