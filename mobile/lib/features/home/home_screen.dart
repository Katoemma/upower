import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/providers.dart';
import '../../../theme/app_colors.dart';
import '../auth/auth_controller.dart';
import 'widgets/live_pill.dart';
import 'widgets/metric_tile.dart';
import 'widgets/server_identity.dart';
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
    final s = raw.replaceAll('_', ' ');
    if (s.isEmpty) return s;
    return s[0].toUpperCase() + s.substring(1);
  }

  String? _subtitle(PowerUiState s) {
    if (s.timeToFull != null && s.timeToFull! > 0) {
      final m = (s.timeToFull! / 60).round();
      return '~$m min to full';
    }
    if (s.timeRemaining != null && s.timeRemaining! > 0) {
      final m = (s.timeRemaining! / 60).round();
      return '~$m min left';
    }
    return null;
  }

  String _etaLabel(PowerUiState s) {
    if (s.acConnected && s.timeToFull != null && s.timeToFull! > 0) {
      return '${(s.timeToFull! / 60).round()} min';
    }
    if (!s.acConnected && s.timeRemaining != null && s.timeRemaining! > 0) {
      return '${(s.timeRemaining! / 60).round()} min';
    }
    return '—';
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final power = ref.watch(powerSnapshotProvider);
    final auth = ref.watch(authControllerProvider);

    return Scaffold(
      backgroundColor: Colors.transparent,
      extendBodyBehindAppBar: false,
      appBar: AppBar(
        backgroundColor: Colors.transparent,
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Power Monitor',
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.w800,
                    letterSpacing: -0.4,
                  ),
            ),
            Text(
              auth.email ?? 'Company home server',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: AppColors.mutedForeground,
                  ),
            ),
          ],
        ),
        actions: [
          IconButton(
            tooltip: 'Events',
            onPressed: () => context.push('/events'),
            icon: const Icon(Icons.history_rounded),
          ),
          IconButton(
            tooltip: 'Settings',
            onPressed: () => context.push('/settings'),
            icon: const Icon(Icons.settings_outlined),
          ),
        ],
      ),
      body: DecoratedBox(
        decoration: const BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [
              Color(0xFFFFF8F3),
              AppColors.background,
              Color(0xFFF7F7F7),
            ],
            stops: [0, 0.35, 1],
          ),
        ),
        child: RefreshIndicator(
          color: AppColors.primary,
          onRefresh: () => ref.read(powerSnapshotProvider.notifier).refresh(),
          child: ListView(
            physics: const AlwaysScrollableScrollPhysics(
              parent: BouncingScrollPhysics(),
            ),
            padding: const EdgeInsets.fromLTRB(20, 4, 20, 36),
            children: [
              Align(
                alignment: Alignment.centerRight,
                child: LivePill(connected: power.live),
              ),
              const SizedBox(height: 8),
              if (power.loading)
                const Padding(
                  padding: EdgeInsets.symmetric(vertical: 80),
                  child: Center(
                    child: CircularProgressIndicator(color: AppColors.primary),
                  ),
                )
              else ...[
                StatusHero(
                  acConnected: power.acConnected,
                  percentage: power.percentage,
                  stateLabel: _prettyState(power.state),
                  subtitle: _subtitle(power),
                ),
                if (power.error != null) ...[
                  const SizedBox(height: 12),
                  Text(
                    power.error!,
                    style: const TextStyle(color: AppColors.destructive),
                  ),
                ],
                const SizedBox(height: 22),
                const ServerIdentity(),
                const SizedBox(height: 16),
                Row(
                  children: [
                    Expanded(
                      child: MetricTile(
                        icon: Icons.favorite_outline_rounded,
                        label: 'Battery health',
                        value: power.health == null
                            ? '—'
                            : '${power.health!.round()}%',
                      ),
                    ),
                    const SizedBox(width: 10),
                    Expanded(
                      child: MetricTile(
                        icon: Icons.schedule_rounded,
                        label: power.acConnected ? 'Time to full' : 'Runtime',
                        value: _etaLabel(power),
                      ),
                    ),
                  ],
                ),
                if (power.lastEvent != null) ...[
                  const SizedBox(height: 10),
                  MetricTile(
                    icon: Icons.bolt_outlined,
                    label: 'Last event',
                    value: _prettyState(power.lastEvent!),
                  ),
                ],
                const SizedBox(height: 20),
                FilledButton.tonal(
                  onPressed: () => context.push('/events'),
                  style: FilledButton.styleFrom(
                    backgroundColor: AppColors.foreground,
                    foregroundColor: Colors.white,
                    minimumSize: const Size.fromHeight(50),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                  ),
                  child: const Text('Event history'),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
