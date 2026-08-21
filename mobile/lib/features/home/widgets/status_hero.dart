import 'package:dotlottie_flutter/dotlottie_flutter.dart';
import 'package:flutter/material.dart';

import '../../../theme/app_colors.dart';

/// Hero: Lottie + big charge readout.
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
    final textTheme = Theme.of(context).textTheme;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SizedBox(
          height: 220,
          child: Stack(
            alignment: Alignment.center,
            children: [
              // Soft glow behind animation
              Positioned.fill(
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    gradient: RadialGradient(
                      colors: [
                        AppColors.primary.withValues(alpha: acConnected ? 0.18 : 0.08),
                        Colors.transparent,
                      ],
                      radius: 0.72,
                    ),
                  ),
                ),
              ),
              Opacity(
                opacity: acConnected ? 1 : 0.55,
                child: const DotLottieView(
                  source: 'assets/lottie/Charging.lottie',
                  sourceType: 'asset',
                  autoplay: true,
                  loop: true,
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 8),
        Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Expanded(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.baseline,
                textBaseline: TextBaseline.alphabetic,
                children: [
                  Text(
                    pct == null ? '—' : '$pct',
                    style: textTheme.displayLarge?.copyWith(
                      fontWeight: FontWeight.w800,
                      height: 0.95,
                      letterSpacing: -2.5,
                      fontSize: 72,
                      color: AppColors.foreground,
                    ),
                  ),
                  Text(
                    '%',
                    style: textTheme.headlineSmall?.copyWith(
                      color: AppColors.mutedForeground,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ],
              ),
            ),
            Column(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                _PowerModeChip(acConnected: acConnected),
                const SizedBox(height: 10),
                Text(
                  stateLabel,
                  textAlign: TextAlign.end,
                  style: textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.w700,
                  ),
                ),
                if (subtitle != null) ...[
                  const SizedBox(height: 4),
                  Text(
                    subtitle!,
                    textAlign: TextAlign.end,
                    style: textTheme.bodyMedium?.copyWith(
                      color: AppColors.mutedForeground,
                    ),
                  ),
                ],
              ],
            ),
          ],
        ),
      ],
    );
  }
}

class _PowerModeChip extends StatelessWidget {
  const _PowerModeChip({required this.acConnected});

  final bool acConnected;

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 280),
      curve: Curves.easeOutCubic,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: acConnected
            ? AppColors.primary.withValues(alpha: 0.12)
            : AppColors.muted,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: acConnected
              ? AppColors.primary.withValues(alpha: 0.28)
              : AppColors.border,
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            acConnected ? Icons.bolt_rounded : Icons.battery_std_rounded,
            size: 16,
            color: acConnected ? AppColors.primary : AppColors.foreground,
          ),
          const SizedBox(width: 6),
          Text(
            acConnected ? 'Mains power' : 'On battery',
            style: TextStyle(
              color: acConnected ? AppColors.primary : AppColors.foreground,
              fontWeight: FontWeight.w700,
              fontSize: 13,
            ),
          ),
        ],
      ),
    );
  }
}
