#!/bin/bash
# Build and run AzurePIM as a proper .app bundle
# This is required for URL scheme (azurepim://) registration

set -e

# Configuration
APP_NAME="AzurePIM"
BUNDLE_ID="de.malvik.azurepim.desktop"
BUILD_TYPE="${1:-debug}"

# Paths
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_BUNDLE="$PROJECT_ROOT/target/$BUILD_TYPE/$APP_NAME.app"
CONTENTS_DIR="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

# Build
echo "Building ($BUILD_TYPE)..."
if [ "$BUILD_TYPE" = "release" ]; then
    cargo build --release --manifest-path "$PROJECT_ROOT/Cargo.toml"
    BINARY="$PROJECT_ROOT/target/release/azurepim"
else
    cargo build --manifest-path "$PROJECT_ROOT/Cargo.toml"
    BINARY="$PROJECT_ROOT/target/debug/azurepim"
fi

# Find Info.plist from build output
PLIST_PATH=$(find "$PROJECT_ROOT/target/$BUILD_TYPE/build" -name "Info.plist" -path "*/azurepim-*/out/*" 2>/dev/null | head -1)

if [ -z "$PLIST_PATH" ]; then
    echo "Error: Info.plist not found in build output"
    exit 1
fi

echo "Creating app bundle at $APP_BUNDLE..."

# Create bundle structure
rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy binary
cp "$BINARY" "$MACOS_DIR/azurepim"

# Copy Info.plist
cp "$PLIST_PATH" "$CONTENTS_DIR/Info.plist"

# Create PkgInfo
echo -n "APPL????" > "$CONTENTS_DIR/PkgInfo"

# Register the URL scheme with Launch Services
echo "Registering URL scheme..."
/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister -f "$APP_BUNDLE"

echo "Launching $APP_NAME..."
echo "(Use Ctrl+C to stop, or quit from the menu bar)"
echo ""

# Run the binary directly (open -W doesn't wait for LSUIElement apps)
exec "$MACOS_DIR/azurepim"
