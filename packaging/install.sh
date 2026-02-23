#!/bin/bash
# MyPowerToys — remote install script
# Usage: curl -fsSL https://raw.githubusercontent.com/pedrokarim/my-power-toys/main/packaging/install.sh | bash
set -euo pipefail

REPO="pedrokarim/my-power-toys"
INSTALL_DIR="$HOME/.local/bin"
ASSETS_DIR="$HOME/.local/share/my-power-toys/assets"
ICON_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"
AUTOSTART_DIR="$HOME/.config/autostart"
SYSTEMD_DIR="$HOME/.config/systemd/user"
TMPDIR="$(mktemp -d)"

cleanup() { rm -rf "$TMPDIR"; }
trap cleanup EXIT

echo "==> MyPowerToys installer"
echo ""

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
    aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
    *)
        echo "Error: unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Fetch latest release version from GitHub API
echo "==> Fetching latest release..."
RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest")"
VERSION="$(echo "$RELEASE_JSON" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"v\?\([^"]*\)".*/\1/')"

if [ -z "$VERSION" ]; then
    echo "Error: could not determine latest version."
    echo "Check https://github.com/$REPO/releases"
    exit 1
fi

echo "    Latest version: v$VERSION"

# Download release archive
ASSET_NAME="my-power-toys-${VERSION}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/$REPO/releases/download/v${VERSION}/${ASSET_NAME}"

echo "==> Downloading $ASSET_NAME..."
if ! curl -fSL --progress-bar -o "$TMPDIR/release.tar.gz" "$DOWNLOAD_URL"; then
    echo "Error: failed to download $DOWNLOAD_URL"
    echo "Check that the release exists at https://github.com/$REPO/releases"
    exit 1
fi

# Extract
echo "==> Extracting..."
tar xzf "$TMPDIR/release.tar.gz" -C "$TMPDIR"

# Install binaries
echo "==> Installing binaries to $INSTALL_DIR..."
mkdir -p "$INSTALL_DIR"
for bin in mpt-daemon mpt-ctl mpt-settings mpt-color-picker mpt-command-palette mpt-peek mpt-image-resizer; do
    if [ -f "$TMPDIR/$bin" ]; then
        install -m 755 "$TMPDIR/$bin" "$INSTALL_DIR/$bin"
        echo "    Installed $bin"
    fi
done

# Install assets
echo "==> Installing assets to $ASSETS_DIR..."
if [ -d "$TMPDIR/assets" ]; then
    mkdir -p "$ASSETS_DIR"
    cp -r "$TMPDIR/assets/icons" "$ASSETS_DIR/"
    cp -r "$TMPDIR/assets/banners" "$ASSETS_DIR/"
    cp -r "$TMPDIR/assets/backgrounds" "$ASSETS_DIR/"
    cp -f "$TMPDIR/assets/logo-200.png" "$ASSETS_DIR/" 2>/dev/null || true
    cp -f "$TMPDIR/assets/logo.png" "$ASSETS_DIR/" 2>/dev/null || true
    echo "    Assets installed"
else
    echo "    WARNING: No assets directory found in archive."
    echo "    The UI may not display images correctly."
fi

# Check PATH
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        echo "    WARNING: $INSTALL_DIR is not in your PATH."
        echo "    Add this to your ~/.bashrc or ~/.zshrc:"
        echo "      export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        ;;
esac

# Download and install icon
echo "==> Installing icon..."
mkdir -p "$ICON_DIR"
curl -fsSL "https://raw.githubusercontent.com/$REPO/main/assets/icons/icon-128.png" \
    -o "$ICON_DIR/my-power-toys.png" 2>/dev/null || true

# Setup desktop autostart
echo "==> Setting up autostart..."
mkdir -p "$AUTOSTART_DIR"
cat > "$AUTOSTART_DIR/my-power-toys-daemon.desktop" << 'DESKTOP'
[Desktop Entry]
Type=Application
Name=MyPowerToys Daemon
Comment=Background daemon for MyPowerToys
Exec=mpt-daemon
Icon=my-power-toys
Terminal=false
Categories=Utility;System;
NoDisplay=true
X-GNOME-Autostart-enabled=true
X-KDE-autostart-after=panel
DESKTOP

# Setup systemd user service
echo "==> Setting up systemd user service..."
mkdir -p "$SYSTEMD_DIR"
cat > "$SYSTEMD_DIR/my-power-toys.service" << SYSTEMD
[Unit]
Description=MyPowerToys Daemon
Documentation=https://github.com/$REPO
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=$INSTALL_DIR/mpt-daemon
Restart=on-failure
RestartSec=5

[Install]
WantedBy=graphical-session.target
SYSTEMD
systemctl --user daemon-reload 2>/dev/null || true

# Enable and (re)start daemon automatically
echo "==> Enabling and starting daemon..."
if systemctl --user enable my-power-toys.service 2>/dev/null; then
    # Restart to pick up the new binary (start alone won't replace a running daemon)
    systemctl --user restart my-power-toys.service 2>/dev/null
    echo "    Daemon (re)started via systemd"
else
    echo "    WARNING: Could not enable systemd service."
    echo "    The daemon will still autostart at login via desktop autostart."
    echo "    You can start it manually: mpt-daemon"
fi

# Install application menu entry for mpt-settings
APP_DIR="$HOME/.local/share/applications"
echo "==> Installing application menu entry..."
mkdir -p "$APP_DIR"
cat > "$APP_DIR/my-power-toys.desktop" << 'DESKTOP'
[Desktop Entry]
Type=Application
Name=MyPowerToys
GenericName=System Utilities
Comment=Suite of system utilities for Linux, inspired by Microsoft PowerToys
Exec=mpt-settings
Icon=my-power-toys
Terminal=false
Categories=Utility;System;Settings;
Keywords=powertoys;utilities;tiling;launcher;
StartupNotify=true
DESKTOP

# Update desktop database if available
update-desktop-database "$APP_DIR" 2>/dev/null || true

echo ""
echo "  MyPowerToys v$VERSION installed successfully!"
echo ""
echo "  The daemon is running in the background."
echo "  Open settings:      mpt-settings (or find 'MyPowerToys' in your app menu)"
echo "  CLI control:        mpt-ctl list"
echo "  Daemon status:      systemctl --user status my-power-toys"
echo ""
