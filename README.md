# power-monitor

Linux-native Rust daemon that monitors AC/battery via UPower (D-Bus), records events in SQLite, shows desktop notifications, and exposes a localhost REST + WebSocket API.

## Requirements

- Linux with UPower, D-Bus, systemd
- Rust 1.75+ (to build)

## Quick start

```bash
cargo run -- daemon
```

In another terminal:

```bash
curl http://127.0.0.1:8765/api/v1/power
power-monitor status   # after install, or: cargo run -- status
```

## Install (user service)

```bash
chmod +x packaging/install-user.sh packaging/uninstall-user.sh
./packaging/install-user.sh
```

## Configuration

Default path: `~/.config/power-monitor/config.toml`

See `packaging/config.toml.example`. The API binds to `127.0.0.1` by default.

### Email (Brevo SMTP)

Optional. Copy `.env.example` to `.env` in the project directory (or export the same vars for the systemd unit):

```bash
cp .env.example .env
# edit SMTP_USER / SMTP_PASSWORD / SMTP_FROM
```

Default recipient is `nativesenior@gmail.com` (`SMTP_TO`). Emails are sent for AC disconnect, low battery, and critical battery by default (see `[email]` in config). Desktop notifications stay independent under `[notifications]`.

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/power` | Current power + battery summary |
| GET | `/api/v1/battery` | Battery details |
| GET | `/api/v1/power/status` | AC connected / source |
| GET | `/api/v1/events` | Event history (`page`, `limit`, `type`, `from`, `to`) |
| WS | `/ws` | Real-time power events |

## CLI

```bash
power-monitor daemon
power-monitor status
power-monitor events --last 20
power-monitor config
power-monitor version
```

## Uninstall

```bash
./packaging/uninstall-user.sh
```
