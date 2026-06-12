#!/bin/bash
# MyPowerToys — remote install script
# Usage: curl -fsSL https://raw.githubusercontent.com/pedrokarim/my-power-toys/main/packaging/install.sh | bash
set -euo pipefail

REPO="pedrokarim/my-power-toys"
INSTALL_DIR="$HOME/.local/bin"
ASSETS_DIR="$HOME/.local/share/my-power-toys/assets"
ICON_DIR="$HOME/.local/share/icons/hicolor/128x128/apps"
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
for bin in mpt-daemon mpt-ctl mpt-settings mpt-color-picker mpt-command-palette mpt-peek mpt-image-resizer mpt-bulk-rename mpt-hosts-editor mpt-fancy-zones mpt-quick-accent mpt-workspaces; do
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

# Remove any legacy XDG autostart entry from previous installs
# (the daemon is now managed exclusively by the systemd user service to avoid
# duplicate launches racing on the D-Bus singleton)
rm -f "$HOME/.config/autostart/my-power-toys-daemon.desktop"

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
# Exit code 1 means another daemon instance is already holding the D-Bus name —
# that's a normal condition, not a failure, so don't restart on it.
RestartPreventExitStatus=1

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
    echo "    You can start the daemon manually: mpt-daemon"
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

# Install file manager integration for Bulk Rename
CONTEXT_MENUS_URL="https://raw.githubusercontent.com/$REPO/main/packaging/context-menus"
echo "==> Installing file manager integration for Bulk Rename..."
if command -v nautilus >/dev/null 2>&1; then
    # Nautilus Python extension
    NAUTILUS_EXT_DIR="$HOME/.local/share/nautilus-python/extensions"
    mkdir -p "$NAUTILUS_EXT_DIR"
    curl -fsSL "$CONTEXT_MENUS_URL/mpt-bulk-rename-nautilus.py" \
        -o "$NAUTILUS_EXT_DIR/mpt-bulk-rename-nautilus.py" 2>/dev/null || true
    # Nautilus script fallback
    NAUTILUS_SCRIPTS_DIR="$HOME/.local/share/nautilus/scripts"
    mkdir -p "$NAUTILUS_SCRIPTS_DIR"
    curl -fsSL "$CONTEXT_MENUS_URL/mpt-bulk-rename-nautilus-script.sh" \
        -o "$NAUTILUS_SCRIPTS_DIR/Bulk Rename (MyPowerToys)" 2>/dev/null || true
    chmod +x "$NAUTILUS_SCRIPTS_DIR/Bulk Rename (MyPowerToys)" 2>/dev/null || true
    echo "    Installed Nautilus integration"
fi
if command -v dolphin >/dev/null 2>&1; then
    DOLPHIN_DIR="$HOME/.local/share/kio/servicemenus"
    mkdir -p "$DOLPHIN_DIR"
    curl -fsSL "$CONTEXT_MENUS_URL/mpt-bulk-rename.desktop" \
        -o "$DOLPHIN_DIR/mpt-bulk-rename.desktop" 2>/dev/null || true
    echo "    Installed Dolphin integration"
fi
if command -v nemo >/dev/null 2>&1; then
    NEMO_DIR="$HOME/.local/share/nemo/actions"
    mkdir -p "$NEMO_DIR"
    curl -fsSL "$CONTEXT_MENUS_URL/mpt-bulk-rename.nemo_action" \
        -o "$NEMO_DIR/mpt-bulk-rename.nemo_action" 2>/dev/null || true
    echo "    Installed Nemo integration"
fi

echo ""
echo "  MyPowerToys v$VERSION installed successfully!"
echo ""
echo "  The daemon is running in the background."
echo "  Open settings:      mpt-settings (or find 'MyPowerToys' in your app menu)"
echo "  CLI control:        mpt-ctl list"
echo "  Daemon status:      systemctl --user status my-power-toys"
echo ""
