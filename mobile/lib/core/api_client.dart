import 'package:dio/dio.dart';

import 'config.dart';

class ApiClient {
  ApiClient(this._config) {
    _dio = Dio(
      BaseOptions(
        connectTimeout: const Duration(seconds: 20),
        receiveTimeout: const Duration(seconds: 20),
        headers: {'Accept': 'application/json'},
      ),
    );
    _dio.interceptors.add(
      InterceptorsWrapper(
        onRequest: (options, handler) {
          options.baseUrl = _config.baseUri.toString();
          if (_token != null && _token!.isNotEmpty) {
            options.headers['Authorization'] = 'Bearer $_token';
          }
          handler.next(options);
        },
      ),
    );
  }

  final AppConfig _config;
  late final Dio _dio;
  String? _token;

  void setToken(String? token) => _token = token;

  Future<Map<String, dynamic>> login({
    required String email,
    required String password,
  }) async {
    final res = await _dio.post<Map<String, dynamic>>(
      '/api/v1/auth/login',
      data: {'email': email, 'password': password},
    );
    return res.data ?? {};
  }

  Future<Map<String, dynamic>> me() async {
    final res = await _dio.get<Map<String, dynamic>>('/api/v1/auth/me');
    return res.data ?? {};
  }

  Future<Map<String, dynamic>> power() async {
    final res = await _dio.get<Map<String, dynamic>>('/api/v1/power');
    return res.data ?? {};
  }

  Future<Map<String, dynamic>> system() async {
    final res = await _dio.get<Map<String, dynamic>>('/api/v1/system');
    return res.data ?? {};
  }

  Future<Map<String, dynamic>> events({int page = 1, int limit = 50}) async {
    final res = await _dio.get<Map<String, dynamic>>(
      '/api/v1/events',
      queryParameters: {'page': page, 'limit': limit},
    );
    return res.data ?? {};
  }

  Future<void> registerPushToken(String token) async {
    await _dio.post('/api/v1/push/tokens', data: {'token': token});
  }

  String describeError(Object error) {
    if (error is DioException) {
      final data = error.response?.data;
      if (data is Map && data['message'] != null) {
        return data['message'].toString();
      }
      if (data is String && data.isNotEmpty) return data;
      if (error.response?.statusCode == 401) {
        return 'Invalid email or password';
      }
      if (error.type == DioExceptionType.connectionError ||
          error.type == DioExceptionType.connectionTimeout) {
        return 'Cannot reach server. Check Server URL / tunnel.';
      }
      return error.message ?? 'Request failed';
    }
    return error.toString();
  }
}
