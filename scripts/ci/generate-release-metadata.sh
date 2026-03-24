#!/usr/bin/env bash
set -euo pipefail

OUTPUT_PATH="${1:-release-metadata.json}"
SHA="${GITHUB_SHA:-$(git rev-parse HEAD)}"
RUN_ID="${GITHUB_RUN_ID:-local-run}"
WORKFLOW_NAME="${GITHUB_WORKFLOW:-Main Release Pipeline}"
RUN_URL="${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:-local/local}/actions/runs/${RUN_ID}"
OUTCOME="${TEST_OUTCOME_AGGREGATE:-success}"

ARTIFACT_NAME_MACOS="${ARTIFACT_NAME_MACOS:-stayactive-macos-${SHA:0:7}-${RUN_ID}}"
ARTIFACT_URI_MACOS="${ARTIFACT_URI_MACOS:-https://example.invalid/artifacts/${ARTIFACT_NAME_MACOS}}"

cat > "$OUTPUT_PATH" <<EOF
{
  "schema_version": "1.0.0",
  "source_revision": "$SHA",
  "pipeline_run_id": "$RUN_ID",
  "workflow_name": "$WORKFLOW_NAME",
  "workflow_run_url": "$RUN_URL",
  "test_outcome_aggregate": "$OUTCOME",
  "artifacts": [
    {
      "platform": "macos",
      "artifact_name": "$ARTIFACT_NAME_MACOS",
      "download_uri": "$ARTIFACT_URI_MACOS"
    }
  ]
}
EOF

echo "Wrote metadata to $OUTPUT_PATH"
