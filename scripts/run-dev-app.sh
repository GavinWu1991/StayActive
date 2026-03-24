#!/usr/bin/env bash
# Build StayActive as a debug .app and open it. Use this for dev when you need
# Accessibility (Start Stay Active): macOS only allows adding .app to the list.
#
# First time: run this script once, then add the .app to Accessibility:
#   System Settings → Privacy & Security → Accessibility → + →
#   Go to Folder (Cmd+Shift+G) → paste the path printed below → select StayActive.app
# Then run this script again; Start Stay Active will work.

set -e
cd "$(dirname "$0")/.."

echo "Building debug .app..."
cargo tauri build --debug

APP_PATH="src-tauri/target/debug/bundle/macos/StayActive.app"
if [[ ! -d "$APP_PATH" ]]; then
  echo "Error: $APP_PATH not found"
  exit 1
fi

echo ""
echo "Add this .app to Accessibility (first time only):"
echo "  $(pwd)/$APP_PATH"
echo ""
echo "Debug logs (when running the .app) are written to:"
echo "  $HOME/Library/Logs/StayActive/debug.log"
echo "  View live: tail -f \"$HOME/Library/Logs/StayActive/debug.log\""
echo ""
echo "Opening StayActive.app..."
open "$APP_PATH"
