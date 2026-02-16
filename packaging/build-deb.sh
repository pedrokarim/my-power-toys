#!/bin/bash
# Build a .deb package for MyPowerToys
set -euo pipefail

VERSION="${1:-0.1.0}"
ARCH="amd64"
PKG_NAME="my-power-toys"
BUILD_DIR="target/release"
DEB_DIR="target/deb"
PKG_DIR="${DEB_DIR}/${PKG_NAME}_${VERSION}_${ARCH}"

echo "==> Building release binaries..."
cargo build --release

echo "==> Assembling .deb package..."
rm -rf "${PKG_DIR}"
mkdir -p "${PKG_DIR}/DEBIAN"
mkdir -p "${PKG_DIR}/usr/bin"
mkdir -p "${PKG_DIR}/usr/share/applications"
mkdir -p "${PKG_DIR}/usr/lib/systemd/user"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/48x48/apps"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/128x128/apps"
mkdir -p "${PKG_DIR}/usr/share/icons/hicolor/256x256/apps"

# Binaries
cp "${BUILD_DIR}/mpt-daemon" "${PKG_DIR}/usr/bin/"
cp "${BUILD_DIR}/mpt-settings" "${PKG_DIR}/usr/bin/"
cp "${BUILD_DIR}/mpt-ctl" "${PKG_DIR}/usr/bin/"

# Desktop files
cp assets/my-power-toys.desktop "${PKG_DIR}/usr/share/applications/"
cp assets/my-power-toys-daemon.desktop "${PKG_DIR}/usr/share/applications/"

# Systemd user service
cp assets/my-power-toys.service "${PKG_DIR}/usr/lib/systemd/user/"

# Icons
cp assets/icons/icon-48.png "${PKG_DIR}/usr/share/icons/hicolor/48x48/apps/my-power-toys.png"
cp assets/icons/icon-128.png "${PKG_DIR}/usr/share/icons/hicolor/128x128/apps/my-power-toys.png"
cp assets/icons/icon-256.png "${PKG_DIR}/usr/share/icons/hicolor/256x256/apps/my-power-toys.png"

# Control file
cat > "${PKG_DIR}/DEBIAN/control" << EOF
Package: ${PKG_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libdbus-1-3
Recommends: tesseract-ocr, wl-clipboard, xdotool, imagemagick
Maintainer: Ahmed Karim <karimisma6464@gmail.com>
Description: System utilities for Linux inspired by Microsoft PowerToys
 MyPowerToys provides a suite of tools including window tiling,
 app launcher, color picker, keyboard remapping, bulk rename,
 image resizer, screen ruler, OCR text extraction, and more.
Homepage: https://github.com/pedrokarim/my-power-toys
EOF

# Post-install script
cat > "${PKG_DIR}/DEBIAN/postinst" << 'EOF'
#!/bin/bash
set -e
echo "MyPowerToys installed. Run 'mpt-daemon' or enable the systemd service:"
echo "  systemctl --user enable --now my-power-toys.service"
echo "Open settings with 'mpt-settings'"
EOF
chmod 755 "${PKG_DIR}/DEBIAN/postinst"

# Build the deb
dpkg-deb --build "${PKG_DIR}"
echo "==> Package built: ${PKG_DIR}.deb"
