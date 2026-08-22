import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../core/providers.dart';
import '../../../theme/app_colors.dart';
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

    return Scaffold(
      backgroundColor: AppColors.voidBlack,
      body: DecoratedBox(
        decoration: const BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
            colors: [Color(0xFF0A1524), AppColors.voidBlack],
          ),
        ),
        child: SafeArea(
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 420),
            child: SingleChildScrollView(
              padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 32),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(
                    'ASTRA',
                    style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                          fontWeight: FontWeight.w800,
                          letterSpacing: 1.2,
                          color: AppColors.batteryHigh,
                        ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Homelab monitor · power & system telemetry',
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                          color: AppColors.textDim,
                        ),
                  ),
                  const SizedBox(height: 32),
                  AppTextField(
                    controller: _serverUrl,
                    label: 'Server URL',
                    keyboardType: TextInputType.url,
                    textInputAction: TextInputAction.next,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Default: Cloudflare tunnel. Change if the hostname rotates.',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: AppColors.textDim,
                        ),
                  ),
                  const SizedBox(height: 16),
                  AppTextField(
                    controller: _email,
                    label: 'Email',
                    keyboardType: TextInputType.emailAddress,
                    textInputAction: TextInputAction.next,
                    autofillHints: const [AutofillHints.email],
                  ),
                  const SizedBox(height: 12),
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
                  const SizedBox(height: 24),
                  PrimaryButton(
                    label: 'Sign in',
                    loading: auth.loading,
                    onPressed: _submit,
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
      ),
    );
  }
}
