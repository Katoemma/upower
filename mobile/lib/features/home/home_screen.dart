import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/system_models.dart';
import '../../../theme/power_atmosphere.dart';
import '../auth/auth_controller.dart';
import '../system/system_controller.dart';
import 'power_controller.dart';
import 'widgets/live_pill.dart';
import 'widgets/metric_tile.dart';
import 'widgets/process_tile.dart';
import 'widgets/radial_gauge.dart';
import 'widgets/server_identity.dart';
import 'widgets/status_hero.dart';
import 'widgets/storage_bar.dart';

class HomeScreen extends ConsumerWidget {
  const HomeScreen({super.key});

  String _prettyState(String raw) {
    final s = raw.replaceAll('_', ' ');
    if (s.isEmpty) return s;
    return s[0].toUpperCase() + s.substring(1);
  }

  String _formatDuration(int seconds) {
    if (seconds <= 0) return '—';
    final totalMin = (seconds / 60).round();
    if (totalMin < 60) return '$totalMin min';
    final h = totalMin ~/ 60;
    final m = totalMin % 60;
    if (m == 0) return '${h}h';
    return '${h}h ${m}m';
  }

  String? _subtitle(PowerUiState s) {
    if (s.timeToFull != null && s.timeToFull! > 0) {
      return '~${_formatDuration(s.timeToFull!)} to full';
    }
    if (s.timeRemaining != null && s.timeRemaining! > 0) {
      return '~${_formatDuration(s.timeRemaining!)} left';
    }
    return null;
  }

  String _etaLabel(PowerUiState s) {
    if (s.acConnected && s.timeToFull != null && s.timeToFull! > 0) {
      return _formatDuration(s.timeToFull!);
    }
    if (!s.acConnected && s.timeRemaining != null && s.timeRemaining! > 0) {
      return _formatDuration(s.timeRemaining!);
    }
    return '—';
  }

  Future<void> _refresh(WidgetRef ref) async {
    await Future.wait([
      ref.read(powerSnapshotProvider.notifier).refresh(),
      ref.read(systemSnapshotProvider.notifier).refresh(),
    ]);
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final power = ref.watch(powerSnapshotProvider);
    final system = ref.watch(systemSnapshotProvider);
    final auth = ref.watch(authControllerProvider);
    final atmo = PowerAtmosphere.fromPower(
      acConnected: power.acConnected,
      percentage: power.percentage,
    );

    final maxProcMem = system.processes.isEmpty
        ? 0
        : system.processes.map((p) => p.memoryBytes).reduce((a, b) => a > b ? a : b);

    return AnnotatedRegion<SystemUiOverlayStyle>(
      value: SystemUiOverlayStyle.light.copyWith(
        statusBarColor: Colors.transparent,
        systemNavigationBarColor: AppColors.voidBlack,
      ),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 700),
        curve: Curves.easeOutCubic,
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: atmo.gradient,
          ),
        ),
        child: Scaffold(
          backgroundColor: Colors.transparent,
          appBar: AppBar(
            title: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Upower',
                  style: Theme.of(context).textTheme.titleLarge?.copyWith(
                        fontWeight: FontWeight.w800,
                        letterSpacing: -0.3,
                        color: AppColors.text,
                      ),
                ),
                Text(
                  auth.email ?? 'ThinkPad · Ubuntu',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: AppColors.textDim,
                      ),
                ),
              ],
            ),
            actions: [
              IconButton(
                tooltip: 'Events',
                onPressed: () => context.push('/events'),
                icon: Icon(Icons.history_rounded, color: atmo.accentSoft),
              ),
              IconButton(
                tooltip: 'Settings',
                onPressed: () => context.push('/settings'),
                icon: Icon(Icons.tune_rounded, color: atmo.accentSoft),
              ),
            ],
          ),
          body: RefreshIndicator(
            color: atmo.accent,
            backgroundColor: AppColors.panel,
            onRefresh: () => _refresh(ref),
            child: ListView(
              physics: const AlwaysScrollableScrollPhysics(
                parent: BouncingScrollPhysics(),
              ),
              padding: const EdgeInsets.fromLTRB(20, 4, 20, 36),
              children: [
                Align(
                  alignment: Alignment.centerRight,
                  child: LivePill(connected: power.live, atmosphere: atmo),
                ),
                const SizedBox(height: 4),
                if (power.loading && system.loading)
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 80),
                    child: Center(
                      child: CircularProgressIndicator(color: atmo.accent),
                    ),
                  )
                else ...[
                  StatusHero(
                    atmosphere: atmo,
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
                  const SizedBox(height: 10),
                  ServerIdentity(atmosphere: atmo),
                  const SizedBox(height: 14),
                  Text(
                    'SYSTEM TELEMETRY',
                    style: Theme.of(context).textTheme.labelSmall?.copyWith(
                          color: atmo.accent,
                          fontWeight: FontWeight.w800,
                          letterSpacing: 1.4,
                        ),
                  ),
                  const SizedBox(height: 14),
                  LayoutBuilder(
                    builder: (context, constraints) {
                      final gaugeSize =
                          (constraints.maxWidth / 3).clamp(72.0, 108.0);
                      return Row(
                        children: [
                          Expanded(
                            child: RadialGauge(
                              value: system.cpu.usagePercent,
                              label: 'CPU',
                              accent: AppColors.batteryHigh,
                              subtitle: '${system.cpu.cores} cores',
                              size: gaugeSize,
                            ),
                          ),
                          Expanded(
                            child: RadialGauge(
                              value: system.memory.usagePercent,
                              label: 'RAM',
                              accent: atmo.accent,
                              subtitle: formatBytes(system.memory.usedBytes),
                              size: gaugeSize,
                            ),
                          ),
                          Expanded(
                            child: RadialGauge(
                              value: power.percentage ?? 0,
                              label: 'Battery',
                              accent: atmo.accentSoft,
                              size: gaugeSize,
                            ),
                          ),
                        ],
                      );
                    },
                  ),
                  const SizedBox(height: 18),
                  Row(
                    children: [
                      Expanded(
                        child: MetricTile(
                          atmosphere: atmo,
                          icon: Icons.memory_rounded,
                          label: 'Available RAM',
                          value: formatBytes(system.memory.availableBytes),
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: MetricTile(
                          atmosphere: atmo,
                          icon: Icons.swap_horiz_rounded,
                          label: 'Swap used',
                          value: formatBytes(system.memory.swapUsedBytes),
                        ),
                      ),
                    ],
                  ),
                  if (system.storage.isNotEmpty) ...[
                    const SizedBox(height: 22),
                    Text(
                      'STORAGE',
                      style: Theme.of(context).textTheme.labelSmall?.copyWith(
                            color: atmo.accentSoft,
                            fontWeight: FontWeight.w800,
                            letterSpacing: 1.2,
                          ),
                    ),
                    const SizedBox(height: 10),
                    ...system.storage.map(
                      (m) => StorageBar(mount: m, atmosphere: atmo),
                    ),
                  ],
                  if (system.processes.isNotEmpty) ...[
                    const SizedBox(height: 16),
                    Text(
                      'TOP PROCESSES',
                      style: Theme.of(context).textTheme.labelSmall?.copyWith(
                            color: atmo.accentSoft,
                            fontWeight: FontWeight.w800,
                            letterSpacing: 1.2,
                          ),
                    ),
                    const SizedBox(height: 8),
                    Container(
                      padding: const EdgeInsets.fromLTRB(14, 10, 14, 10),
                      decoration: BoxDecoration(
                        color: AppColors.panel.withValues(alpha: 0.85),
                        borderRadius: BorderRadius.circular(14),
                        border: Border.all(
                          color: atmo.accent.withValues(alpha: 0.2),
                        ),
                      ),
                      child: Column(
                        children: system.processes
                            .take(8)
                            .map(
                              (p) => ProcessTile(
                                process: p,
                                atmosphere: atmo,
                                maxMemory: maxProcMem,
                              ),
                            )
                            .toList(),
                      ),
                    ),
                  ],
                  const SizedBox(height: 18),
                  Row(
                    children: [
                      Expanded(
                        child: MetricTile(
                          atmosphere: atmo,
                          icon: Icons.favorite_outline_rounded,
                          label: 'Cell health',
                          value: power.health == null
                              ? '—'
                              : '${power.health!.round()}%',
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: MetricTile(
                          atmosphere: atmo,
                          icon: Icons.schedule_rounded,
                          label: power.acConnected ? 'To full' : 'Runtime',
                          value: _etaLabel(power),
                        ),
                      ),
                    ],
                  ),
                  if (power.lastEvent != null) ...[
                    const SizedBox(height: 10),
                    MetricTile(
                      atmosphere: atmo,
                      icon: Icons.bolt_outlined,
                      label: 'Last signal',
                      value: _prettyState(power.lastEvent!),
                    ),
                  ],
                  const SizedBox(height: 22),
                  FilledButton(
                    onPressed: () => context.push('/events'),
                    style: FilledButton.styleFrom(
                      backgroundColor: atmo.accent,
                      foregroundColor: AppColors.voidBlack,
                      minimumSize: const Size.fromHeight(52),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(14),
                      ),
                    ),
                    child: const Text('Open event timeline'),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }
}
