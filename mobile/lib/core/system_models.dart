class MemoryInfo {
  const MemoryInfo({
    this.totalBytes = 0,
    this.usedBytes = 0,
    this.availableBytes = 0,
    this.usagePercent = 0,
    this.swapTotalBytes = 0,
    this.swapUsedBytes = 0,
  });

  final int totalBytes;
  final int usedBytes;
  final int availableBytes;
  final double usagePercent;
  final int swapTotalBytes;
  final int swapUsedBytes;

  factory MemoryInfo.fromJson(Map<String, dynamic>? json) {
    if (json == null) return const MemoryInfo();
    final swap = json['swap'] as Map?;
    return MemoryInfo(
      totalBytes: (json['total_bytes'] as num?)?.toInt() ?? 0,
      usedBytes: (json['used_bytes'] as num?)?.toInt() ?? 0,
      availableBytes: (json['available_bytes'] as num?)?.toInt() ?? 0,
      usagePercent: (json['usage_percent'] as num?)?.toDouble() ?? 0,
      swapTotalBytes: (swap?['total_bytes'] as num?)?.toInt() ?? 0,
      swapUsedBytes: (swap?['used_bytes'] as num?)?.toInt() ?? 0,
    );
  }
}

class CpuInfo {
  const CpuInfo({
    this.usagePercent = 0,
    this.cores = 0,
    this.perCore = const [],
  });

  final double usagePercent;
  final int cores;
  final List<double> perCore;

  factory CpuInfo.fromJson(Map<String, dynamic>? json) {
    if (json == null) return const CpuInfo();
    return CpuInfo(
      usagePercent: (json['usage_percent'] as num?)?.toDouble() ?? 0,
      cores: (json['cores'] as num?)?.toInt() ?? 0,
      perCore: (json['per_core'] as List?)
              ?.map((e) => (e as num).toDouble())
              .toList() ??
          const [],
    );
  }
}

class StorageMount {
  const StorageMount({
    required this.mount,
    required this.device,
    required this.filesystem,
    required this.totalBytes,
    required this.usedBytes,
    required this.availableBytes,
    required this.usagePercent,
  });

  final String mount;
  final String device;
  final String filesystem;
  final int totalBytes;
  final int usedBytes;
  final int availableBytes;
  final double usagePercent;

  factory StorageMount.fromJson(Map<String, dynamic> json) {
    return StorageMount(
      mount: json['mount']?.toString() ?? '',
      device: json['device']?.toString() ?? '',
      filesystem: json['filesystem']?.toString() ?? '',
      totalBytes: (json['total_bytes'] as num?)?.toInt() ?? 0,
      usedBytes: (json['used_bytes'] as num?)?.toInt() ?? 0,
      availableBytes: (json['available_bytes'] as num?)?.toInt() ?? 0,
      usagePercent: (json['usage_percent'] as num?)?.toDouble() ?? 0,
    );
  }
}

class ProcessInfo {
  const ProcessInfo({
    required this.pid,
    required this.name,
    required this.cpuPercent,
    required this.memoryBytes,
  });

  final int pid;
  final String name;
  final double cpuPercent;
  final int memoryBytes;

  factory ProcessInfo.fromJson(Map<String, dynamic> json) {
    return ProcessInfo(
      pid: (json['pid'] as num?)?.toInt() ?? 0,
      name: json['name']?.toString() ?? 'unknown',
      cpuPercent: (json['cpu_percent'] as num?)?.toDouble() ?? 0,
      memoryBytes: (json['memory_bytes'] as num?)?.toInt() ?? 0,
    );
  }
}

class SystemSnapshot {
  const SystemSnapshot({
    this.memory = const MemoryInfo(),
    this.cpu = const CpuInfo(),
    this.storage = const [],
    this.processes = const [],
    this.loading = true,
  });

  final MemoryInfo memory;
  final CpuInfo cpu;
  final List<StorageMount> storage;
  final List<ProcessInfo> processes;
  final bool loading;

  SystemSnapshot copyWith({
    MemoryInfo? memory,
    CpuInfo? cpu,
    List<StorageMount>? storage,
    List<ProcessInfo>? processes,
    bool? loading,
  }) {
    return SystemSnapshot(
      memory: memory ?? this.memory,
      cpu: cpu ?? this.cpu,
      storage: storage ?? this.storage,
      processes: processes ?? this.processes,
      loading: loading ?? this.loading,
    );
  }

  factory SystemSnapshot.fromJson(Map<String, dynamic> json) {
    final mounts = (json['storage'] as List? ?? json['mounts'] as List?)
            ?.whereType<Map>()
            .map((e) => StorageMount.fromJson(Map<String, dynamic>.from(e)))
            .toList() ??
        const <StorageMount>[];
    final procs = (json['processes'] as List?)
            ?.whereType<Map>()
            .map((e) => ProcessInfo.fromJson(Map<String, dynamic>.from(e)))
            .toList() ??
        const <ProcessInfo>[];

    return SystemSnapshot(
      memory: MemoryInfo.fromJson(json['memory'] as Map<String, dynamic>?),
      cpu: CpuInfo.fromJson(json['cpu'] as Map<String, dynamic>?),
      storage: mounts,
      processes: procs,
      loading: false,
    );
  }
}

String formatBytes(int bytes) {
  if (bytes <= 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  var value = bytes.toDouble();
  var i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return '${value.toStringAsFixed(value >= 10 || i == 0 ? 0 : 1)} ${units[i]}';
}
