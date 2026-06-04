#!/bin/sh
TARGET=$1
RELEASE=$2

case "${TARGET}" in
  "freebsd")
    export BUILD_TARGET="x86_64-unknown-freebsd"
    export PKG_CONFIG_PATH="/usr/local/libdata/pkgconfig:/usr/libdata/pkgconfig"
    export PKG_CONFIG_ALLOW_SYSTEM_CFLAGS="1"
    ;;
  "linux")
    export BUILD_TARGET="x86_64-unknown-linux-gnu"
    export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig"
    ;;
  "windows")
    export BUILD_TARGET="x86_64-pc-windows-gnu"
    ;;
  *)
    echo "Valid targets are freebsd, linux or windows"
    return 1
  ;;
esac

# Add -- to the release argument if not present
[ "${RELEASE}" = "release" ] && RELEASE="--release"

if [ "${RELEASE}" = "release" ]; then
  rustup override unset
  cargo build --target ${BUILD_TARGET} ${RELEASE}
else
  rustup override set nightly
  RUSTFLAGS="-Zmacro-backtrace" cargo build --target ${BUILD_TARGET} ${RELEASE}
fi
