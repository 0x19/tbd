import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:tbd_mobile/l10n/app_localizations.dart';
import 'package:tbd_ui/tbd_ui.dart';

/// The shell: the kit's theme in both modes following the system, the two
/// languages, and the router. Products add routes and features; they do not
/// touch this.
class App extends StatelessWidget {
  const App({required this.home, super.key});

  final Widget home;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      onGenerateTitle: (context) => AppLocalizations.of(context).appTitle,
      theme: TbdTheme.light(),
      darkTheme: TbdTheme.dark(),
      localizationsDelegates: const [
        AppLocalizations.delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
      ],
      supportedLocales: AppLocalizations.supportedLocales,
      home: home,
    );
  }
}
