# GitHub Actions Pipeline Guide

This document defines the CI/CD workflow behavior for StayActive.

## Workflow names and required check

- PR workflow: `PR Quality Gate` (`.github/workflows/ci-pr.yml`)
- Main workflow: `Main Release Pipeline` (`.github/workflows/release-main.yml`)
- Tag workflow: `Tag Release Pipeline` (`.github/workflows/release-tag.yml`)
- Required branch protection check: `PR Quality Gate / quality-gate`

## PR workflow conventions

- Trigger on `pull_request` events: `opened`, `synchronize`, `reopened` for `main`.
- Run on `macos-latest` (current platform support is macOS-only).
- Run quality gate only (no publish/release steps).
- Use least-privilege permissions.
- Keep workflow fork-safe (no privileged secrets required).

## Main workflow conventions

- Trigger on `push` to `main` and optional `workflow_dispatch`.
- Enforce order: quality-gate -> build installers -> publish metadata/artifacts.
- Build/publish scope is currently macOS only.
- Publish only when prior stages succeed.
- Artifact names include short SHA and run ID to avoid collisions.
- Does **not** create GitHub Releases (those are produced by the tag workflow).

## Tag release workflow conventions

- Trigger on `push` of tags matching `v*` (for example `v0.1.0`), plus optional `workflow_dispatch` with a tag input.
- Enforce order: quality-gate (including tag/version validation) -> build macOS package -> GitHub Release.
- Upload macOS installer/app bundles as release assets for public download.
- Auto-generate release notes via GitHub Release Notes API (`generateReleaseNotes: true`).
- Tags containing a hyphen (for example `v0.1.0-rc.1`) are marked as prerelease.
- Tag version without the leading `v` MUST match `src-tauri/tauri.conf.json` `version`.

## How to cut a public release

1. Ensure `main` is green and ready.
2. Set the version in `src-tauri/tauri.conf.json` (and `package.json` if you keep them in sync).
3. Commit, merge to `main`, then create and push a tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

4. Watch `Tag Release Pipeline` on GitHub Actions.
5. Share the release page: `https://github.com/<owner>/<repo>/releases/tag/v0.1.0`

## Rerun policy

- `quality-gate` can be rerun alone for transient failures.
- Build and publish reruns should only happen after quality-gate passes.
- Failed tag releases can be republished with Actions → `Tag Release Pipeline` → Run workflow (provide the existing tag).
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
3. If tag validation fails:
   - confirm tag is `vX.Y.Z` and matches `src-tauri/tauri.conf.json`
4. If publish fails:
   - confirm generated metadata file exists
   - verify release permissions are present
   - verify artifact upload step output
5. For each published installer set, verify mapping:
   - `source_revision` -> commit SHA
   - `pipeline_run_id` -> workflow run ID
   - `artifacts[]` -> download URLs

## End-to-end manual verification checklist

- [x] PR workflow trigger contract verified locally via `tests/ci/pr-trigger-events.test.sh`.
- [x] Required-check naming contract verified in workflow/docs and contract tests.
- [ ] Push to `main` remote execution verified on GitHub Actions run.
- [x] Job dependency order verified locally via `tests/ci/release-main-trigger-order.test.sh`.
- [x] Tag release contract verified locally via `tests/ci/release-tag-trigger.test.sh`.
- [x] Artifact naming strategy with short SHA + run ID implemented in workflow.
- [x] Metadata schema fields verified via `tests/ci/release-metadata-schema.test.sh`.

### Verification outcomes (current)

- Local contract tests: pass
- Workflow files: `ci-pr.yml`, `release-main.yml`, `release-tag.yml`
- Pending remote confirmation: first PR run, first `main` run, and first `v*` tag release on GitHub
