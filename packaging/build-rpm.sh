#!/bin/bash
# Build an RPM package for MyPowerToys using rpmbuild
set -euo pipefail

VERSION="${1:-0.1.0}"

echo "==> Building release binaries..."
cargo build --release

RPMBUILD_DIR="${HOME}/rpmbuild"
mkdir -p "${RPMBUILD_DIR}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

cat > "${RPMBUILD_DIR}/SPECS/my-power-toys.spec" << EOF
Name:           my-power-toys
Version:        ${VERSION}
Release:        1%{?dist}
Summary:        System utilities for Linux inspired by Microsoft PowerToys

License:        GPL-3.0-only
URL:            https://github.com/pedrokarim/my-power-toys

Requires:       dbus

%description
MyPowerToys provides a suite of tools including window tiling,
app launcher, color picker, keyboard remapping, bulk rename,
image resizer, screen ruler, OCR text extraction, and more.

%install
install -Dm755 %{_builddir}/../../target/release/mpt-daemon %{buildroot}%{_bindir}/mpt-daemon
install -Dm755 %{_builddir}/../../target/release/mpt-settings %{buildroot}%{_bindir}/mpt-settings
install -Dm755 %{_builddir}/../../target/release/mpt-ctl %{buildroot}%{_bindir}/mpt-ctl
install -Dm644 %{_builddir}/../../assets/my-power-toys.desktop %{buildroot}%{_datadir}/applications/my-power-toys.desktop
install -Dm644 %{_builddir}/../../assets/my-power-toys-daemon.desktop %{buildroot}%{_datadir}/applications/my-power-toys-daemon.desktop
install -Dm644 %{_builddir}/../../assets/my-power-toys.service %{buildroot}%{_userunitdir}/my-power-toys.service
install -Dm644 %{_builddir}/../../assets/icons/icon-128.png %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/my-power-toys.png

%files
%{_bindir}/mpt-daemon
%{_bindir}/mpt-settings
%{_bindir}/mpt-ctl
%{_datadir}/applications/my-power-toys.desktop
%{_datadir}/applications/my-power-toys-daemon.desktop
%{_userunitdir}/my-power-toys.service
%{_datadir}/icons/hicolor/128x128/apps/my-power-toys.png

%post
echo "MyPowerToys installed. Enable the daemon with:"
echo "  systemctl --user enable --now my-power-toys.service"
EOF

echo "==> Building RPM..."
rpmbuild -bb "${RPMBUILD_DIR}/SPECS/my-power-toys.spec"
echo "==> Done. Check ${RPMBUILD_DIR}/RPMS/ for the package."
