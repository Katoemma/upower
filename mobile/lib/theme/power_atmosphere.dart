import 'package:flutter/material.dart';

/// Base neutrals for the futuristic shell (not brand-locked).
abstract final class AppColors {
  static const Color voidBlack = Color(0xFF05070D);
  static const Color panel = Color(0xFF0C101A);
  static const Color panelElevated = Color(0xFF141A28);
  static const Color stroke = Color(0xFF243049);
  static const Color text = Color(0xFFF4F7FF);
  static const Color textDim = Color(0xFF8B97B0);
  static const Color destructive = Color(0xFFF87171);

  // Status accents
  static const Color ac = Color(0xFF34D399);
  static const Color batteryHigh = Color(0xFF38BDF8);
  static const Color batteryMid = Color(0xFFFBBF24);
  static const Color batteryLow = Color(0xFFFB923C);
  static const Color batteryCritical = Color(0xFFF87171);

  // App-shell aliases (login / settings / events)
  static const Color primary = batteryHigh;
  static const Color primaryLight = ac;
  static const Color background = voidBlack;
  static const Color foreground = text;
  static const Color muted = panelElevated;
  static const Color mutedForeground = textDim;
  static const Color border = stroke;
  static const Color live = ac;
  static const Color gold = batteryMid;
}

enum PowerMood {
  acOnline,
  batteryHigh,
  batteryMid,
  batteryLow,
  batteryCritical,
  unknown,
}

/// Status → palette. Drives home atmosphere, accents, and copy.
class PowerAtmosphere {
  const PowerAtmosphere({
    required this.mood,
    required this.accent,
    required this.accentSoft,
    required this.glow,
    required this.label,
    required this.tagline,
    required this.gradient,
  });

  final PowerMood mood;
  final Color accent;
  final Color accentSoft;
  final Color glow;
  final String label;
  final String tagline;
  final List<Color> gradient;

  factory PowerAtmosphere.fromPower({
    required bool acConnected,
    double? percentage,
  }) {
    if (acConnected) {
      return const PowerAtmosphere(
        mood: PowerMood.acOnline,
        accent: AppColors.ac,
        accentSoft: Color(0xFF6EE7B7),
        glow: Color(0xFF10B981),
        label: 'GRID ONLINE',
        tagline: 'Mains feeding the homelab',
        gradient: [Color(0xFF04140F), Color(0xFF05070D), Color(0xFF071A14)],
      );
    }

    final pct = percentage ?? 0;
    if (pct >= 70) {
      return const PowerAtmosphere(
        mood: PowerMood.batteryHigh,
        accent: AppColors.batteryHigh,
        accentSoft: Color(0xFF7DD3FC),
        glow: Color(0xFF0284C7),
        label: 'BATTERY STRONG',
        tagline: 'Running cool on internal cells',
        gradient: [Color(0xFF06101A), Color(0xFF05070D), Color(0xFF0A1524)],
      );
    }
    if (pct >= 40) {
      return const PowerAtmosphere(
        mood: PowerMood.batteryMid,
        accent: AppColors.batteryMid,
        accentSoft: Color(0xFFFDE68A),
        glow: Color(0xFFD97706),
        label: 'BATTERY FAIR',
        tagline: 'Still fine — keep an eye on the wall',
        gradient: [Color(0xFF141008), Color(0xFF05070D), Color(0xFF1A1408)],
      );
    }
    if (pct >= 20) {
      return const PowerAtmosphere(
        mood: PowerMood.batteryLow,
        accent: AppColors.batteryLow,
        accentSoft: Color(0xFFFDBA74),
        glow: Color(0xFFEA580C),
        label: 'BATTERY LOW',
        tagline: 'Plug in before the lights dim',
        gradient: [Color(0xFF180C06), Color(0xFF05070D), Color(0xFF1F1008)],
      );
    }
    return const PowerAtmosphere(
      mood: PowerMood.batteryCritical,
      accent: AppColors.batteryCritical,
      accentSoft: Color(0xFFFCA5A5),
      glow: Color(0xFFDC2626),
      label: 'CRITICAL',
      tagline: 'Home server needs power — now',
      gradient: [Color(0xFF1A0608), Color(0xFF05070D), Color(0xFF22080C)],
    );
  }
}
