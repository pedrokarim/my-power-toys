#!/bin/bash
# Simple install script — build and install to ~/.cargo/bin + setup autostart
set -euo pipefail

echo "==> Building MyPowerToys (release)..."
cargo build --release

echo "==> Installing binaries to ~/.cargo/bin..."
cp target/release/mpt-daemon ~/.cargo/bin/
cp target/release/mpt-settings ~/.cargo/bin/
cp target/release/mpt-ctl ~/.cargo/bin/

echo "==> Installing icon..."
mkdir -p ~/.local/share/icons/hicolor/128x128/apps
cp assets/icons/icon-128.png ~/.local/share/icons/hicolor/128x128/apps/my-power-toys.png

echo "==> Setting up autostart..."
mkdir -p ~/.config/autostart
cp assets/my-power-toys-daemon.desktop ~/.config/autostart/

echo "==> Setting up systemd user service..."
mkdir -p ~/.config/systemd/user
cp assets/my-power-toys.service ~/.config/systemd/user/
systemctl --user daemon-reload

echo ""
echo "Installation complete!"
echo ""
echo "  Start the daemon:   mpt-daemon"
echo "  Open settings:      mpt-settings"
echo "  CLI control:        mpt-ctl list"
echo ""
echo "  Or use systemd:     systemctl --user enable --now my-power-toys.service"
