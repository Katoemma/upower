#!/usr/bin/env bash
set -euo pipefail

PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

echo "==> Building power-monitor (release)"
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"

echo "==> Installing binary to $BIN_DIR"
mkdir -p "$BIN_DIR"
install -m 755 "$REPO_ROOT/target/release/power-monitor" "$BIN_DIR/power-monitor"

echo "==> Installing user systemd unit"
mkdir -p "$UNIT_DIR"
install -m 644 "$REPO_ROOT/packaging/power-monitor.user.service" \
  "$UNIT_DIR/power-monitor.service"

# Ensure config exists
CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/power-monitor"
mkdir -p "$CONFIG_DIR"
if [[ ! -f "$CONFIG_DIR/config.toml" ]]; then
  install -m 644 "$REPO_ROOT/packaging/config.toml.example" "$CONFIG_DIR/config.toml"
  echo "    wrote $CONFIG_DIR/config.toml"
fi

systemctl --user daemon-reload
systemctl --user enable --now power-monitor.service

echo
echo "Installed. Useful commands:"
echo "  systemctl --user status power-monitor"
echo "  journalctl --user -u power-monitor -f"
echo "  power-monitor status"
echo "  curl http://127.0.0.1:8765/api/v1/power"
