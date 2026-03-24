# StayActive

StayActive is a macOS menu bar app that keeps your Mac appearing active (no sleep, no idle in collaboration tools like Teams) via minimal simulated mouse movement and optional sleep prevention. Built with Tauri v2 (Rust + React/Vite).

**Menu bar**: The tray icon shows “on” when stay-active is running and “off” when stopped. Use the menu: **Stay Active** (toggle with checkmark), **Timer** (presets 10 min–3 hr or **Custom…** for a time picker), **Settings…**, **Quit**. An active timer shows remaining time in the Timer submenu; click it to cancel with confirmation.

## First launch (important)

If you downloaded the app outside the Mac App Store:

1. **Do not double-click** the app. Gatekeeper may block it.
2. **Right-click** the app (or `.app` bundle) → **Open** → confirm **Open** in the dialog.
3. Grant **Accessibility** permission when prompted: **System Settings** → **Privacy & Security** → **Accessibility** → add StayActive and enable it.

## Development

- **Prerequisites**: Rust (stable), Node.js (LTS), macOS.
- **Run**: `npm install` then `cargo tauri dev` (or `npx tauri dev`).
- **Build**: `cargo tauri build`. Output: `src-tauri/target/release/bundle/macos/StayActive.app`.
- **Ad-hoc sign (MVP)**: `codesign --force --deep -s - src-tauri/target/release/bundle/macos/StayActive.app`
- **Dev with Accessibility (Start Stay Active)**: macOS only allows adding a `.app` to Accessibility. Run `npm run dev:app` to build a debug `.app` and open it; add that `.app` to **Privacy & Security → Accessibility** (use Cmd+Shift+G and open `src-tauri/target/debug/bundle/macos`), then run `npm run dev:app` again to test.
- **Debug logs (debug build only)**: Logs are written to **`~/Library/Logs/StayActive/debug.log`**. View live: `tail -f ~/Library/Logs/StayActive/debug.log`. When you run the app via `open StayActive.app`, stderr is not attached to a terminal, so use this file to collect logs.
