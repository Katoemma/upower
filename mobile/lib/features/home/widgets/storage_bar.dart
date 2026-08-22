import 'package:flutter/material.dart';

import '../../../core/system_models.dart';
import '../../../theme/power_atmosphere.dart';

class StorageBar extends StatelessWidget {
  const StorageBar({
    super.key,
    required this.mount,
    required this.atmosphere,
  });

  final StorageMount mount;
  final PowerAtmosphere atmosphere;

  Color _barColor(double pct) {
    if (pct >= 90) return AppColors.batteryCritical;
    if (pct >= 75) return AppColors.batteryLow;
    return atmosphere.accent;
  }

  @override
  Widget build(BuildContext context) {
    final pct = mount.usagePercent.clamp(0.0, 100.0);
    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      padding: const EdgeInsets.fromLTRB(14, 12, 14, 12),
      decoration: BoxDecoration(
        color: AppColors.panel.withValues(alpha: 0.85),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: atmosphere.accent.withValues(alpha: 0.2)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  mount.mount,
                  style: const TextStyle(fontWeight: FontWeight.w700),
                ),
              ),
              Text(
                '${pct.round()}%',
                style: TextStyle(
                  fontWeight: FontWeight.w800,
                  color: _barColor(pct),
                ),
              ),
            ],
          ),
          const SizedBox(height: 4),
          Text(
            '${mount.filesystem} · ${formatBytes(mount.usedBytes)} / ${formatBytes(mount.totalBytes)}',
            style: const TextStyle(color: AppColors.textDim, fontSize: 12),
          ),
          const SizedBox(height: 10),
          ClipRRect(
            borderRadius: BorderRadius.circular(999),
            child: LinearProgressIndicator(
              value: pct / 100,
              minHeight: 8,
              backgroundColor: AppColors.stroke,
              color: _barColor(pct),
            ),
          ),
        ],
      ),
    );
  }
}
