# StayActive

[中文文档](./README.zh-CN.md)

**StayActive** is a lightweight macOS menu bar app that keeps your Mac looking active—so collaboration tools (like Microsoft Teams) stay green, and your machine can skip idle sleep when you need it to.

It lives in the menu bar only (no Dock icon), and works by simulating tiny mouse activity and optionally holding a system wake assertion.

> **Platform:** macOS only · **Status:** MVP (v0.1.0)

---

## For users

### Why StayActive?

- Stay “available” in Teams and similar apps during long meetings or focus blocks
- Optionally prevent the Mac from sleeping while Stay Active is on
- One-click toggle from the menu bar; optional auto-stop timer
- English and Chinese UI

### Download

1. Open the latest [GitHub Release](https://github.com/GavinWu1991/StayActive/releases).
2. Download the macOS `.dmg` (or `.app` asset).
3. Follow **First launch** below before using it day to day.

### First launch (important)

Builds are not from the Mac App Store, so macOS Gatekeeper will block a normal double-click.

1. Do **not** double-click the app the first time.
2. In Finder, **right-click** the `.app` → **Open** → confirm **Open**.
3. When prompted, grant **Accessibility** permission:  
   **System Settings → Privacy & Security → Accessibility** → enable **StayActive**.

Without Accessibility, StayActive cannot simulate input and will guide you to turn it on.

### How to use

Click the menu bar icon:

| Menu item | What it does |
|-----------|----------------|
| **Stay Active** | Turn activity simulation on or off |
| **Timer** | Auto-stop after 10m / 30m / 1h / 2h / 3h, or **Custom…** end time |
| Countdown (when a timer is set) | Shows time left; click to cancel the timer (Stay Active keeps running) |
| **Settings...** | Interval, mouse move/click options, prevent sleep, language |
| **Quit** | Exit the app |

The tray icon reflects on/off state.

### Privacy & behavior notes

- Activity is local: small mouse moves/clicks on your Mac. Nothing is sent to a server.
- Simulation pauses briefly if you recently used the mouse/keyboard yourself.
- This is an MVP. Some collaboration apps may still detect idle in edge cases; feel free to open an issue if you hit one.

---

## For contributors

Contributions are welcome—bug reports, docs, and PRs. Please keep changes scoped and aligned with the existing macOS MVP.

### Stack

| Layer | Tech |
|-------|------|
| Shell | [Tauri v2](https://v2.tauri.app/) |
| Backend | Rust (`src-tauri/`) |
| Frontend | React 18 + Vite + TypeScript (`src/`) |
| Input simulation | `enigo` |
| Sleep prevention | `keepawake` |

### Prerequisites

- macOS
- [Node.js](https://nodejs.org/) 20+ (LTS; CI uses Node 20)
- [Rust](https://rustup.rs/) stable (edition 2021, `rust-version` ≥ 1.70)
- Tauri CLI: `cargo install tauri-cli` (or use `cargo tauri` via the project tooling)

### Quick start

```bash
npm install
```

Frontend only (browser):

```bash
npm run dev
```

**Desktop app (recommended)** — needed to test Accessibility (only a `.app` can be added in System Settings):

```bash
npm run dev:app
```

This builds a debug `.app` and opens it. Add that `.app` under **Accessibility**, then run `dev:app` again if needed.

### Project layout

```
src/                 React UI (settings, timer picker, locales)
src-tauri/           Rust backend (tray, automation, permissions, timer)
scripts/             Dev helpers and CI command wrappers
docs/                Design and CI documentation
specs/               Feature specs / contracts
.github/workflows/   PR quality gate, main builds, tag releases
```

### Build

```bash
npm run build
cargo tauri build
```

App bundle:

`src-tauri/target/release/bundle/macos/StayActive.app`

Optional ad-hoc sign (local MVP):

```bash
codesign --force --deep -s - src-tauri/target/release/bundle/macos/StayActive.app
```

### Quality gate

Same checks as CI:

```bash
bash scripts/ci/commands.sh quality-gate
```

Or manually:

```bash
npm ci
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

### Debug logs

Debug builds write to:

`~/Library/Logs/StayActive/debug.log`

```bash
tail -f ~/Library/Logs/StayActive/debug.log
```

### CI / CD (macOS only)

| Workflow | When | What |
|----------|------|------|
| [ci-pr.yml](.github/workflows/ci-pr.yml) | PR → `main` | Quality gate on `macos-latest` |
| [release-main.yml](.github/workflows/release-main.yml) | Push to `main` | Quality gate → build installers → artifacts (**no** public Release) |
| [release-tag.yml](.github/workflows/release-tag.yml) | `v*` tag | Build + publish [GitHub Release](https://github.com/GavinWu1991/StayActive/releases) |

Details: [docs/ci/github-actions-pipeline.md](./docs/ci/github-actions-pipeline.md)

### Cut a release

1. Align `version` in `src-tauri/tauri.conf.json` (e.g. `0.1.0`).
2. Merge to `main`, then:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The tag pipeline publishes the Release and assets automatically.

### Further reading

- Design notes: [docs/start.md](./docs/start.md)
- Specs under `specs/` for feature contracts

---

## License

No license file is published yet. All rights reserved until a license is added—ask before redistributing.