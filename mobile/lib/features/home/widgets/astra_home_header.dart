import 'dart:ui';

import 'package:flutter/material.dart';

import '../../../theme/power_atmosphere.dart';

/// Greeting header inspired by modern dark dashboards: salutation + time greeting,
/// with glass action chips on the right.
class AstraHomeHeader extends StatelessWidget {
  const AstraHomeHeader({
    super.key,
    required this.displayName,
    required this.atmosphere,
    required this.live,
    required this.onEvents,
    required this.onSettings,
  });

  final String displayName;
  final PowerAtmosphere atmosphere;
  final bool live;
  final VoidCallback onEvents;
  final VoidCallback onSettings;

  static String greetingForTime([DateTime? now]) {
    final hour = (now ?? DateTime.now()).hour;
    if (hour < 12) return 'Good Morning';
    if (hour < 17) return 'Good Afternoon';
    return 'Good Evening';
  }

  static String nameFromEmail(String? email) {
    if (email == null || email.isEmpty) return 'there';
    final local = email.split('@').first;
    if (local.isEmpty) return 'there';
    final cleaned = local.replaceAll(RegExp(r'[._0-9]+'), ' ').trim();
    final word = cleaned.split(RegExp(r'\s+')).firstWhere(
          (w) => w.isNotEmpty,
          orElse: () => local,
        );
    if (word.isEmpty) return 'there';
    return word[0].toUpperCase() + word.substring(1).toLowerCase();
  }

  @override
  Widget build(BuildContext context) {
    final greeting = greetingForTime();

    return Padding(
      padding: const EdgeInsets.fromLTRB(0, 8, 0, 20),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Expanded(
            child: Material(
              color: Colors.transparent,
              child: InkWell(
                onTap: onSettings,
                borderRadius: BorderRadius.circular(12),
                child: Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Hi $displayName 👋',
                        style:
                            Theme.of(context).textTheme.bodyMedium?.copyWith(
                                  color: AppColors.text.withValues(alpha: 0.72),
                                  fontWeight: FontWeight.w400,
                                  fontSize: 15,
                                  height: 1.2,
                                ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        greeting,
                        style: Theme.of(context)
                            .textTheme
                            .headlineMedium
                            ?.copyWith(
                              color: AppColors.text,
                              fontWeight: FontWeight.w700,
                              fontSize: 30,
                              letterSpacing: -0.6,
                              height: 1.05,
                            ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
          const SizedBox(width: 12),
          _GlassIconButton(
            icon: Icons.search_rounded,
            tooltip: 'Events',
            onPressed: onEvents,
          ),
          const SizedBox(width: 10),
          _GlassIconButton(
            icon: Icons.notifications_none_rounded,
            tooltip: live ? 'Connected · Settings' : 'Settings',
            onPressed: onSettings,
            showLiveDot: live,
            accent: atmosphere.accent,
          ),
        ],
      ),
    );
  }
}

class _GlassIconButton extends StatelessWidget {
  const _GlassIconButton({
    required this.icon,
    required this.tooltip,
    required this.onPressed,
    this.showLiveDot = false,
    this.accent,
  });

  final IconData icon;
  final String tooltip;
  final VoidCallback onPressed;
  final bool showLiveDot;
  final Color? accent;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: tooltip,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(14),
        child: BackdropFilter(
          filter: ImageFilter.blur(sigmaX: 14, sigmaY: 14),
          child: Material(
            color: Colors.transparent,
            child: InkWell(
              onTap: onPressed,
              borderRadius: BorderRadius.circular(14),
              child: Container(
                width: 46,
                height: 46,
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(14),
                  color: Colors.white.withValues(alpha: 0.07),
                  border: Border.all(
                    color: Colors.white.withValues(alpha: 0.14),
                  ),
                ),
                child: Stack(
                  alignment: Alignment.center,
                  children: [
                    Icon(
                      icon,
                      size: 22,
                      color: AppColors.text.withValues(alpha: 0.92),
                    ),
                    if (showLiveDot)
                      Positioned(
                        top: 10,
                        right: 10,
                        child: Container(
                          width: 8,
                          height: 8,
                          decoration: BoxDecoration(
                            shape: BoxShape.circle,
                            color: accent ?? AppColors.ac,
                            border: Border.all(
                              color: AppColors.voidBlack.withValues(alpha: 0.6),
                              width: 1.5,
                            ),
                            boxShadow: [
                              BoxShadow(
                                color: (accent ?? AppColors.ac)
                                    .withValues(alpha: 0.55),
                                blurRadius: 6,
                              ),
                            ],
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
