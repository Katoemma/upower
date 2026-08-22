# Astra (Flutter)

Mobile client for the Linux `power-monitor` daemon.

## Defaults

- Server URL: `https://hostels-rolling-lol-films.trycloudflare.com`
- WebSocket: `wss://hostels-rolling-lol-films.trycloudflare.com/api/v1/stream?token=…`
- Emulator preset: `http://10.0.2.2:8765` (Settings)

## Run

```bash
# Host machine: daemon + tunnel
power-monitor daemon
../packaging/cloudflare-tunnel.sh   # or: cloudflared tunnel --url http://127.0.0.1:8765

# Phone
flutter run
```

Sign in with a seeded user (`power-monitor user add …`). After login the app registers the FCM token via `POST /api/v1/push/tokens`.

## Firebase

`Firebase.initializeApp` uses [lib/firebase_options.dart](lib/firebase_options.dart) (FlutterFire). Android `google-services.json` is already wired.
