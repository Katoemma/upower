import 'package:shared_preferences/shared_preferences.dart';

/// App configuration / Server URL.
class AppConfig {
  static const defaultServerUrl =
      'https://hostels-rolling-lol-films.trycloudflare.com';
  static const emulatorServerUrl = 'http://10.0.2.2:8765';
  static const _key = 'server_url';

  String _serverUrl = defaultServerUrl;

  String get serverUrl => _serverUrl;

  Uri get baseUri => Uri.parse(_serverUrl.replaceAll(RegExp(r'/+$'), ''));

  Uri wsUri({String? token}) {
    final base = baseUri;
    final scheme = base.scheme == 'https' ? 'wss' : 'ws';
    return base.replace(
      scheme: scheme,
      path: '/ws',
      queryParameters: token == null || token.isEmpty ? null : {'token': token},
    );
  }

  /// Live telemetry: memory, CPU, storage, processes + power events.
  Uri streamUri({String? token}) {
    final base = baseUri;
    final scheme = base.scheme == 'https' ? 'wss' : 'ws';
    return base.replace(
      scheme: scheme,
      path: '/api/v1/stream',
      queryParameters: token == null || token.isEmpty ? null : {'token': token},
    );
  }

  Future<void> load() async {
    final prefs = await SharedPreferences.getInstance();
    _serverUrl = prefs.getString(_key) ?? defaultServerUrl;
  }

  Future<void> setServerUrl(String url) async {
    final trimmed = url.trim().replaceAll(RegExp(r'/+$'), '');
    _serverUrl = trimmed.isEmpty ? defaultServerUrl : trimmed;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_key, _serverUrl);
  }
}
