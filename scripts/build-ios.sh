#!/bin/bash
set -euo pipefail

# Ensure cargo/rustup are on PATH (Xcode doesn't inherit shell profile)
export PATH="$HOME/.cargo/bin:$PATH"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${PROJECT_DIR}/target/mobile/ios"

# Minimum iOS deployment target (required for ___chkstk_darwin symbol)
export IPHONEOS_DEPLOYMENT_TARGET=15.0

# Install targets if not present
for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    rustup target add "$target" 2>/dev/null || true
done

echo "Building for iOS device (arm64)..."
cargo build --release --target aarch64-apple-ios

echo "Building for iOS simulator (arm64 - Apple Silicon)..."
cargo build --release --target aarch64-apple-ios-sim

# Create output directories
mkdir -p "${OUTPUT_DIR}/device" "${OUTPUT_DIR}/simulator"

# Copy static libraries with renamed output (exif_rmFFI) for the XCFramework
# The Rust crate is still named exif_rm, but we rename for the framework identity
cp "${PROJECT_DIR}/target/aarch64-apple-ios/release/libexif_rm.a" \
   "${OUTPUT_DIR}/device/libexif_rmFFI.a"
cp "${PROJECT_DIR}/target/aarch64-apple-ios-sim/release/libexif_rm.a" \
   "${OUTPUT_DIR}/simulator/libexif_rmFFI.a"

# Generate Swift bindings first (needed for headers)
echo "Generating Swift bindings..."
BINDINGS_DIR="${OUTPUT_DIR}/swift"
mkdir -p "${BINDINGS_DIR}"

cargo run --bin uniffi-bindgen generate \
    --library "${PROJECT_DIR}/target/aarch64-apple-ios/release/libexif_rm.a" \
    --language swift \
    --out-dir "${BINDINGS_DIR}"

# Create headers directory for the XCFramework
HEADERS_DIR="${OUTPUT_DIR}/headers"
mkdir -p "${HEADERS_DIR}"
cp "${BINDINGS_DIR}/exif_rmFFI.h" "${HEADERS_DIR}/"
cp "${BINDINGS_DIR}/exif_rmFFI.modulemap" "${HEADERS_DIR}/module.modulemap"

# Create xcframework with headers (remove existing first to allow rebuilds)
# Named exif_rmFFI.xcframework to match the library identity
echo "Creating xcframework with headers..."
rm -rf "${OUTPUT_DIR}/exif_rmFFI.xcframework"
xcodebuild -create-xcframework \
    -library "${OUTPUT_DIR}/device/libexif_rmFFI.a" \
    -headers "${HEADERS_DIR}" \
    -library "${OUTPUT_DIR}/simulator/libexif_rmFFI.a" \
    -headers "${HEADERS_DIR}" \
    -output "${OUTPUT_DIR}/exif_rmFFI.xcframework"

echo "Done! iOS build output:"
echo "  XCFramework:      ${OUTPUT_DIR}/exif_rmFFI.xcframework"
echo "  Swift bindings:   ${BINDINGS_DIR}/"

# Copy Swift bindings for SPM target
BINDINGS_DEST="${PROJECT_DIR}/ios-rust/Sources/ExifRmRust/"
mkdir -p "${BINDINGS_DEST}"
cp "${BINDINGS_DIR}/exif_rm.swift" "${BINDINGS_DEST}"
echo "  Bindings copied:  ${BINDINGS_DEST}exif_rm.swift"

# Copy XCFramework for SPM binary target
XCFRAMEWORK_DEST="${PROJECT_DIR}/ios-rust/Sources/exif_rmFFI/"
rm -rf "${XCFRAMEWORK_DEST}exif_rmFFI.xcframework"
mkdir -p "${XCFRAMEWORK_DEST}"
cp -r "${OUTPUT_DIR}/exif_rmFFI.xcframework" "${XCFRAMEWORK_DEST}"
echo "  XCFramework copied:  ${XCFRAMEWORK_DEST}exif_rmFFI.xcframework"
