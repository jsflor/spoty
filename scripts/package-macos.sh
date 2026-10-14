#!/usr/bin/env bash
set -euo pipefail

echo "==========================================="
echo " Packaging Spoty for macOS (ARM64 & x86_64)"
echo "==========================================="

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="${ROOT_DIR}/target/dist"
APP_DIR="${DIST_DIR}/Spoty.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "1. Building release binary..."
cargo build --release -p spoty-ui

echo "2. Creating macOS App Bundle directory structure..."
rm -rf "${APP_DIR}"
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

echo "3. Copying binary..."
cp "${ROOT_DIR}/target/release/spoty" "${MACOS_DIR}/spoty"
chmod +x "${MACOS_DIR}/spoty"

echo "4. Generating Info.plist..."
cat << 'EOF' > "${CONTENTS_DIR}/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>spoty</string>
    <key>CFBundleIdentifier</key>
    <string>com.spoty.desktop</string>
    <key>CFBundleName</key>
    <string>Spoty</string>
    <key>CFBundleDisplayName</key>
    <string>Spoty</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.music</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSRequiresAquaSystemAppearance</key>
    <false/>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
</dict>
</plist>
EOF

echo "5. Generating PkgInfo..."
printf "APPL????" > "${CONTENTS_DIR}/PkgInfo"

echo "==========================================="
echo " Successfully created: ${APP_DIR}"
echo " To install on your Mac, run:"
echo "   cp -r '${APP_DIR}' /Applications/"
echo "==========================================="
