import 'package:flutter/material.dart';

import '../../../theme/app_colors.dart';

class StatusHero extends StatelessWidget {
  const StatusHero({
    super.key,
    required this.acConnected,
    required this.percentage,
    required this.stateLabel,
    this.subtitle,
  });

  final bool acConnected;
  final double? percentage;
  final String stateLabel;
  final String? subtitle;

  @override
  Widget build(BuildContext context) {
    final pct = percentage?.round();
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
          decoration: BoxDecoration(
            color: acConnected
                ? AppColors.primary.withValues(alpha: 0.12)
                : AppColors.muted,
            borderRadius: BorderRadius.circular(999),
          ),
          child: Text(
            acConnected ? 'On AC' : 'On Battery',
            style: TextStyle(
              color: acConnected ? AppColors.primary : AppColors.foreground,
              fontWeight: FontWeight.w600,
              fontSize: 13,
            ),
          ),
        ),
        const SizedBox(height: 16),
        Text(
          pct == null ? '—' : '$pct%',
          style: Theme.of(context).textTheme.displayLarge?.copyWith(
                fontWeight: FontWeight.w700,
                height: 1,
                letterSpacing: -1.5,
              ),
        ),
        const SizedBox(height: 8),
        Text(
          stateLabel,
          style: Theme.of(context).textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.w600,
              ),
        ),
        if (subtitle != null) ...[
          const SizedBox(height: 4),
          Text(
            subtitle!,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  color: AppColors.mutedForeground,
                ),
          ),
        ],
      ],
    );
  }
}
