# GitHub Actions Pipeline Guide

This document defines the CI/CD workflow behavior for StayActive.

## Workflow names and required check

- PR workflow: `PR Quality Gate` (`.github/workflows/ci-pr.yml`)
- Main workflow: `Main Release Pipeline` (`.github/workflows/release-main.yml`)
- Required branch protection check: `PR Quality Gate / quality-gate`

## PR workflow conventions

- Trigger on `pull_request` events: `opened`, `synchronize`, `reopened` for `main`.
- Run quality gate only (no publish/release steps).
- Use least-privilege permissions.
- Keep workflow fork-safe (no privileged secrets required).

## Main workflow conventions

- Trigger on `push` to `main` and optional `workflow_dispatch`.
- Enforce order: quality-gate -> build installers -> publish artifacts.
- Publish only when prior stages succeed.
- Artifact names include short SHA and run ID to avoid collisions.

## Rerun policy

- `quality-gate` can be rerun alone for transient failures.
- Build and publish reruns should only happen after quality-gate passes.
- Failed publish runs should be rerun from failed jobs to preserve run history.

## Branch protection configuration

In repository branch protection for `main`:

1. Enable "Require status checks to pass before merging".
2. Mark `PR Quality Gate / quality-gate` as required.
3. Optionally require branches to be up to date before merge.

## Troubleshooting and traceability playbook

1. Open workflow run summary and copy:
   - run URL
   - triggering commit SHA
   - release or artifact URLs
2. If quality-gate fails, inspect:
   - frontend build output (`npm run build`)
   - rust test output (`cargo test`)
   - rust lint output (`cargo clippy`)
3. If publish fails:
   - confirm generated metadata file exists
   - verify release permissions are present
   - verify artifact upload step output
4. For each published installer set, verify mapping:
   - `source_revision` -> commit SHA
   - `pipeline_run_id` -> workflow run ID
   - `artifacts[]` -> download URLs

## End-to-end manual verification checklist

- [x] PR workflow trigger contract verified locally via `tests/ci/pr-trigger-events.test.sh`.
- [x] Required-check naming contract verified in workflow/docs and contract tests.
- [ ] Push to `main` remote execution verified on GitHub Actions run.
- [x] Job dependency order verified locally via `tests/ci/release-main-trigger-order.test.sh`.
- [x] Artifact naming strategy with short SHA + run ID implemented in workflow.
- [x] Metadata schema fields verified via `tests/ci/release-metadata-schema.test.sh`.

### Verification outcomes (current)

- Local contract tests: pass (6/6)
- Workflow files generated: `ci-pr.yml`, `release-main.yml`
- Pending remote confirmation: first PR run and first `main` run in GitHub
