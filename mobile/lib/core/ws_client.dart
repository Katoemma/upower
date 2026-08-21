import 'dart:async';
import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

import 'config.dart';

typedef WsMessageHandler = void Function(Map<String, dynamic> message);

class WsClient {
  WsClient(this._config);

  final AppConfig _config;
  WebSocketChannel? _channel;
  StreamSubscription? _sub;
  Timer? _reconnect;
  String? _token;
  WsMessageHandler? onMessage;
  void Function(bool connected)? onConnectionChanged;
  bool _wanted = false;

  bool get isConnected => _channel != null;

  void connect(String? token) {
    _token = token;
    _wanted = true;
    _open();
  }

  void disconnect() {
    _wanted = false;
    _reconnect?.cancel();
    _sub?.cancel();
    _channel?.sink.close();
    _channel = null;
    onConnectionChanged?.call(false);
  }

  void _open() {
    _sub?.cancel();
    _channel?.sink.close();
    try {
      final uri = _config.wsUri(token: _token);
      final channel = WebSocketChannel.connect(uri);
      _channel = channel;
      onConnectionChanged?.call(true);
      _sub = channel.stream.listen(
        (raw) {
          try {
            final map = jsonDecode(raw as String) as Map<String, dynamic>;
            onMessage?.call(map);
          } catch (_) {}
        },
        onError: (_) => _scheduleReconnect(),
        onDone: _scheduleReconnect,
      );
    } catch (_) {
      _scheduleReconnect();
    }
  }

  void _scheduleReconnect() {
    onConnectionChanged?.call(false);
    _channel = null;
    if (!_wanted) return;
    _reconnect?.cancel();
    _reconnect = Timer(const Duration(seconds: 3), _open);
  }
}
