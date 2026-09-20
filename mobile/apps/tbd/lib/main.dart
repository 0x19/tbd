import 'package:flutter/material.dart';
import 'package:tbd_mobile/app/app.dart';
import 'package:tbd_mobile/app/env.dart';
import 'package:tbd_mobile/l10n/app_localizations.dart';
import 'package:tbd_ui/tbd_ui.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  // A build without its configuration fails here, naming the key, not on the
  // first screen that needs a URL.
  final env = hasEnv ? readEnv() : null;
  runApp(App(home: Foundation(envName: env?.name)));
}

/// The placeholder the foundation ships with; the sign-in and hello screens
/// replace it. Labelled as such on the screen, in both languages' terms.
class Foundation extends StatelessWidget {
  const Foundation({required this.envName, super.key});

  final String? envName;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final c = TbdColors.of(context);
    return Scaffold(
      appBar: AppBar(title: Text(l.appTitle)),
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              l.helloTitle,
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            const SizedBox(height: 8),
            Text(
              envName ?? l.errorConfig,
              style: mono(context, color: c.mutedForeground),
            ),
          ],
        ),
      ),
    );
  }
}
