#!/usr/bin/env bash
# Copy secrets + runtime data from THIS machine (test/dev box) to the homelab.
# Run on the test machine, NOT on the homelab:
#   ./packaging/copy-to-homelab.sh user@192.168.100.37
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 user@HOMELAB_IP"
  echo "Example: $0 user@192.168.100.37"
  exit 1
fi

HOMELAB="$1"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ENV="$REPO_ROOT/.env"
LOCAL_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/power-monitor"
LOCAL_DATA="${XDG_DATA_HOME:-$HOME/.local/share}/power-monitor"
REMOTE_USER="${HOMELAB#*@}"
REMOTE_USER="${REMOTE_USER%%:*}"

echo "==> Target: $HOMELAB"
echo "==> Repo:   $REPO_ROOT"

missing=0
if [[ ! -f "$REPO_ENV" ]]; then
  echo "ERROR: missing $REPO_ENV"
  missing=1
fi
if [[ ! -f "$LOCAL_CONFIG/firebase-service-account.json" ]]; then
  echo "ERROR: missing $LOCAL_CONFIG/firebase-service-account.json"
  missing=1
fi
if [[ "$missing" -ne 0 ]]; then
  exit 1
fi

echo "==> Creating remote directories"
ssh "$HOMELAB" "mkdir -p ~/.config/power-monitor ~/.local/share/power-monitor && chmod 700 ~/.config/power-monitor"

echo "==> Copying .env (repo → remote systemd path)"
scp "$REPO_ENV" "$HOMELAB:~/.config/power-monitor/.env"

echo "==> Copying Firebase service account"
scp "$LOCAL_CONFIG/firebase-service-account.json" \
  "$HOMELAB:~/.config/power-monitor/"

if [[ -f "$LOCAL_CONFIG/config.toml" ]]; then
  echo "==> Copying config.toml"
  scp "$LOCAL_CONFIG/config.toml" "$HOMELAB:~/.config/power-monitor/"
fi

if [[ -d "$LOCAL_DATA" ]] && [[ -n "$(ls -A "$LOCAL_DATA" 2>/dev/null || true)" ]]; then
  echo "==> Copying data dir (SQLite, users, FCM tokens)"
  rsync -av "$LOCAL_DATA/" "$HOMELAB:~/.local/share/power-monitor/"
else
  echo "==> No local data dir to copy (fresh user seed on homelab)"
fi

echo "==> Fixing FIREBASE_CREDENTIALS path for remote user ($REMOTE_USER)"
ssh "$HOMELAB" "python3 - <<'PY'
from pathlib import Path
import re
p = Path.home() / '.config/power-monitor/.env'
text = p.read_text()
cred = str(Path.home() / '.config/power-monitor/firebase-service-account.json')
text = re.sub(r'^FIREBASE_CREDENTIALS=.*$', f'FIREBASE_CREDENTIALS={cred}', text, flags=re.M)
p.write_text(text)
PY
chmod 600 ~/.config/power-monitor/.env ~/.config/power-monitor/firebase-service-account.json 2>/dev/null || true
"

echo "==> Restarting remote daemon"
ssh "$HOMELAB" "systemctl --user restart power-monitor && sleep 1 && systemctl --user is-active power-monitor"

echo
echo "Done. On homelab, verify:"
echo "  ssh $HOMELAB 'journalctl --user -u power-monitor -n 15 --no-pager'"
echo "  ssh $HOMELAB 'curl -sS -o /dev/null -w \"%{http_code}\\n\" http://127.0.0.1:8765/api/v1/power'"
