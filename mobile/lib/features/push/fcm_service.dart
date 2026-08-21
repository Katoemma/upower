import 'package:firebase_messaging/firebase_messaging.dart';
import 'package:flutter/foundation.dart';

import '../../core/api_client.dart';

class FcmService {
  FcmService(this._api);

  final ApiClient _api;
  final FirebaseMessaging _messaging = FirebaseMessaging.instance;

  Future<void> init() async {
    await _messaging.requestPermission(alert: true, badge: true, sound: true);
    FirebaseMessaging.onMessage.listen((message) {
      debugPrint('FCM foreground: ${message.notification?.title}');
    });
  }

  Future<String?> registerWithServer() async {
    final token = await _messaging.getToken();
    if (token == null || token.isEmpty) {
      debugPrint('FCM token unavailable');
      return null;
    }
    await _api.registerPushToken(token);
    debugPrint('FCM token registered with API');
    return token;
  }
}
