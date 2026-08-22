import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'router/app_router.dart';
import 'theme/app_theme.dart';

class AstraApp extends ConsumerWidget {
  const AstraApp({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final router = ref.watch(routerProvider);
    return MaterialApp.router(
      title: 'Astra',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.futuristic(),
      routerConfig: router,
    );
  }
}
