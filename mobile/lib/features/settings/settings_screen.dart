import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/config.dart';
import '../../../core/providers.dart';
import '../../../theme/app_colors.dart';
import '../auth/auth_controller.dart';
import '../auth/widgets/primary_button.dart';

class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  late final TextEditingController _url;

  @override
  void initState() {
    super.initState();
    _url = TextEditingController(text: ref.read(appConfigProvider).serverUrl);
  }

  @override
  void dispose() {
    _url.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    await ref.read(appConfigProvider).setServerUrl(_url.text);
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Server URL saved')),
    );
  }

  @override
  Widget build(BuildContext context) {
    final auth = ref.watch(authControllerProvider);

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(24, 8, 24, 32),
        children: [
          Text('Account', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          Text(auth.email ?? '—', style: const TextStyle(fontWeight: FontWeight.w600)),
          const SizedBox(height: 24),
          Text('Server URL', style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          TextField(
            controller: _url,
            keyboardType: TextInputType.url,
            decoration: const InputDecoration(hintText: 'https://….trycloudflare.com'),
          ),
          const SizedBox(height: 8),
          Wrap(
            spacing: 8,
            children: [
              ActionChip(
                label: const Text('Tunnel default'),
                onPressed: () {
                  _url.text = AppConfig.defaultServerUrl;
                },
              ),
              ActionChip(
                label: const Text('Emulator'),
                onPressed: () {
                  _url.text = AppConfig.emulatorServerUrl;
                },
              ),
            ],
          ),
          const SizedBox(height: 16),
          PrimaryButton(label: 'Save server URL', onPressed: _save),
          const SizedBox(height: 32),
          const Divider(),
          const SizedBox(height: 12),
          Text(
            'Push notifications register automatically after login '
            '(FCM token → server SQLite).',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: AppColors.mutedForeground,
                ),
          ),
          const SizedBox(height: 24),
          OutlinedButton(
            onPressed: () async {
              await ref.read(authControllerProvider.notifier).logout();
              if (context.mounted) context.go('/login');
            },
            style: OutlinedButton.styleFrom(
              foregroundColor: AppColors.destructive,
              side: const BorderSide(color: AppColors.border),
              minimumSize: const Size.fromHeight(48),
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
            ),
            child: const Text('Log out'),
          ),
        ],
      ),
    );
  }
}
