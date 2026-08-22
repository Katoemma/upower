# Astra (Flutter)

Mobile client for the Linux `power-monitor` daemon.

## Defaults

- Server URL: `https://astra.lipon.store`
- WebSocket: `wss://astra.lipon.store/api/v1/stream?token=…`
- Emulator preset: `http://10.0.2.2:8765` (Settings)

## Run

```bash
# Host machine: daemon (+ tunnel or reverse proxy to astra.lipon.store)
power-monitor daemon

# Phone
flutter run
```

Sign in with a seeded user (`power-monitor user add …`). After login the app registers the FCM token via `POST /api/v1/push/tokens`.

## Firebase

`Firebase.initializeApp` uses [lib/firebase_options.dart](lib/firebase_options.dart) (FlutterFire). Android `google-services.json` is already wired.
