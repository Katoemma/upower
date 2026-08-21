import 'package:dotlottie_flutter/dotlottie_flutter.dart';
import 'package:flutter/material.dart';

import '../../../theme/power_atmosphere.dart';

class StatusHero extends StatelessWidget {
  const StatusHero({
    super.key,
    required this.atmosphere,
    required this.acConnected,
    required this.percentage,
    required this.stateLabel,
    this.subtitle,
  });

  final PowerAtmosphere atmosphere;
  final bool acConnected;
  final double? percentage;
  final String stateLabel;
  final String? subtitle;

  @override
  Widget build(BuildContext context) {
    final pct = percentage?.round();
    final textTheme = Theme.of(context).textTheme;
    final accent = atmosphere.accent;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SizedBox(
          height: 240,
          child: Stack(
            alignment: Alignment.center,
            children: [
              AnimatedContainer(
                duration: const Duration(milliseconds: 700),
                curve: Curves.easeOutCubic,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  boxShadow: [
                    BoxShadow(
                      color: atmosphere.glow.withValues(alpha: 0.45),
                      blurRadius: 80,
                      spreadRadius: 8,
                    ),
                    BoxShadow(
                      color: accent.withValues(alpha: 0.22),
                      blurRadius: 140,
                      spreadRadius: 24,
                    ),
                  ],
                ),
                width: 160,
                height: 160,
              ),
              Opacity(
                opacity: acConnected ? 1 : 0.72,
                child: const DotLottieView(
                  source: 'assets/lottie/Charging.lottie',
                  sourceType: 'asset',
                  autoplay: true,
                  loop: true,
                ),
              ),
              Positioned(
                top: 12,
                child: _HudBadge(
                  text: atmosphere.label,
                  color: accent,
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 4),
        Row(
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            Expanded(
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.baseline,
                textBaseline: TextBaseline.alphabetic,
                children: [
                  AnimatedDefaultTextStyle(
                    duration: const Duration(milliseconds: 500),
                    style: textTheme.displayLarge!.copyWith(
                      fontWeight: FontWeight.w800,
                      height: 0.92,
                      letterSpacing: -3,
                      fontSize: 76,
                      color: accent,
                      shadows: [
                        Shadow(
                          color: atmosphere.glow.withValues(alpha: 0.55),
                          blurRadius: 24,
                        ),
                      ],
                    ),
                    child: Text(pct == null ? '—' : '$pct'),
                  ),
                  Text(
                    '%',
                    style: textTheme.headlineSmall?.copyWith(
                      color: AppColors.textDim,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ],
              ),
            ),
            Column(
              crossAxisAlignment: CrossAxisAlignment.end,
              children: [
                _PowerModeChip(
                  acConnected: acConnected,
                  accent: accent,
                ),
                const SizedBox(height: 10),
                Text(
                  stateLabel,
                  textAlign: TextAlign.end,
                  style: textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.w700,
                    color: AppColors.text,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  atmosphere.tagline,
                  textAlign: TextAlign.end,
                  style: textTheme.bodySmall?.copyWith(
                    color: AppColors.textDim,
                  ),
                ),
                if (subtitle != null) ...[
                  const SizedBox(height: 4),
                  Text(
                    subtitle!,
                    textAlign: TextAlign.end,
                    style: textTheme.bodyMedium?.copyWith(
                      color: atmosphere.accentSoft,
                      fontWeight: FontWeight.w600,
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

class _HudBadge extends StatelessWidget {
  const _HudBadge({required this.text, required this.color});

  final String text;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 450),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 5),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: color.withValues(alpha: 0.45)),
        boxShadow: [
          BoxShadow(color: color.withValues(alpha: 0.25), blurRadius: 16),
        ],
      ),
      child: Text(
        text,
        style: TextStyle(
          color: color,
          fontWeight: FontWeight.w800,
          fontSize: 11,
          letterSpacing: 1.6,
        ),
      ),
    );
  }
}

class _PowerModeChip extends StatelessWidget {
  const _PowerModeChip({
    required this.acConnected,
    required this.accent,
  });

  final bool acConnected;
  final Color accent;

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 350),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      decoration: BoxDecoration(
        color: accent.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: accent.withValues(alpha: 0.35)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(
            acConnected ? Icons.bolt_rounded : Icons.battery_charging_full_rounded,
            size: 16,
            color: accent,
          ),
          const SizedBox(width: 6),
          Text(
            acConnected ? 'Mains' : 'Battery',
            style: TextStyle(
              color: accent,
              fontWeight: FontWeight.w700,
              fontSize: 13,
            ),
          ),
        ],
      ),
    );
  }
}
