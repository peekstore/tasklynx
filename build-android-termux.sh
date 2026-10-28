#!/bin/sh
# build-android-termux.sh - Build tasklynx for Android / Termux
#
# Two modes:
#   1. Inside Termux: builds natively with Termux's Rust toolchain (bionic).
#   2. On a desktop with the Android NDK: cross-compiles aarch64-linux-android.
#
# Usage:
#   ./build-android-termux.sh                 # auto-detect
#   ABI=arm64-v8a ./build-android-termux.sh   # explicit ABI
#   ANDROID_NDK_HOME=/path/to/ndk ./build-android-termux.sh
set -e

cd "$(dirname "$0")"

IN_TERMUX=false
if [ -n "${TERMUX_VERSION:-}" ] || [ -d "/data/data/com.termux" ]; then
    IN_TERMUX=true
fi

if [ "$IN_TERMUX" = true ]; then
    echo ">>> Building tasklynx natively inside Termux..."
    command -v cargo >/dev/null 2>&1 || {
        echo "cargo not found. Install it first:"
        echo "  pkg install rust clang"
        exit 1
    }
    cargo build --release
    OUT="tasklynx-android-$(getprop ro.product.cpu.abi 2>/dev/null || uname -m)"
    cp target/release/tasklynx "$OUT"
    echo "OK   Built $OUT"
    exit 0
fi

echo ">>> Cross-compiling tasklynx for Android..."

NDK="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-}}"
if [ -z "$NDK" ]; then
    echo "Error: set ANDROID_NDK_HOME to your Android NDK path."
    exit 1
fi

HOST_TAG="linux-x86_64"
case "$(uname -s)" in
    Darwin*) HOST_TAG="darwin-x86_64" ;;
esac
TOOLCHAIN="$NDK/toolchains/llvm/prebuilt/$HOST_TAG/bin"
export PATH="$TOOLCHAIN:$PATH"

ABI="${ABI:-arm64-v8a}"
API="${API:-21}"

case "$ABI" in
    arm64-v8a)
        TARGET="aarch64-linux-android"
        CLANG="$TOOLCHAIN/aarch64-linux-android${API}-clang"
        ENV="AARCH64_LINUX_ANDROID"
        CCVAR="aarch64_linux_android"
        ;;
    armeabi-v7a)
        TARGET="armv7-linux-androideabi"
        CLANG="$TOOLCHAIN/armv7a-linux-androideabi${API}-clang"
        ENV="ARMV7_LINUX_ANDROIDEABI"
        CCVAR="armv7_linux_androideabi"
        ;;
    x86_64)
        TARGET="x86_64-linux-android"
        CLANG="$TOOLCHAIN/x86_64-linux-android${API}-clang"
        ENV="X86_64_LINUX_ANDROID"
        CCVAR="x86_64_linux_android"
        ;;
    *)
        echo "Unsupported ABI: $ABI (use arm64-v8a, armeabi-v7a, or x86_64)"
        exit 1
        ;;
esac

rustup target add "$TARGET"

LINKER_VAR="CARGO_TARGET_${ENV}_LINKER"
AR_VAR="CARGO_TARGET_${ENV}_AR"
export "$LINKER_VAR=$CLANG"
export "$AR_VAR=$TOOLCHAIN/llvm-ar"
export "CC_${CCVAR}=$CLANG"
export "AR_${CCVAR}=$TOOLCHAIN/llvm-ar"

echo "    ABI:    $ABI"
echo "    Target: $TARGET"
echo "    API:    $API"

cargo build --release --target "$TARGET"

OUT="tasklynx-android-${ABI}"
cp "target/${TARGET}/release/tasklynx" "$OUT"
file "$OUT" 2>/dev/null || true
echo "OK   Built $OUT"
