import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/config.dart';
import '../../../core/providers.dart';
import '../../../theme/app_colors.dart';
import '../../../theme/astra_shell.dart';
import '../../../widgets/glass_panel.dart';
import 'auth_controller.dart';
import 'widgets/auth_text_field.dart';
import 'widgets/primary_button.dart';

class LoginScreen extends ConsumerStatefulWidget {
  const LoginScreen({super.key});

  @override
  ConsumerState<LoginScreen> createState() => _LoginScreenState();
}

class _LoginScreenState extends ConsumerState<LoginScreen> {
  final _email = TextEditingController();
  final _password = TextEditingController();
  final _serverUrl = TextEditingController();
  bool _showAdvanced = false;

  @override
  void initState() {
    super.initState();
    _serverUrl.text = ref.read(appConfigProvider).serverUrl;
  }

  @override
  void dispose() {
    _email.dispose();
    _password.dispose();
    _serverUrl.dispose();
    super.dispose();
  }

  Future<void> _submit() async {
    await ref.read(appConfigProvider).setServerUrl(_serverUrl.text);
    final ok = await ref.read(authControllerProvider.notifier).login(
          _email.text,
          _password.text,
        );
    if (ok && mounted) context.go('/home');
  }

  @override
  Widget build(BuildContext context) {
    final auth = ref.watch(authControllerProvider);
    final atmo = AstraShell.defaultAtmosphere;
    final topInset = MediaQuery.paddingOf(context).top;

    return AstraPageScaffold(
      atmosphere: atmo,
      body: SafeArea(
        top: false,
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 440),
            child: SingleChildScrollView(
              padding: EdgeInsets.fromLTRB(24, topInset + 28, 24, 32),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Center(
                    child: Image.asset(
                      AstraShell.logoAsset,
                      height: 52,
                      fit: BoxFit.contain,
                    ),
                  ),
                  const SizedBox(height: 28),
                  Text(
                    'Welcome back',
                    textAlign: TextAlign.center,
                    style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                          fontWeight: FontWeight.w700,
                          fontSize: 28,
                          letterSpacing: -0.5,
                          color: AppColors.text,
                        ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Sign in to monitor your homelab',
                    textAlign: TextAlign.center,
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                          color: AppColors.textDim,
                        ),
                  ),
                  const SizedBox(height: 28),
                  GlassPanel(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        AppTextField(
                          controller: _email,
                          label: 'Email',
                          keyboardType: TextInputType.emailAddress,
                          textInputAction: TextInputAction.next,
                          autofillHints: const [AutofillHints.email],
                        ),
                        const SizedBox(height: 14),
                        AppTextField(
                          controller: _password,
                          label: 'Password',
                          obscureText: true,
                          textInputAction: TextInputAction.done,
                          autofillHints: const [AutofillHints.password],
                          onSubmitted: (_) => _submit(),
                        ),
                        if (auth.error != null) ...[
                          const SizedBox(height: 12),
                          Text(
                            auth.error!,
                            style: const TextStyle(color: AppColors.destructive),
                          ),
                        ],
                        const SizedBox(height: 20),
                        PrimaryButton(
                          label: 'Sign in',
                          loading: auth.loading,
                          onPressed: _submit,
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 16),
                  GlassPanel(
                    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                    child: Row(
                      children: [
                        Icon(Icons.link_rounded, size: 18, color: atmo.accentSoft),
                        const SizedBox(width: 10),
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Text(
                                AppConfig.defaultServerUrl.replaceFirst('https://', ''),
                                style: const TextStyle(
                                  fontWeight: FontWeight.w600,
                                  fontSize: 13,
                                ),
                              ),
                              Text(
                                'Production server',
                                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                                      color: AppColors.textDim,
                                    ),
                              ),
                            ],
                          ),
                        ),
                        TextButton(
                          onPressed: () =>
                              setState(() => _showAdvanced = !_showAdvanced),
                          child: Text(_showAdvanced ? 'Hide' : 'Change'),
                        ),
                      ],
                    ),
                  ),
                  if (_showAdvanced) ...[
                    const SizedBox(height: 12),
                    GlassPanel(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          const GlassSectionLabel(label: 'Server URL'),
                          const SizedBox(height: 12),
                          AppTextField(
                            controller: _serverUrl,
                            label: 'Server URL',
                            keyboardType: TextInputType.url,
                            textInputAction: TextInputAction.next,
                          ),
                          const SizedBox(height: 10),
                          Wrap(
                            spacing: 8,
                            children: [
                              ActionChip(
                                label: const Text('Production'),
                                onPressed: () {
                                  _serverUrl.text = AppConfig.defaultServerUrl;
                                },
                              ),
                              ActionChip(
                                label: const Text('Emulator'),
                                onPressed: () {
                                  _serverUrl.text = AppConfig.emulatorServerUrl;
                                },
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
