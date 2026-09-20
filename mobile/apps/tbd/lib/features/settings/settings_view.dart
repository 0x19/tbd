import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tbd_mobile/app/providers.dart';
import 'package:tbd_mobile/l10n/app_localizations.dart';
import 'package:tbd_ui/tbd_ui.dart';

/// Where the build points and when the token runs out; sign out.
class SettingsView extends ConsumerWidget {
  const SettingsView({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l = AppLocalizations.of(context);
    final env = ref.watch(envProvider);
    final session = ref.watch(sessionProvider);
    // Watched so the expiry follows a refresh.
    ref.watch(sessionStateProvider);
    final expires = session.tokens?.expiresAt.toLocal();
    return Scaffold(
      appBar: AppBar(title: Text(l.settingsTitle)),
      body: ListView(
        children: [
          _Row(l.settingsEnvironment, env.name),
          _Row(l.settingsIssuer, env.authIssuer.host),
          _Row(l.settingsApi, env.apiBaseUrl.host),
          if (expires != null)
            _Row(
              l.settingsExpires,
              MaterialLocalizations.of(
                context,
              ).formatTimeOfDay(TimeOfDay.fromDateTime(expires)),
            ),
          const Divider(),
          ListTile(
            leading: const Icon(Icons.logout),
            title: Text(l.signOut),
            onTap: session.signOut,
          ),
        ],
      ),
    );
  }
}

class _Row extends StatelessWidget {
  const _Row(this.label, this.value);
  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => ListTile(
    title: Text(label),
    trailing: Text(value, style: mono(context, size: 13)),
  );
}
