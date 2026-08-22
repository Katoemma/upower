import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/config.dart';
import '../../../core/providers.dart';
import '../../../theme/app_colors.dart';
import '../../../theme/astra_shell.dart';
import '../../../theme/power_atmosphere.dart';
import '../../../widgets/glass_panel.dart';
import '../auth/auth_controller.dart';
import '../auth/widgets/primary_button.dart';
import '../home/power_controller.dart';

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
      SnackBar(
        content: const Text('Server URL saved'),
        backgroundColor: AppColors.panelElevated,
        behavior: SnackBarBehavior.floating,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final auth = ref.watch(authControllerProvider);
    final power = ref.watch(powerSnapshotProvider);
    final atmo = PowerAtmosphere.fromPower(
      acConnected: power.acConnected,
      percentage: power.percentage,
    );

    return AstraPageScaffold(
      atmosphere: atmo,
      appBar: AstraShell.appBar(context, 'Settings'),
      body: ListView(
        padding: EdgeInsets.fromLTRB(
          20,
          8,
          20,
          MediaQuery.paddingOf(context).bottom + 32,
        ),
        children: [
          GlassPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                GlassSectionLabel(label: 'Account', accent: atmo.accentSoft),
                const SizedBox(height: 10),
                Text(
                  auth.email ?? '—',
                  style: const TextStyle(
                    fontWeight: FontWeight.w700,
                    fontSize: 16,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 14),
          GlassPanel(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                GlassSectionLabel(label: 'Server URL', accent: atmo.accentSoft),
                const SizedBox(height: 12),
                TextField(
                  controller: _url,
                  keyboardType: TextInputType.url,
                  style: const TextStyle(color: AppColors.text),
                  decoration: InputDecoration(
                    hintText: AppConfig.defaultServerUrl,
                    filled: true,
                    fillColor: Colors.white.withValues(alpha: 0.05),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(14),
                      borderSide: BorderSide(
                        color: Colors.white.withValues(alpha: 0.1),
                      ),
                    ),
                    enabledBorder: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(14),
                      borderSide: BorderSide(
                        color: Colors.white.withValues(alpha: 0.1),
                      ),
                    ),
                    focusedBorder: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(14),
                      borderSide: BorderSide(color: atmo.accent, width: 1.5),
                    ),
                  ),
                ),
                const SizedBox(height: 10),
                Wrap(
                  spacing: 8,
                  children: [
                    ActionChip(
                      label: const Text('Production'),
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
              ],
            ),
          ),
          const SizedBox(height: 14),
          GlassPanel(
            padding: const EdgeInsets.all(16),
            child: Text(
              'Push notifications register automatically after login '
              '(FCM token → server).',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: AppColors.textDim,
                    height: 1.4,
                  ),
            ),
          ),
          const SizedBox(height: 20),
          OutlinedButton(
            onPressed: () async {
              await ref.read(authControllerProvider.notifier).logout();
              if (context.mounted) context.go('/login');
            },
            style: OutlinedButton.styleFrom(
              foregroundColor: AppColors.destructive,
              side: BorderSide(color: AppColors.destructive.withValues(alpha: 0.5)),
              minimumSize: const Size.fromHeight(50),
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(14)),
            ),
            child: const Text('Log out'),
          ),
        ],
      ),
    );
  }
}
