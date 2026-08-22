import 'dart:async';
import 'dart:convert';

import 'package:web_socket_channel/web_socket_channel.dart';

import 'config.dart';

class WsClient {
  WsClient(this._config);

  final AppConfig _config;
  WebSocketChannel? _channel;
  StreamSubscription? _sub;
  Timer? _reconnect;
  String? _token;
  bool _wanted = false;

  final _messages = StreamController<Map<String, dynamic>>.broadcast();
  final _connection = StreamController<bool>.broadcast();

  Stream<Map<String, dynamic>> get messages => _messages.stream;
  Stream<bool> get connection => _connection.stream;

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
    _connection.add(false);
  }

  void _open() {
    _sub?.cancel();
    _channel?.sink.close();
    try {
      final uri = _config.streamUri(token: _token);
      final channel = WebSocketChannel.connect(uri);
      _channel = channel;
      _connection.add(true);
      _sub = channel.stream.listen(
        (raw) {
          try {
            final map = jsonDecode(raw as String) as Map<String, dynamic>;
            _messages.add(map);
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
    _connection.add(false);
    _channel = null;
    if (!_wanted) return;
    _reconnect?.cancel();
    _reconnect = Timer(const Duration(seconds: 3), _open);
  }
}
