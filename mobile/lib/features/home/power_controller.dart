import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/providers.dart';

final powerSnapshotProvider =
    StateNotifierProvider<PowerController, PowerUiState>((ref) {
  return PowerController(ref);
});

class PowerUiState {
  const PowerUiState({
    this.acConnected = false,
    this.percentage,
    this.state = 'unknown',
    this.health,
    this.timeRemaining,
    this.timeToFull,
    this.live = false,
    this.lastEvent,
    this.error,
    this.loading = true,
  });

  final bool acConnected;
  final double? percentage;
  final String state;
  final double? health;
  final int? timeRemaining;
  final int? timeToFull;
  final bool live;
  final String? lastEvent;
  final String? error;
  final bool loading;

  PowerUiState copyWith({
    bool? acConnected,
    double? percentage,
    String? state,
    double? health,
    int? timeRemaining,
    int? timeToFull,
    bool? live,
    String? lastEvent,
    String? error,
    bool? loading,
    bool clearError = false,
  }) {
    return PowerUiState(
      acConnected: acConnected ?? this.acConnected,
      percentage: percentage ?? this.percentage,
      state: state ?? this.state,
      health: health ?? this.health,
      timeRemaining: timeRemaining ?? this.timeRemaining,
      timeToFull: timeToFull ?? this.timeToFull,
      live: live ?? this.live,
      lastEvent: lastEvent ?? this.lastEvent,
      error: clearError ? null : (error ?? this.error),
      loading: loading ?? this.loading,
    );
  }
}

class PowerController extends StateNotifier<PowerUiState> {
  PowerController(this._ref) : super(const PowerUiState()) {
    _bindStream();
    refresh();
  }

  final Ref _ref;
  StreamSubscription<Map<String, dynamic>>? _msgSub;
  StreamSubscription<bool>? _connSub;

  void _bindStream() {
    final ws = _ref.read(wsClientProvider);
    _msgSub?.cancel();
    _connSub?.cancel();
    _connSub = ws.connection.listen((c) {
      state = state.copyWith(live: c);
    });
    _msgSub = ws.messages.listen(_onMessage);
  }

  void _onMessage(Map<String, dynamic> msg) {
    final type = msg['type'] as String?;
    if (type == 'system_snapshot') {
      final power = msg['power'] as Map?;
      if (power != null) {
        _applyPowerMap(power);
      }
      return;
    }
    if (type == 'snapshot') {
      _applyPowerMap(msg);
      return;
    }
    if (type == 'power_event') {
      final ac = msg['ac_connected'] as bool? ?? state.acConnected;
      final pct = (msg['battery_percentage'] as num?)?.toDouble();
      final st = msg['event']?.toString() ?? state.state;
      state = state.copyWith(
        acConnected: ac,
        percentage: pct ?? state.percentage,
        state: st,
        lastEvent: st,
        loading: false,
      );
    }
  }

  void _applyPowerMap(Map<dynamic, dynamic> map) {
    state = state.copyWith(
      acConnected: map['ac_connected'] as bool? ?? state.acConnected,
      percentage: (map['battery_percentage'] as num?)?.toDouble() ?? state.percentage,
      state: map['state']?.toString() ?? state.state,
      loading: false,
      clearError: true,
    );
  }

  Future<void> refresh() async {
    final api = _ref.read(apiClientProvider);
    try {
      final data = await api.power();
      state = state.copyWith(
        acConnected: data['ac_connected'] as bool? ?? false,
        percentage: (data['battery_percentage'] as num?)?.toDouble(),
        state: data['state']?.toString() ?? 'unknown',
        health: (data['battery_health'] as num?)?.toDouble(),
        timeRemaining: (data['time_remaining_seconds'] as num?)?.toInt(),
        timeToFull: (data['time_to_full_seconds'] as num?)?.toInt(),
        loading: false,
        clearError: true,
      );
    } catch (e) {
      state = state.copyWith(
        loading: false,
        error: api.describeError(e),
      );
    }
  }

  @override
  void dispose() {
    _msgSub?.cancel();
    _connSub?.cancel();
    super.dispose();
  }
}
