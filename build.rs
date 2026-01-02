//! Build script that generates Info.plist for the macOS application bundle.
//!
//! This enables:
//! - Custom URL scheme (azurepim://callback) for OAuth
//! - LSUIElement (no dock icon)
//! - Bundle metadata

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Generate Info.plist in the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let plist_path = Path::new(&out_dir).join("Info.plist");

    let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>de.malvik.azurepim.desktop</string>

    <key>CFBundleName</key>
    <string>Azure PIM</string>

    <key>CFBundleDisplayName</key>
    <string>Azure PIM</string>

    <key>CFBundleExecutable</key>
    <string>azurepim</string>

    <key>CFBundleVersion</key>
    <string>0.1.0</string>

    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>

    <key>CFBundlePackageType</key>
    <string>APPL</string>

    <!-- No dock icon - menu bar only -->
    <key>LSUIElement</key>
    <true/>

    <!-- Custom URL scheme for OAuth callback -->
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>CFBundleURLName</key>
            <string>de.malvik.azurepim.oauth</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>azurepim</string>
            </array>
        </dict>
    </array>

    <!-- Minimum macOS version -->
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>

    <!-- High resolution capable -->
    <key>NSHighResolutionCapable</key>
    <true/>

    <!-- Principal class for Cocoa apps -->
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
"#;

    fs::write(&plist_path, plist_content).expect("Failed to write Info.plist");

    // Tell Cargo to rerun if build.rs changes
    println!("cargo:rerun-if-changed=build.rs");

    // Output the path for reference
    println!("cargo:rustc-env=INFO_PLIST_PATH={}", plist_path.display());
}
