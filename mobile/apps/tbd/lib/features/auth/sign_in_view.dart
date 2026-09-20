import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:tbd_mobile/app/errors.dart';
import 'package:tbd_mobile/features/auth/sign_in_view_model.dart';
import 'package:tbd_mobile/l10n/app_localizations.dart';
import 'package:tbd_ui/tbd_ui.dart';

/// One button. The SSO's own pages do the rest, in the system browser.
class SignInView extends ConsumerWidget {
  const SignInView({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l = AppLocalizations.of(context);
    final c = TbdColors.of(context);
    final ui = ref.watch(signInViewModelProvider);
    final error = ui.error;
    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                l.signInTitle,
                style: Theme.of(context).textTheme.headlineMedium,
              ),
              const SizedBox(height: 8),
              Text(l.signInBody, style: TextStyle(color: c.mutedForeground)),
              const SizedBox(height: 24),
              FilledButton(
                onPressed: ui.busy
                    ? null
                    : ref.read(signInViewModelProvider.notifier).signIn,
                child: Text(ui.busy ? l.signInBusy : l.signInButton),
              ),
              if (error != null) ...[
                const SizedBox(height: 16),
                Text(
                  error is CancelledError
                      ? l.signInCancelled
                      : describe(l, error),
                  style: TextStyle(color: c.destructive),
                  textAlign: TextAlign.center,
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
