import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/providers.dart';
import '../../../theme/app_colors.dart';
import '../auth/auth_controller.dart';
import 'widgets/live_pill.dart';
import 'widgets/metric_row.dart';
import 'widgets/status_hero.dart';

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
    _bindWs();
    refresh();
  }

  final Ref _ref;

  void _bindWs() {
    final ws = _ref.read(wsClientProvider);
    ws.onConnectionChanged = (c) {
      state = state.copyWith(live: c);
    };
    ws.onMessage = (msg) {
      final type = msg['type'] as String?;
      if (type == 'snapshot' || type == 'power_event') {
        final ac = msg['ac_connected'] as bool? ?? state.acConnected;
        final pct = (msg['battery_percentage'] as num?)?.toDouble();
        final st = msg['state']?.toString() ??
            msg['event']?.toString() ??
            state.state;
        state = state.copyWith(
          acConnected: ac,
          percentage: pct ?? state.percentage,
          state: st,
          lastEvent: type == 'power_event'
              ? (msg['event']?.toString() ?? state.lastEvent)
              : state.lastEvent,
          loading: false,
        );
      }
    };
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
}

class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  String _prettyState(String raw) {
    return raw.replaceAll('_', ' ');
  }

  String? _subtitle(PowerUiState s) {
    if (s.timeToFull != null && s.timeToFull! > 0) {
      final m = (s.timeToFull! / 60).round();
      return 'About $m min to full';
    }
    if (s.timeRemaining != null && s.timeRemaining! > 0) {
      final m = (s.timeRemaining! / 60).round();
      return 'About $m min remaining';
    }
    return null;
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final power = ref.watch(powerSnapshotProvider);
    final auth = ref.watch(authControllerProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Power Monitor'),
        actions: [
          IconButton(
            tooltip: 'Events',
            onPressed: () => context.push('/events'),
            icon: const Icon(Icons.history),
          ),
          IconButton(
            tooltip: 'Settings',
            onPressed: () => context.push('/settings'),
            icon: const Icon(Icons.settings_outlined),
          ),
        ],
      ),
      body: RefreshIndicator(
        color: AppColors.primary,
        onRefresh: () => ref.read(powerSnapshotProvider.notifier).refresh(),
        child: ListView(
          padding: const EdgeInsets.fromLTRB(24, 8, 24, 32),
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    auth.email ?? '',
                    style: const TextStyle(color: AppColors.mutedForeground),
                  ),
                ),
                LivePill(connected: power.live),
              ],
            ),
            const SizedBox(height: 28),
            if (power.loading)
              const Padding(
                padding: EdgeInsets.symmetric(vertical: 48),
                child: Center(child: CircularProgressIndicator()),
              )
            else ...[
              StatusHero(
                acConnected: power.acConnected,
                percentage: power.percentage,
                stateLabel: _prettyState(power.state),
                subtitle: _subtitle(power),
              ),
              if (power.error != null) ...[
                const SizedBox(height: 16),
                Text(power.error!, style: const TextStyle(color: AppColors.destructive)),
              ],
              const SizedBox(height: 28),
              const Divider(),
              if (power.health != null)
                MetricRow(
                  label: 'Battery health',
                  value: '${power.health!.round()}%',
                ),
              if (power.lastEvent != null)
                MetricRow(
                  label: 'Last event',
                  value: _prettyState(power.lastEvent!),
                ),
              const SizedBox(height: 16),
              OutlinedButton(
                onPressed: () => context.push('/events'),
                style: OutlinedButton.styleFrom(
                  minimumSize: const Size.fromHeight(48),
                  side: const BorderSide(color: AppColors.border),
                  foregroundColor: AppColors.foreground,
                  shape: RoundedRectangleBorder(
                    borderRadius: BorderRadius.circular(8),
                  ),
                ),
                child: const Text('View event history'),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
