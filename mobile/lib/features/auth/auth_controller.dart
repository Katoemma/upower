import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/providers.dart';

class AuthState {
  const AuthState({
    this.token,
    this.email,
    this.loading = false,
    this.bootstrapped = false,
    this.error,
  });

  final String? token;
  final String? email;
  final bool loading;
  final bool bootstrapped;
  final String? error;

  bool get isAuthenticated => token != null && token!.isNotEmpty;

  AuthState copyWith({
    String? token,
    String? email,
    bool? loading,
    bool? bootstrapped,
    String? error,
    bool clearError = false,
    bool clearSession = false,
  }) {
    return AuthState(
      token: clearSession ? null : (token ?? this.token),
      email: clearSession ? null : (email ?? this.email),
      loading: loading ?? this.loading,
      bootstrapped: bootstrapped ?? this.bootstrapped,
      error: clearError ? null : (error ?? this.error),
    );
  }
}

class AuthController extends StateNotifier<AuthState> {
  AuthController(this._ref) : super(const AuthState());

  final Ref _ref;

  Future<void> bootstrap() async {
    final store = _ref.read(secureStoreProvider);
    final api = _ref.read(apiClientProvider);
    final config = _ref.read(appConfigProvider);
    await config.load();

    final token = await store.readToken();
    final email = await store.readEmail();
    if (token != null && token.isNotEmpty) {
      api.setToken(token);
      try {
        await api.me();
        state = AuthState(
          token: token,
          email: email,
          bootstrapped: true,
        );
        _connectLive(token);
        // Refresh FCM registration in background.
        unawaited(_registerFcm());
        return;
      } catch (_) {
        await store.clear();
        api.setToken(null);
      }
    }
    state = const AuthState(bootstrapped: true);
  }

  Future<bool> login(String email, String password) async {
    state = state.copyWith(loading: true, clearError: true);
    final api = _ref.read(apiClientProvider);
    final store = _ref.read(secureStoreProvider);
    try {
      final data = await api.login(email: email.trim(), password: password);
      final token = data['token'] as String?;
      final userEmail =
          (data['user'] as Map?)?['email'] as String? ?? email.trim();
      if (token == null || token.isEmpty) {
        state = state.copyWith(loading: false, error: 'Login failed');
        return false;
      }
      await store.saveSession(token: token, email: userEmail);
      api.setToken(token);
      state = AuthState(
        token: token,
        email: userEmail,
        bootstrapped: true,
        loading: false,
      );
      _connectLive(token);
      await _registerFcm();
      return true;
    } catch (e) {
      state = state.copyWith(
        loading: false,
        error: api.describeError(e),
      );
      return false;
    }
  }

  Future<void> logout() async {
    _ref.read(wsClientProvider).disconnect();
    await _ref.read(secureStoreProvider).clear();
    _ref.read(apiClientProvider).setToken(null);
    state = const AuthState(bootstrapped: true);
  }

  void _connectLive(String token) {
    final ws = _ref.read(wsClientProvider);
    ws.connect(token);
  }

  Future<void> _registerFcm() async {
    try {
      final fcm = _ref.read(fcmServiceProvider);
      await fcm.init();
      await fcm.registerWithServer();
    } catch (e) {
      debugPrint('FCM register skipped: $e');
    }
  }
}

final authControllerProvider =
    StateNotifierProvider<AuthController, AuthState>((ref) {
  return AuthController(ref);
});
