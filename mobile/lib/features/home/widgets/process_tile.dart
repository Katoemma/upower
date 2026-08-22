import 'package:flutter/material.dart';

import '../../../core/system_models.dart';
import '../../../theme/power_atmosphere.dart';

class ProcessTile extends StatelessWidget {
  const ProcessTile({
    super.key,
    required this.process,
    required this.atmosphere,
    required this.maxMemory,
  });

  final ProcessInfo process;
  final PowerAtmosphere atmosphere;
  final int maxMemory;

  @override
  Widget build(BuildContext context) {
    final frac = maxMemory <= 0
        ? 0.0
        : (process.memoryBytes / maxMemory).clamp(0.0, 1.0);

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  process.name,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(fontWeight: FontWeight.w600),
                ),
                const SizedBox(height: 2),
                Text(
                  'PID ${process.pid} · CPU ${process.cpuPercent.toStringAsFixed(1)}%',
                  style: const TextStyle(color: AppColors.textDim, fontSize: 11),
                ),
              ],
            ),
          ),
          const SizedBox(width: 12),
          SizedBox(
            width: 72,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                Text(
                  formatBytes(process.memoryBytes),
                  style: const TextStyle(
                    fontWeight: FontWeight.w700,
                    fontSize: 12,
                  ),
                ),
                const SizedBox(height: 4),
                ClipRRect(
                  borderRadius: BorderRadius.circular(999),
                  child: LinearProgressIndicator(
                    value: frac,
                    minHeight: 4,
                    backgroundColor: AppColors.stroke,
                    color: atmosphere.accentSoft,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
