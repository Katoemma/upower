import 'package:flutter/material.dart';

import '../../../theme/power_atmosphere.dart';

class ServerIdentity extends StatelessWidget {
  const ServerIdentity({super.key, required this.atmosphere});

  final PowerAtmosphere atmosphere;

  @override
  Widget build(BuildContext context) {
    final accent = atmosphere.accent;
    return AnimatedContainer(
      duration: const Duration(milliseconds: 600),
      width: double.infinity,
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 16),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(16),
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            AppColors.panelElevated,
            Color.lerp(AppColors.panel, accent, 0.12)!,
          ],
        ),
        border: Border.all(color: accent.withValues(alpha: 0.35)),
        boxShadow: [
          BoxShadow(
            color: accent.withValues(alpha: 0.12),
            blurRadius: 24,
            offset: const Offset(0, 8),
          ),
        ],
      ),
      child: Row(
        children: [
          Container(
            width: 44,
            height: 44,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: accent.withValues(alpha: 0.15),
              border: Border.all(color: accent.withValues(alpha: 0.4)),
            ),
            child: Icon(Icons.dns_rounded, color: accent, size: 22),
          ),
          const SizedBox(width: 14),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'ASTRA HOMELAB',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: accent,
                        fontWeight: FontWeight.w800,
                        letterSpacing: 1.5,
                      ),
                ),
                const SizedBox(height: 4),
                Text(
                  'Homelab · Ubuntu',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        color: AppColors.text,
                        fontWeight: FontWeight.w700,
                      ),
                ),
                const SizedBox(height: 2),
                Text(
                  'Your homelab node — watching watts and system health.',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: AppColors.textDim,
                        height: 1.35,
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
