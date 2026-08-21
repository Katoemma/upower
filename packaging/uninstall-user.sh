#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"

echo "==> Stopping user service (if running)"
systemctl --user disable --now power-monitor.service 2>/dev/null || true
rm -f "$UNIT_DIR/power-monitor.service"
systemctl --user daemon-reload 2>/dev/null || true

echo "==> Removing binary"
rm -f "$BIN_DIR/power-monitor"

echo "Uninstalled service and binary."
echo "Config/data left in place:"
echo "  ${XDG_CONFIG_HOME:-$HOME/.config}/power-monitor/"
echo "  ${XDG_DATA_HOME:-$HOME/.local/share}/power-monitor/"
echo "Remove those directories manually if desired."
