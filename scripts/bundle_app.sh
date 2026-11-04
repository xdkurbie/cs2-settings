#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/cs2-settings"
APP_NAME="CS2 Settings"
DEST="${1:-$HOME/Applications/$APP_NAME.app}"

if [[ ! -x "$BIN" ]]; then
  echo "Missing release binary. Run: cargo build --release" >&2
  exit 1
fi

rm -rf "$DEST"
mkdir -p "$DEST/Contents/MacOS" "$DEST/Contents/Resources"
cp "$BIN" "$DEST/Contents/MacOS/cs2-settings"
chmod +x "$DEST/Contents/MacOS/cs2-settings"

cat > "$DEST/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key><string>cs2-settings</string>
  <key>CFBundleIdentifier</key><string>com.xdkurbie.cs2settings</string>
  <key>CFBundleName</key><string>CS2 Settings</string>
  <key>CFBundleDisplayName</key><string>CS2 Settings</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>1.0.0</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

echo "Bundled → $DEST"
