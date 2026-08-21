Yes. In that case, I would define it as a **Linux-native Rust power monitoring daemon/application**, with the API as part of the product rather than thinking about Flutter at all.

Here is a requirements document you can give directly to the developer.

# Linux Power Monitor — Technical Requirements

## 1. Objective

Build a **Linux-only power monitoring application written in Rust**.

The application will run continuously in the background and monitor the computer's power and battery state.

Its primary purpose is to detect changes in power status and provide this information through a **local API**, while also supporting desktop notifications and power-event history.

The application should be designed as a native Linux service rather than as a GUI application.

---

# 2. Target Platform

### Operating system

Primary target:

* Ubuntu
* Debian-based Linux distributions

The application should ideally work on most modern Linux distributions that provide:

* D-Bus
* UPower
* systemd

### Architecture

```text
Linux
 │
 ├── systemd
 │
 └── Power Monitor Service
          │
          ├── UPower / D-Bus
          │
          ├── Power State Monitor
          │
          ├── Battery Monitor
          │
          ├── Event Manager
          │
          ├── Notification Manager
          │
          ├── Event History
          │
          └── Local REST/WebSocket API
```

---

# 3. Core Responsibilities

The service must be able to determine:

### AC power

* Is AC power connected?
* When was AC power connected?
* When was AC power disconnected?
* Detect unexpected power loss.

### Battery

* Battery percentage
* Charging state
* Battery state
* Battery health
* Battery capacity
* Battery energy
* Estimated time to full charge
* Estimated time remaining
* Battery temperature where available

### Charging

Detect:

```text
Charging
Discharging
Fully charged
Not charging
Unknown
```

---

# 4. Event Detection

The application should be **event-driven** rather than continuously polling the system unnecessarily.

For example:

```text
AC connected
      ↓
Power event detected
      ↓
Update internal state
      ↓
Record event
      ↓
Trigger notification
      ↓
Notify API clients
```

The same should happen for:

* AC disconnected
* Charging started
* Charging stopped
* Battery became full
* Battery percentage changed
* Low battery
* Critical battery

---

# 5. Power Events

The service should maintain an event history.

Example:

```text
21 Aug 2026 14:02:11
AC_CONNECTED
Battery: 63%

21 Aug 2026 16:42:19
AC_DISCONNECTED
Battery: 87%

21 Aug 2026 16:58:03
LOW_BATTERY
Battery: 20%

21 Aug 2026 17:04:31
AC_CONNECTED
Battery: 14%
```

Each event should contain at minimum:

```json
{
  "event": "ac_disconnected",
  "timestamp": "2026-08-21T16:42:19+03:00",
  "battery_percentage": 87
}
```

---

# 6. Local API

The service must expose a **localhost-only API**.

The API must not be publicly accessible by default.

Bind to:

```text
127.0.0.1
```

rather than:

```text
0.0.0.0
```

### Suggested port

For example:

```text
8765
```

The port should ideally be configurable.

---

# 7. REST API

### Current power status

```http
GET /api/v1/power
```

Example response:

```json
{
  "ac_connected": true,
  "state": "charging",
  "battery_percentage": 82,
  "battery_health": 94,
  "time_to_full_seconds": 4320,
  "time_remaining_seconds": null
}
```

---

### Battery information

```http
GET /api/v1/battery
```

Example:

```json
{
  "percentage": 82,
  "state": "charging",
  "health": 94,
  "energy_now_wh": 38.4,
  "energy_full_wh": 46.8,
  "energy_full_design_wh": 49.5,
  "temperature_celsius": 34.2
}
```

Only return fields that the hardware/UPower actually provides.

Do **not** fabricate unavailable information.

---

### Power status

```http
GET /api/v1/power/status
```

Example:

```json
{
  "connected": true,
  "source": "AC"
}
```

---

### Event history

```http
GET /api/v1/events
```

Support parameters:

```text
?page=1
&limit=50
&type=ac_disconnected
&from=...
&to=...
```

---

# 8. Real-Time API

The service should provide WebSocket support.

```text
/ws
```

When an event occurs:

```json
{
  "type": "power_event",
  "event": "ac_disconnected",
  "timestamp": "2026-08-21T16:42:19+03:00",
  "battery_percentage": 87
}
```

This allows clients to receive updates immediately without polling.

---

# 9. Notifications

The service should be capable of displaying Linux desktop notifications.

Examples:

### Charger disconnected

> **Power disconnected**
> Your computer is now running on battery. Battery: 87%

### Charger connected

> **Power connected**
> Your computer is charging. Battery: 42%

### Battery low

> **Low battery**
> Battery level has reached 20%.

### Battery critical

> **Critical battery**
> Battery level has reached 10%.

### Fully charged

> **Battery fully charged**
> Battery is at 100%.

Notifications should be configurable.

For example:

```text
Notify on AC disconnect       ON
Notify on AC connect          ON
Notify at 20%                 ON
Notify at 10%                 ON
Notify when fully charged     ON
```

---

# 10. Configuration

Configuration should be stored in a standard Linux configuration location.

For example:

```text
/etc/power-monitor/config.toml
```

Possible configuration:

```toml
[server]
host = "127.0.0.1"
port = 8765

[notifications]
enabled = true
ac_connected = true
ac_disconnected = true
fully_charged = true

[battery]
low_threshold = 20
critical_threshold = 10
```

User-specific configuration could alternatively live under:

```text
~/.config/power-monitor/
```

The developer should determine whether the service needs system-wide or user-level configuration.

---

# 11. Data Storage

The application should store power events locally.

**SQLite** would be a good choice.

Example:

```text
~/.local/share/power-monitor/power-monitor.db
```

Possible table:

```text
power_events

id
event_type
timestamp
battery_percentage
battery_state
ac_connected
created_at
```

The application should not require an external database server.

---

# 12. Service Management

The application should run as a native Linux `systemd` service.

Example:

```text
power-monitor.service
```

It should support:

```bash
systemctl start power-monitor
systemctl stop power-monitor
systemctl restart power-monitor
systemctl status power-monitor
```

The service should automatically restart if it crashes.

Example requirements:

```text
Restart=on-failure
```

It should also start automatically according to the chosen deployment model.

---

# 13. CLI

Although the main application is a daemon, a CLI would be useful.

Example:

```bash
power-monitor status
```

Output:

```text
Power:       AC Connected
Battery:     82%
State:       Charging
Health:      94%
```

Other commands:

```bash
power-monitor status
power-monitor events
power-monitor config
power-monitor version
```

Potentially:

```bash
power-monitor events --last 20
```

---

# 14. Technology Requirements

### Language

**Rust**

### Linux integration

Primary:

* UPower
* D-Bus
* systemd

### API

Recommended:

* Axum
* Tokio
* Serde

### Database

Recommended:

* SQLite
* SQLx or another suitable Rust SQLite library

### Logging

Use structured logging, preferably:

* `tracing`
* `tracing-subscriber`

Logs should be accessible through:

```bash
journalctl -u power-monitor
```

---

# 15. Security Requirements

The API must be localhost-only by default.

The application should **not expose battery/system information to the LAN or internet**.

Avoid:

```text
0.0.0.0:8765
```

unless explicitly enabled by configuration.

The application should also avoid running with unnecessary root privileges.

Where possible, the monitoring service should run with the minimum privileges required to access UPower/D-Bus and provide notifications.

---

# 16. Failure Handling

The service should gracefully handle:

* Battery temporarily unavailable
* UPower restarting
* D-Bus connection failure
* Laptop without a battery
* Desktop computer without battery
* Multiple batteries
* Multiple power supplies
* Missing temperature information
* Missing estimated time information
* Unexpected UPower changes

For example, a desktop computer without a battery should still report:

```json
{
  "ac_connected": true,
  "battery": null
}
```

rather than crashing.

---

# 17. Multiple Batteries

The implementation should not assume that every Linux machine has only:

```text
BAT0
```

It should support systems with multiple batteries where UPower exposes them.

The API could eventually return:

```json
{
  "batteries": [
    {
      "id": "BAT0",
      "percentage": 82,
      "state": "charging"
    },
    {
      "id": "BAT1",
      "percentage": 76,
      "state": "discharging"
    }
  ]
}
```

---

# 18. Architecture

I would ask the developer to keep the Rust project modular.

```text
src/
├── main.rs
│
├── config/
│   └── mod.rs
│
├── power/
│   ├── mod.rs
│   ├── monitor.rs
│   ├── battery.rs
│   └── events.rs
│
├── upower/
│   ├── mod.rs
│   └── client.rs
│
├── api/
│   ├── mod.rs
│   ├── routes.rs
│   ├── power.rs
│   └── events.rs
│
├── websocket/
│   └── mod.rs
│
├── notifications/
│   └── mod.rs
│
├── database/
│   ├── mod.rs
│   └── events.rs
│
├── cli/
│   └── mod.rs
│
└── logging/
    └── mod.rs
```

The important architectural principle is:

> **Linux/UPower integration should be separated from the API layer.**

That way, the API isn't directly responsible for reading Linux hardware information.

---

# 19. MVP

I would **not** ask the developer to build everything above initially.

The first milestone should be:

### Phase 1 — Core daemon

* Rust
* UPower/D-Bus
* Detect AC connection/disconnection
* Read battery percentage
* Read charging state
* Log events
* systemd service

### Phase 2 — API

```text
GET /api/v1/power
GET /api/v1/battery
GET /api/v1/events
```

### Phase 3 — Real-time events

```text
WebSocket /ws
```

### Phase 4 — Notifications

* AC disconnected
* AC connected
* Low battery
* Critical battery
* Fully charged

### Phase 5 — Persistence

* SQLite
* Power history
* Event querying

### Phase 6 — CLI + packaging

* CLI
* `.deb`
* systemd installation
* uninstall script
* configuration

---

## The actual product concept

I would describe the target to the developer as:

> **A lightweight Linux-native power monitoring daemon written in Rust. It runs continuously as a system service, communicates with Linux's UPower/D-Bus subsystem, monitors AC and battery state, records power events, generates desktop notifications, and exposes a localhost REST/WebSocket API for other applications to consume.**

That is a much better target than simply saying *"build an app that tells me when my charger is unplugged."*

It gives you a **reusable Linux power-monitoring service** that other applications can consume later.
