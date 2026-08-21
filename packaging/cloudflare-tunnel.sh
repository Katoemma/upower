#!/usr/bin/env bash
# Quick Cloudflare tunnel to local power-monitor (127.0.0.1:8765).
set -euo pipefail

DEFAULT_APP_URL="https://hostels-rolling-lol-films.trycloudflare.com"

if ! command -v cloudflared >/dev/null 2>&1; then
  echo "cloudflared not found. Install with:"
  echo "  echo 'deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] https://pkg.cloudflare.com/cloudflared any main' | sudo tee /etc/apt/sources.list.d/cloudflared.list"
  echo "  sudo apt-get update && sudo apt-get install -y cloudflared"
  exit 1
fi

echo "cloudflared $(cloudflared --version 2>&1 | head -1)"
echo "Probing local API..."
code="$(curl -sS -o /dev/null -w '%{http_code}' http://127.0.0.1:8765/api/v1/power || true)"
if [[ "$code" == "000" ]]; then
  echo "WARNING: nothing responding on 127.0.0.1:8765 — start power-monitor daemon first."
else
  echo "Local API HTTP $code (401 with seeded users is OK)."
fi

echo
echo "App Server URL (update Settings if tunnel hostname rotates):"
echo "  $DEFAULT_APP_URL"
echo "Auth remains required (JWT). Starting quick tunnel..."
echo

exec cloudflared tunnel --url http://127.0.0.1:8765
