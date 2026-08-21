import 'package:flutter_test/flutter_test.dart';
import 'package:mobile/theme/power_atmosphere.dart';

void main() {
  test('AC online maps to green mood', () {
    final a = PowerAtmosphere.fromPower(acConnected: true, percentage: 50);
    expect(a.mood, PowerMood.acOnline);
    expect(a.accent.toARGB32(), AppColors.ac.toARGB32());
  });

  test('battery above 70 maps to blue', () {
    final a = PowerAtmosphere.fromPower(acConnected: false, percentage: 82);
    expect(a.mood, PowerMood.batteryHigh);
    expect(a.accent.toARGB32(), AppColors.batteryHigh.toARGB32());
  });

  test('critical maps under 20%', () {
    final a = PowerAtmosphere.fromPower(acConnected: false, percentage: 12);
    expect(a.mood, PowerMood.batteryCritical);
  });
}
