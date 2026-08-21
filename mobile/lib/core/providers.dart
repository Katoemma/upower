import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'api_client.dart';
import 'config.dart';
import 'secure_storage.dart';
import 'ws_client.dart';
import '../features/push/fcm_service.dart';

final appConfigProvider = Provider<AppConfig>((ref) => AppConfig());

final secureStoreProvider = Provider<SecureStore>((ref) => SecureStore());

final apiClientProvider = Provider<ApiClient>((ref) {
  return ApiClient(ref.watch(appConfigProvider));
});

final wsClientProvider = Provider<WsClient>((ref) {
  return WsClient(ref.watch(appConfigProvider));
});

final fcmServiceProvider = Provider<FcmService>((ref) {
  return FcmService(ref.watch(apiClientProvider));
});
