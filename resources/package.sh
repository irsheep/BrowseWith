#!/usr/bin/env sh

VERSION=$1
TARGET=$2
DLLS_ROOT=${DLLS_ROOT:-/c/msys64/mingw64}
PACKAGES_ROOT=${PACKAGES_ROOT:-/tmp/browsewith}

APP_NAME=browsewith-${TARGET}-${VERSION}

if [ $# -lt 2 ]; then
  echo "Invalid arguments"
  echo "usage: ./package.sh <version> freebsd|linux|windows [<packages_root>]"
  echo "ex: ./package.sh 1.0.1 linux"
  exit 1
fi

case "${TARGET}" in
  "freebsd")
    BUILD_TARGET=x86_64-unknown-freebsd
    EXE_NAME=browsewith
    ;;
  "linux")
    BUILD_TARGET=x86_64-unknown-linux-gnu
    EXE_NAME=browsewith
    ;;
  "windows")
    BUILD_TARGET=x86_64-pc-windows-gnu
    EXE_NAME=browsewith.exe
    ;;
  *)
    echo "Valid targets are freebsd, linux or windows"
    return 1
  ;;
esac

# Abort if we can't find browsewith executable
[ ! -f target/${BUILD_TARGET}/release/${EXE_NAME} ] && echo "Application not found" && exit 1

# Create directories
[ ! -d ${PACKAGES_ROOT}/${VERSION}/${APP_NAME} ] && mkdir -p ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}
[ ! -d ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/bin ] && mkdir ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/bin
[ ! -d ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/icons ] && mkdir ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/icons

# Copy browsewith binay and document files
cp target/${BUILD_TARGET}/release/${EXE_NAME} ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/bin/
cp README.md ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/bin/
cp LICENSE ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/bin/

# Copy images
cp resources/browsewith.ico ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/icons/
cp resources/download.png ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/icons/
cp resources/close.png ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/icons/

# If target its Windows then copy the requried DLLs
if [ "${TARGET}" = "windows" ]; then
  for f in `cat resources/dlls.txt | tr '\' '/'`; do
    [ ! -f ${DLLS_ROOT}/$f ] && echo "Could not copy DLL file '${DLLS_ROOT}/$f'" && exit 2
    echo cp ${DLLS_ROOT}/$f ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}/$f
  done
fi

# Calculate and save sha256 file hashes
cd ${PACKAGES_ROOT}/${VERSION}/${APP_NAME}
find ./ -type f -not -name *.sha256 -exec sha256sum {} + > ${APP_NAME}.sha256

# Compress using tar for FreeBsd and Linux, and zip for Windows
cd ${PACKAGES_ROOT}/${VERSION}
[ "${TARGET}" != "windows" ] && tar zcvf ${APP_NAME}.tar.gz ${APP_NAME}/* || zip -r9 ${APP_NAME}.zip ${APP_NAME}/*
sha256sum ${APP_NAME}.tar.gz > ${APP_NAME}.sha256
