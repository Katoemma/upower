# power-monitor — build / start / restart

Run these from the **repo root** (`upower/`) so `.env` is picked up (SMTP + Firebase).

## Build

```bash
# Debug (fast iterate)
cargo build

# Release (for install / daily use)
cargo build --release
```

Binary paths:

- Debug: `target/debug/power-monitor`
- Release: `target/release/power-monitor`

Optional install (user systemd unit + `~/.local/bin/power-monitor`):

```bash
./packaging/install-user.sh
```

## Start

**Dev (foreground, loads `.env` from cwd):**

```bash
cd /home/lenovo/Desktop/server-tools/upower
RUST_LOG=info cargo run -- daemon
```

Or with an already-built binary:

```bash
cd /home/lenovo/Desktop/server-tools/upower
RUST_LOG=info ./target/debug/power-monitor daemon
# release:
RUST_LOG=info ./target/release/power-monitor daemon
```

**Background (no systemd):**

```bash
cd /home/lenovo/Desktop/server-tools/upower
RUST_LOG=info ./target/release/power-monitor daemon > /tmp/power-monitor.log 2>&1 &
```

**User systemd (after install script):**

```bash
systemctl --user start power-monitor
systemctl --user status power-monitor
```

API listens on `http://127.0.0.1:8765` by default.

## Restart

**If running via cargo / binary (no systemd):**

```bash
pkill -f 'power-monitor daemon' || true
# free the port if something is stuck
fuser -k 8765/tcp 2>/dev/null || true

cd /home/lenovo/Desktop/server-tools/upower
RUST_LOG=info ./target/release/power-monitor daemon
# or: RUST_LOG=info cargo run -- daemon
```

**If installed as user service:**

```bash
systemctl --user restart power-monitor
journalctl --user -u power-monitor -f
```

Restart after changing `.env`, seeding users, or rotating Firebase credentials.

## Smoke checks

```bash
curl -sS -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8765/api/v1/power
# 401 = API up + auth required (users seeded)
# 200 = API up + no users yet

./target/release/power-monitor status   # needs a saved token, or:
./target/release/power-monitor login you@example.com 'password'
./target/release/power-monitor status
```

## Phone testing (optional tunnel)

Keep the daemon on localhost, then:

```bash
bash packaging/cloudflare-tunnel.sh
# or: cloudflared tunnel --url http://127.0.0.1:8765
```

Paste the printed `https://….trycloudflare.com` URL into the Flutter app Server URL if it changes.
