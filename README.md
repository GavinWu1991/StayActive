# StayActive

[中文文档 (Chinese README)](./README.zh-CN.md)

StayActive is a macOS menu bar app that keeps your Mac appearing active by simulating lightweight activity and optionally preventing sleep.  
Built with Tauri v2 (`Rust` + `React` + `Vite`).

## Current Scope

- Platform support: **macOS only**
- CI/CD support: **macOS only**
- Delivery stage: **MVP**

## MVP Features

- Menu bar app with tray icon state (`on` / `off`)
- One-click **Stay Active** toggle
- Timer presets: `10m`, `30m`, `1h`, `2h`, `3h`
- Custom timer end-time picker
- Countdown display and cancel flow from tray menu
- Settings window (automation behavior + language)
- Localization: English and Chinese
- Accessibility permission guidance and in-app prompt handling

## CI/CD (GitHub Actions, macOS-only)

- PR workflow: `.github/workflows/ci-pr.yml`
  - Trigger: pull requests to `main`
  - Job: `quality-gate` on `macos-latest`
- Main workflow: `.github/workflows/release-main.yml`
  - Trigger: push to `main` and optional manual dispatch
  - Order: `quality-gate` -> `build-installers-macos` -> metadata/artifacts
  - Does not create public GitHub Releases
- Tag workflow: `.github/workflows/release-tag.yml`
  - Trigger: push of `v*` tags (e.g. `v0.1.0`) or manual dispatch with a tag
  - Builds macOS package and publishes a GitHub Release with downloadable assets
  - Auto-generates release notes from commits since the previous tag

See also:

- `docs/ci/github-actions-pipeline.md`
- `specs/005-github-actions-pipeline/contracts/workflow-triggers.md`

## Public trial download (GitHub Releases)

1. Open [Releases](https://github.com/GavinWu1991/StayActive/releases).
2. Download the latest macOS `.dmg` / app asset.
3. Follow [First Launch](#first-launch-important) below.

### Cut a versioned release

```bash
# 1) Ensure src-tauri/tauri.conf.json version matches (e.g. 0.1.0)
# 2) Merge to main, then:
git tag v0.1.0
git push origin v0.1.0
```

GitHub Actions runs `Tag Release Pipeline` and publishes the release page automatically.

## First Launch (Important)

If the app is downloaded outside the Mac App Store:

1. Do **not** double-click first.
2. Right-click `.app` -> **Open** -> confirm **Open**.
3. Grant Accessibility permission in  
   **System Settings -> Privacy & Security -> Accessibility**.

## Local Development

### Prerequisites

- macOS
- Node.js LTS
- Rust stable toolchain

### Install

```bash
npm install
```

### Run (frontend dev)

```bash
npm run dev
```

### Run app (recommended for Accessibility testing)

```bash
npm run dev:app
```

### Build

```bash
npm run build
cargo tauri build
```

Build output (macOS app bundle):

`src-tauri/target/release/bundle/macos/StayActive.app`

### Optional ad-hoc sign (MVP)

```bash
codesign --force --deep -s - src-tauri/target/release/bundle/macos/StayActive.app
```

## Quality Gate Commands

The workflow quality gate currently uses:

```bash
npm ci
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

Or use the helper script:

```bash
bash scripts/ci/commands.sh quality-gate
```

## Debug Logs

In debug builds, logs are written to:

`~/Library/Logs/StayActive/debug.log`

Live view:

```bash
tail -f ~/Library/Logs/StayActive/debug.log
```
