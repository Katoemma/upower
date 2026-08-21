import 'package:flutter_test/flutter_test.dart';
import 'package:mobile/theme/app_colors.dart';

void main() {
  test('brand primary matches Native orange', () {
    expect(AppColors.primary.toARGB32(), 0xFFD2500A);
  });
}
