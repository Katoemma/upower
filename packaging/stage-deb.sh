#!/usr/bin/env bash
# Minimal helper to stage a .deb-like layout (for packaging Phase 6).
# Full Debian packaging can wrap this with dpkg-deb.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
STAGE="${1:-$ROOT/target/deb-stage}"
VERSION="${VERSION:-0.1.0}"

cargo build --release --manifest-path "$ROOT/Cargo.toml"

rm -rf "$STAGE"
mkdir -p "$STAGE/usr/bin" \
  "$STAGE/lib/systemd/system" \
  "$STAGE/usr/lib/systemd/user" \
  "$STAGE/etc/power-monitor" \
  "$STAGE/usr/share/doc/power-monitor" \
  "$STAGE/DEBIAN"

install -m 755 "$ROOT/target/release/power-monitor" "$STAGE/usr/bin/power-monitor"
install -m 644 "$ROOT/packaging/power-monitor.service" "$STAGE/lib/systemd/system/power-monitor.service"
install -m 644 "$ROOT/packaging/power-monitor.user.service" "$STAGE/usr/lib/systemd/user/power-monitor.service"
install -m 644 "$ROOT/packaging/config.toml.example" "$STAGE/etc/power-monitor/config.toml"
install -m 644 "$ROOT/README.md" "$STAGE/usr/share/doc/power-monitor/README.md"

cat > "$STAGE/DEBIAN/control" <<EOF
Package: power-monitor
Version: $VERSION
Section: utils
Priority: optional
Architecture: amd64
Depends: libdbus-1-3, libgcc-s1
Maintainer: Local <local@localhost>
Description: Linux power monitoring daemon with localhost API
 Monitors AC/battery via UPower, records events, notifies, and exposes REST/WebSocket.
EOF

echo "Staged at $STAGE"
echo "Build .deb with: dpkg-deb --build $STAGE power-monitor_${VERSION}_amd64.deb"
