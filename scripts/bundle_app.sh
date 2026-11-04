#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/release/cs2-settings"
# Prefer explicit Intel target path when cross/CI builds for x86_64-apple-darwin
if [[ ! -x "$BIN" && -x "$ROOT/target/x86_64-apple-darwin/release/cs2-settings" ]]; then
  BIN="$ROOT/target/x86_64-apple-darwin/release/cs2-settings"
fi
APP_NAME="CS2 Settings"
DEST="${1:-$ROOT/dist/$APP_NAME.app}"
VERSION="${APP_VERSION:-1.0.0}"

if [[ ! -x "$BIN" ]]; then
  echo "Missing release binary. Run: cargo build --release" >&2
  exit 1
fi

rm -rf "$DEST"
mkdir -p "$DEST/Contents/MacOS" "$DEST/Contents/Resources"
cp "$BIN" "$DEST/Contents/MacOS/cs2-settings"
chmod +x "$DEST/Contents/MacOS/cs2-settings"

# Optional .icns from PNG (macOS only)
ICON_PNG="$ROOT/assets/icons/icon.png"
ICNS="$DEST/Contents/Resources/AppIcon.icns"
ICON_KEY=""
if [[ -f "$ICON_PNG" ]] && command -v sips >/dev/null 2>&1 && command -v iconutil >/dev/null 2>&1; then
  ICONSET="$(mktemp -d)/AppIcon.iconset"
  mkdir -p "$ICONSET"
  for size in 16 32 128 256 512; do
    sips -z "$size" "$size" "$ICON_PNG" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    sips -z $((size*2)) $((size*2)) "$ICON_PNG" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
  done
  if iconutil -c icns "$ICONSET" -o "$ICNS" 2>/dev/null; then
    ICON_KEY=$'\n  <key>CFBundleIconFile</key><string>AppIcon</string>'
  fi
  rm -rf "$(dirname "$ICONSET")"
elif [[ -f "$ICON_PNG" ]]; then
  cp "$ICON_PNG" "$DEST/Contents/Resources/icon.png"
fi

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
  <key>CFBundleShortVersionString</key><string>${VERSION}</string>
  <key>CFBundleVersion</key><string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>NSHighResolutionCapable</key><true/>${ICON_KEY}
</dict>
</plist>
PLIST

echo "Bundled → $DEST"
