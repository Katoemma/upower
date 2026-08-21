import 'package:flutter/material.dart';

import '../../../theme/power_atmosphere.dart';

class MetricTile extends StatelessWidget {
  const MetricTile({
    super.key,
    required this.atmosphere,
    required this.label,
    required this.value,
    this.icon,
  });

  final PowerAtmosphere atmosphere;
  final String label;
  final String value;
  final IconData? icon;

  @override
  Widget build(BuildContext context) {
    final accent = atmosphere.accent;
    return AnimatedContainer(
      duration: const Duration(milliseconds: 450),
      padding: const EdgeInsets.fromLTRB(14, 14, 14, 14),
      decoration: BoxDecoration(
        color: AppColors.panel.withValues(alpha: 0.85),
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: accent.withValues(alpha: 0.22)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              if (icon != null) ...[
                Icon(icon, size: 15, color: accent.withValues(alpha: 0.85)),
                const SizedBox(width: 6),
              ],
              Expanded(
                child: Text(
                  label,
                  style: TextStyle(
                    color: AppColors.textDim,
                    fontSize: 12,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Text(
            value,
            style: TextStyle(
              fontWeight: FontWeight.w700,
              fontSize: 16,
              letterSpacing: -0.2,
              color: AppColors.text,
            ),
          ),
        ],
      ),
    );
  }
}
