import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/providers.dart';
import '../../core/system_models.dart';

final systemSnapshotProvider =
    StateNotifierProvider<SystemController, SystemSnapshot>((ref) {
  return SystemController(ref);
});

class SystemController extends StateNotifier<SystemSnapshot> {
  SystemController(this._ref) : super(const SystemSnapshot()) {
    _bindStream();
    refresh();
  }

  final Ref _ref;
  StreamSubscription<Map<String, dynamic>>? _sub;

  void _bindStream() {
    final ws = _ref.read(wsClientProvider);
    _sub?.cancel();
    _sub = ws.messages.listen(_onMessage);
  }

  void _onMessage(Map<String, dynamic> msg) {
    final type = msg['type'] as String?;
    switch (type) {
      case 'system_snapshot':
        state = SystemSnapshot.fromJson(msg);
      case 'memory':
        state = state.copyWith(
          memory: MemoryInfo(
            totalBytes: state.memory.totalBytes,
            usedBytes: (msg['used_bytes'] as num?)?.toInt() ?? state.memory.usedBytes,
            availableBytes:
                (msg['available_bytes'] as num?)?.toInt() ?? state.memory.availableBytes,
            usagePercent:
                (msg['used_percent'] as num?)?.toDouble() ?? state.memory.usagePercent,
            swapTotalBytes: state.memory.swapTotalBytes,
            swapUsedBytes: state.memory.swapUsedBytes,
          ),
          loading: false,
        );
      case 'cpu':
        state = state.copyWith(
          cpu: CpuInfo(
            usagePercent:
                (msg['usage_percent'] as num?)?.toDouble() ?? state.cpu.usagePercent,
            cores: (msg['cores'] as num?)?.toInt() ?? state.cpu.cores,
            perCore: (msg['per_core'] as List?)
                    ?.map((e) => (e as num).toDouble())
                    .toList() ??
                state.cpu.perCore,
          ),
          loading: false,
        );
      case 'storage':
        final mounts = (msg['mounts'] as List?)
                ?.whereType<Map>()
                .map((e) => StorageMount.fromJson(Map<String, dynamic>.from(e)))
                .toList() ??
            state.storage;
        state = state.copyWith(storage: mounts, loading: false);
      case 'processes':
        final procs = (msg['processes'] as List?)
                ?.whereType<Map>()
                .map((e) => ProcessInfo.fromJson(Map<String, dynamic>.from(e)))
                .toList() ??
            state.processes;
        state = state.copyWith(processes: procs, loading: false);
    }
  }

  Future<void> refresh() async {
    final api = _ref.read(apiClientProvider);
    try {
      final data = await api.system();
      state = SystemSnapshot.fromJson(data);
    } catch (_) {
      state = state.copyWith(loading: false);
    }
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }
}
