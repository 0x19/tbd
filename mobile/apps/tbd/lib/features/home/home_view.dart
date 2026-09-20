import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_mobile/app/providers.dart';
import 'package:tbd_mobile/l10n/app_localizations.dart';
import 'package:tbd_ui/tbd_ui.dart';

/// Hello, by the name the ID token carries. The backend's own hello follows.
class HomeView extends ConsumerWidget {
  const HomeView({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l = AppLocalizations.of(context);
    final c = TbdColors.of(context);
    final state = ref.watch(sessionStateProvider);
    final who = state is SignedIn ? state.principal : null;
    return Scaffold(
      appBar: AppBar(
        title: Text(l.helloTitle),
        actions: [
          IconButton(
            tooltip: l.signOut,
            icon: const Icon(Icons.logout),
            onPressed: () => ref.read(sessionProvider).signOut(),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              l.helloName(who?.displayName ?? ''),
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            if (who?.email != null) ...[
              const SizedBox(height: 4),
              Text(who!.email!, style: mono(context, color: c.mutedForeground)),
            ],
          ],
        ),
      ),
    );
  }
}
