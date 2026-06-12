#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${PROJECT_DIR}/target/mobile/android"

# Check for cargo-ndk
if ! command -v cargo-ndk &>/dev/null; then
    echo "Error: cargo-ndk not found. Install with: cargo install cargo-ndk"
    exit 1
fi

# Check for ANDROID_NDK_HOME
if [ -z "${ANDROID_NDK_HOME:-}" ]; then
    echo "Warning: ANDROID_NDK_HOME not set. cargo-ndk may fail to find the NDK."
    echo "Set it to your Android NDK path, e.g.:"
    echo "  export ANDROID_NDK_HOME=\$HOME/Library/Android/sdk/ndk/<version>"
fi

# Install targets if not present
for target in aarch64-linux-android armv7-linux-androideabi; do
    rustup target add "$target" 2>/dev/null || true
done

echo "Building for Android (arm64-v8a, armeabi-v7a)..."
cargo ndk -t arm64-v8a -t armeabi-v7a build --release

# Copy shared libraries to jniLibs structure
mkdir -p "${OUTPUT_DIR}/jniLibs/arm64-v8a"
mkdir -p "${OUTPUT_DIR}/jniLibs/armeabi-v7a"

cp "${PROJECT_DIR}/target/aarch64-linux-android/release/libexif_rm.so" \
    "${OUTPUT_DIR}/jniLibs/arm64-v8a/"
cp "${PROJECT_DIR}/target/armv7-linux-androideabi/release/libexif_rm.so" \
    "${OUTPUT_DIR}/jniLibs/armeabi-v7a/"

# Generate Kotlin bindings (use one of the built .so files)
echo "Generating Kotlin bindings..."
BINDINGS_DIR="${OUTPUT_DIR}/kotlin"
mkdir -p "${BINDINGS_DIR}"

# For bindgen we need a library that can be loaded on the host platform.
# On macOS, use the .dylib; on Linux, use the native .so.
if [ -f "${PROJECT_DIR}/target/release/libexif_rm.dylib" ]; then
    cargo run --bin uniffi-bindgen generate \
        --library "${PROJECT_DIR}/target/release/libexif_rm.dylib" \
        --language kotlin \
        --out-dir "${BINDINGS_DIR}"
elif [ -f "${PROJECT_DIR}/target/release/libexif_rm.so" ]; then
    cargo run --bin uniffi-bindgen generate \
        --library "${PROJECT_DIR}/target/release/libexif_rm.so" \
        --language kotlin \
        --out-dir "${BINDINGS_DIR}"
else
    echo "Error: No host-native library found. Run 'cargo build --release' first." >&2
    exit 1
fi

echo "Done! Android build output:"
echo "  JNI libs:    ${OUTPUT_DIR}/jniLibs/"
echo "  Kotlin bindings: ${BINDINGS_DIR}/"