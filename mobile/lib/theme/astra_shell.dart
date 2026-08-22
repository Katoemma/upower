import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'power_atmosphere.dart';

/// Shared gradient shell used across Astra screens (matches Home).
abstract final class AstraShell {
  static const logoAsset = 'assets/images/logo.png';

  static PowerAtmosphere get defaultAtmosphere =>
      PowerAtmosphere.fromPower(acConnected: false, percentage: 78);

  static SystemUiOverlayStyle get overlayStyle => SystemUiOverlayStyle.light.copyWith(
        statusBarColor: Colors.transparent,
        statusBarIconBrightness: Brightness.light,
        systemStatusBarContrastEnforced: false,
        systemNavigationBarColor: AppColors.voidBlack,
        systemNavigationBarIconBrightness: Brightness.light,
      );

  static BoxDecoration gradientDecoration(PowerAtmosphere atmosphere) {
    return BoxDecoration(
      gradient: LinearGradient(
        begin: Alignment.topRight,
        end: Alignment.bottomLeft,
        colors: [
          atmosphere.glow.withValues(alpha: 0.42),
          atmosphere.gradient[0],
          atmosphere.gradient.length > 2
              ? atmosphere.gradient[2]
              : atmosphere.gradient[1],
        ],
        stops: const [0.0, 0.38, 1.0],
      ),
    );
  }

  static AppBar appBar(BuildContext context, String title) {
    return AppBar(
      backgroundColor: Colors.transparent,
      elevation: 0,
      scrolledUnderElevation: 0,
      surfaceTintColor: Colors.transparent,
      iconTheme: const IconThemeData(color: AppColors.text),
      title: Text(
        title,
        style: Theme.of(context).textTheme.titleLarge?.copyWith(
              color: AppColors.text,
              fontWeight: FontWeight.w700,
              letterSpacing: -0.2,
            ),
      ),
    );
  }
}

class AstraPageScaffold extends StatelessWidget {
  const AstraPageScaffold({
    super.key,
    required this.atmosphere,
    required this.body,
    this.appBar,
  });

  final PowerAtmosphere atmosphere;
  final Widget body;
  final PreferredSizeWidget? appBar;

  @override
  Widget build(BuildContext context) {
    return AnnotatedRegion<SystemUiOverlayStyle>(
      value: AstraShell.overlayStyle,
      child: SizedBox.expand(
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 700),
          curve: Curves.easeOutCubic,
          decoration: AstraShell.gradientDecoration(atmosphere),
          child: Scaffold(
            backgroundColor: Colors.transparent,
            appBar: appBar,
            body: body,
          ),
        ),
      ),
    );
  }
}
