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

Optional. Copy `.env.example` to `.env`:

```bash
cp .env.example .env
```

**Required by Brevo:** `SMTP_FROM_ADDRESS` must be a **verified sender** in Brevo
(Settings → Senders), e.g. `emmanuelkato39@gmail.com`.  
Do **not** put `*@smtp-brevo.com` in From — that is only the SMTP login (`SMTP_USER`).

Laravel-style names also work: `MAIL_USERNAME`, `MAIL_PASSWORD`, `MAIL_FROM_ADDRESS`, `MAIL_FROM_NAME`.

Default recipient is `nativesenior@gmail.com` (`SMTP_TO`). Email events are configured under `[email]` in `config.toml` (AC disconnect / low / critical by default).

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
