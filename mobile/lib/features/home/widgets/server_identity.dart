import 'package:flutter/material.dart';

import '../../../theme/app_colors.dart';

/// Fun, fixed identity strip for the company home server.
class ServerIdentity extends StatelessWidget {
  const ServerIdentity({super.key});

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.fromLTRB(16, 14, 16, 14),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(12),
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            AppColors.foreground.withValues(alpha: 0.92),
            const Color(0xFF1A1A1A),
          ],
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'NATIVE HOME SERVER',
            style: Theme.of(context).textTheme.labelSmall?.copyWith(
                  color: AppColors.primaryLight,
                  fontWeight: FontWeight.w800,
                  letterSpacing: 1.4,
                ),
          ),
          const SizedBox(height: 6),
          Text(
            'ThinkPad · Ubuntu',
            style: Theme.of(context).textTheme.titleMedium?.copyWith(
                  color: Colors.white,
                  fontWeight: FontWeight.w700,
                ),
          ),
          const SizedBox(height: 4),
          Text(
            'Keeping the office lights on — one watt at a time.',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Colors.white.withValues(alpha: 0.65),
                  height: 1.35,
                ),
          ),
        ],
      ),
    );
  }
}
