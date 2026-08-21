import 'package:flutter/material.dart';

/// Native brand palette (from app CSS tokens).
abstract final class AppColors {
  static const Color primary = Color(0xFFD2500A);
  static const Color primaryLight = Color(0xFFE8702A);
  static const Color gold = Color(0xFFD4A017);
  static const Color background = Color(0xFFFFFFFF);
  static const Color foreground = Color(0xFF0A0A0A);
  static const Color muted = Color(0xFFF5F5F5);
  static const Color mutedForeground = Color(0xFF737373);
  static const Color border = Color(0xFFECECEC);
  static const Color destructive = Color(0xFFEF4444);
  static const Color live = Color(0xFF16A34A);

  static const Color darkBackground = Color(0xFF0A0A0A);
  static const Color darkForeground = Color(0xFFFAFAFA);
  static const Color darkMuted = Color(0xFF292929);
  static const Color darkMutedForeground = Color(0xFFA3A3A3);
  static const Color darkBorder = Color(0xFF262626);
}
