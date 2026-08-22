# Deploy Astra on Ubuntu homelab

Move the **power-monitor** daemon from your test machine to a real always-on Ubuntu homelab (ThinkPad, mini PC, etc.). The API stays on `127.0.0.1:8765`; reach it from your phone via **Cloudflare tunnel** (or Tailscale). The Flutter app (**Astra**) connects with JWT login.

---

## Architecture

```
┌─────────────────┐     HTTPS/WSS      ┌──────────────────┐
│  Astra (phone)  │ ◄────────────────► │  cloudflared     │
└─────────────────┘                    │  (quick or named)│
                                         └────────┬─────────┘
                                                  │ localhost
                                         ┌────────▼─────────┐
                                         │ power-monitor    │
                                         │ :8765            │
                                         ├──────────────────┤
                                         │ UPower / D-Bus   │
                                         │ sysinfo telemetry│
                                         │ SQLite + JWT     │
                                         │ SMTP + FCM       │
                                         └──────────────────┘
```

| Path | Purpose |
|------|---------|
| `~/.local/bin/power-monitor` | Daemon binary (user install) |
| `~/.config/power-monitor/config.toml` | Intervals, alerts, API bind |
| `~/.config/power-monitor/.env` | SMTP + Firebase secrets (loaded by systemd) |
| `~/.config/power-monitor/firebase-service-account.json` | FCM service account (optional) |
| `~/.local/share/power-monitor/` | SQLite DB, users, FCM token file |
| `~/.config/power-monitor/token` | CLI JWT (from `power-monitor login`) |

---

## 1. Homelab prerequisites

On the **homelab host** (Ubuntu 22.04+ recommended):

```bash
sudo apt update
sudo apt install -y \
  upower dbus dbus-user-session \
  build-essential pkg-config libdbus-1-dev \
  curl git

# Rust (if building on the homelab)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc --version   # need 1.75+
```

Check UPower sees your machine:

```bash
upower -i /org/freedesktop/UPower/devices/battery_BAT0 2>/dev/null || upower -e
```

For a **headless** homelab (no graphical login), enable user systemd services at boot:

```bash
sudo loginctl enable-linger "$USER"
```

---

## 2. Get the code on the homelab

**Option A — git clone (recommended)**

```bash
git clone <your-repo-url> ~/upower
cd ~/upower
```

**Option B — copy from test machine**

From the test machine:

```bash
rsync -av --exclude target --exclude mobile/build \
  /home/lenovo/Desktop/server-tools/upower/ \
  homelab-user@HOMELAB_IP:~/upower/
```

**Option C — copy only the release binary**

On the test machine:

```bash
cd ~/upower   # or your repo path
cargo build --release
scp target/release/power-monitor homelab-user@HOMELAB_IP:~/.local/bin/
```

Then on the homelab, skip to [§4 Configuration](#4-configuration) if you only copied the binary.

---

## 3. Build and install (user service)

On the homelab:

```bash
cd ~/upower
chmod +x packaging/install-user.sh packaging/uninstall-user.sh
./packaging/install-user.sh
```

This will:

1. `cargo build --release`
2. Install `~/.local/bin/power-monitor`
3. Install `~/.config/systemd/user/power-monitor.service`
4. Create `~/.config/power-monitor/config.toml` from the example if missing
5. `systemctl --user enable --now power-monitor`

The user unit loads secrets from `~/.config/power-monitor/.env` via `EnvironmentFile`.

**Alternative — system-wide service (optional)**

```bash
cd ~/upower
bash packaging/stage-deb.sh
sudo dpkg-deb --build target/deb-stage power-monitor_0.1.0_amd64.deb
sudo dpkg -i power-monitor_0.1.0_amd64.deb
sudo systemctl enable --now power-monitor
```

For system-wide installs, put env vars in `/etc/default/power-monitor` or a systemd drop-in and point `EnvironmentFile` there.

---

## 4. Configuration

Edit the homelab config:

```bash
nano ~/.config/power-monitor/config.toml
```

Start from `packaging/config.toml.example`. Important sections:

```toml
[server]
host = "127.0.0.1"   # keep localhost; expose via tunnel only
port = 8765

[monitoring]
memory_interval_ms = 1000
cpu_interval_ms = 1000
storage_interval_ms = 5000
processes_interval_ms = 2000
process_limit = 20
```

Tune `[email]`, `[push]`, and `[battery]` thresholds as needed.

---

## 5. Secrets (SMTP + Firebase)

Create env file on the **homelab** (not the repo root — systemd reads this path):

```bash
cp ~/upower/.env.example ~/.config/power-monitor/.env
chmod 600 ~/.config/power-monitor/.env
nano ~/.config/power-monitor/.env
```

**Brevo email** — required fields:

```bash
SMTP_HOST=smtp-relay.brevo.com
SMTP_PORT=587
SMTP_USER=your-login@smtp-brevo.com
SMTP_PASSWORD=your-brevo-smtp-key
SMTP_FROM_ADDRESS=verified-sender@example.com   # must be verified in Brevo → Senders
SMTP_FROM_NAME=Astra
SMTP_TO=katoemmy001@gmail.com
```

**Firebase push (FCM HTTP v1)** — copy the service account JSON from the test machine:

```bash
# On test machine
scp ~/.config/power-monitor/firebase-service-account.json \
  homelab-user@HOMELAB_IP:~/.config/power-monitor/
```

On the homelab `.env`:

```bash
FIREBASE_CREDENTIALS=/home/YOUR_USER/.config/power-monitor/firebase-service-account.json
FIREBASE_PROJECT_ID=native-server
```

Use absolute paths. Restart after any secret change:

```bash
systemctl --user restart power-monitor
```

---

## 6. Seed API users

No public signup. Create at least one user on the homelab:

```bash
power-monitor user add katoemmy001@gmail.com 'your-secure-password'
power-monitor user list
systemctl --user restart power-monitor   # auth engages once users exist
```

Login from CLI (optional smoke test):

```bash
power-monitor login katoemmy001@gmail.com 'your-secure-password'
power-monitor status
```

---

## 7. Verify the daemon locally

```bash
systemctl --user status power-monitor
journalctl --user -u power-monitor -f
```

```bash
# 401 = API up + auth required (expected after seeding users)
curl -sS -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8765/api/v1/power

# With token from `power-monitor login`:
TOKEN=$(cat ~/.config/power-monitor/token 2>/dev/null || true)
curl -sS -H "Authorization: Bearer $TOKEN" http://127.0.0.1:8765/api/v1/system | head -c 400
```

---

## 8. Remote access (phone / off-LAN)

The API binds to **localhost only**. Use a tunnel so the phone never needs your homelab IP or open firewall ports.

### Quick tunnel (testing / simple homelab)

Install cloudflared on the homelab:

```bash
curl -fsSL https://pkg.cloudflare.com/cloudflare-main.gpg | sudo tee /usr/share/keyrings/cloudflare-main.gpg >/dev/null
echo 'deb [signed-by=/usr/share/keyrings/cloudflare-main.gpg] https://pkg.cloudflare.com/cloudflared any main' | \
  sudo tee /etc/apt/sources.list.d/cloudflared.list
sudo apt update && sudo apt install -y cloudflared
```

Run (daemon must already be up):

```bash
cd ~/upower
bash packaging/cloudflare-tunnel.sh
# or: cloudflared tunnel --url http://127.0.0.1:8765
```

Copy the printed `https://….trycloudflare.com` URL. **Quick tunnel hostnames change every restart** — update the app when it rotates.

**Run tunnel at boot (user systemd example)**

```bash
mkdir -p ~/.config/systemd/user
cat > ~/.config/systemd/user/upower-tunnel.service <<'EOF'
[Unit]
Description=Cloudflare quick tunnel to power-monitor
After=power-monitor.service
Requires=power-monitor.service

[Service]
ExecStart=/usr/bin/cloudflared tunnel --url http://127.0.0.1:8765
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now upower-tunnel
journalctl --user -u upower-tunnel -f   # read the new public URL from logs
```

### Stable URL (recommended for production)

Use a [Cloudflare named tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/) with your own subdomain (e.g. `upower.yourdomain.com`), or **Tailscale** / WireGuard and set the app Server URL to the homelab’s tailnet IP (`http://100.x.x.x:8765` only if you change `[server] host` — not recommended; prefer tunnel or tailscale serve).

---

## 9. Mobile app (Astra)

On your dev machine (or CI), build/install the APK as you do today. No rebuild required for a new server — only **Settings → Server URL**.

1. Install/open **Astra** on the phone.
2. **Settings → Server URL** → paste the homelab tunnel URL (no trailing slash).
3. **Save**, then log in with the homelab user (`katoemmy001@gmail.com` / password from §6).
4. FCM registers automatically after login (`POST /api/v1/push/tokens`).

Test push from the homelab:

```bash
power-monitor push-token list
power-monitor push-token test --email katoemmy001@gmail.com
```

Pull-to-refresh on Home loads `GET /api/v1/system`; live gauges use `WSS /api/v1/stream?token=…`.

---

## 10. Migrate state from the test machine (optional)

To keep event history, users, and FCM tokens:

```bash
# On test machine — stop daemon first
pkill -f 'power-monitor daemon' || true

rsync -av \
  ~/.config/power-monitor/ \
  homelab-user@HOMELAB_IP:~/.config/power-monitor/

rsync -av \
  ~/.local/share/power-monitor/ \
  homelab-user@HOMELAB_IP:~/.local/share/power-monitor/
```

On the homelab:

```bash
systemctl --user restart power-monitor
```

**Fresh start instead:** skip rsync; only copy `.env` + Firebase JSON and re-run `user add` + phone login.

---

## 11. Shut down the test instance

On the old test machine:

```bash
systemctl --user stop power-monitor upower-tunnel 2>/dev/null || true
pkill -f 'power-monitor daemon' || true
pkill cloudflared || true
```

Update the phone Server URL to the homelab tunnel so you are not hitting the old host.

---

## 12. Day-2 operations

| Task | Command |
|------|---------|
| Status | `systemctl --user status power-monitor` |
| Logs | `journalctl --user -u power-monitor -f` |
| Restart | `systemctl --user restart power-monitor` |
| Config path | `power-monitor config` |
| Events | `power-monitor events --last 20` |
| Change password | `power-monitor user set-password email@x.com 'new'` |
| Uninstall | `./packaging/uninstall-user.sh` |

**Update after `git pull`:**

```bash
cd ~/upower
git pull
./packaging/install-user.sh    # rebuilds + restarts user service
```

**Dev foreground run** (loads `.env` from repo cwd — not for production):

```bash
cd ~/upower
cp .env.example .env && nano .env
RUST_LOG=info cargo run -- daemon
```

See also [start.md](start.md) for local build/restart notes.

---

## 13. Troubleshooting

| Symptom | Fix |
|---------|-----|
| `401` on curl | Expected when users exist; login or pass `Authorization: Bearer` |
| Email not sending | Check Brevo sender verification; `journalctl --user -u power-monitor` for SMTP errors |
| Push not working | Confirm `FIREBASE_CREDENTIALS` path in `.env`; re-login on phone to re-register token |
| Phone can’t connect | Tunnel running? URL in Settings matches current `trycloudflare.com` hostname? |
| Service not starting at boot | `loginctl enable-linger $USER`; `systemctl --user enable power-monitor` |
| Port in use | `fuser -k 8765/tcp`; restart service |
| Stale `SMTP_TO` in shell | Systemd uses `~/.config/power-monitor/.env`, not your interactive shell |

---

## Quick checklist

- [ ] UPower + Rust/build deps installed
- [ ] `loginctl enable-linger` (headless)
- [ ] `./packaging/install-user.sh` on homelab
- [ ] `~/.config/power-monitor/config.toml` tuned
- [ ] `~/.config/power-monitor/.env` + Firebase JSON in place
- [ ] `power-monitor user add …` and service restarted
- [ ] Local curl returns 401 or 200
- [ ] Cloudflare tunnel (or stable alternative) running
- [ ] Phone Settings → new Server URL → login
- [ ] Test push + Home gauges updating
- [ ] Test machine daemon/tunnel stopped
