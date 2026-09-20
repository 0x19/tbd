// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get appTitle => 'tbd';

  @override
  String get signInTitle => 'Sign in';

  @override
  String get signInBody =>
      'Use your account. The browser opens, you sign in there and come back.';

  @override
  String get signInButton => 'Sign in';

  @override
  String get signInBusy => 'Waiting for the browser…';

  @override
  String get signInCancelled => 'Sign-in was cancelled.';

  @override
  String get helloTitle => 'Hello';

  @override
  String helloName(String name) {
    return 'Hello, $name';
  }

  @override
  String helloRole(String role) {
    return 'Role: $role';
  }

  @override
  String get helloRest => 'REST /v1/me';

  @override
  String get helloGrpc => 'gRPC ProtocolService/Ping';

  @override
  String helloMillis(int ms) {
    return '$ms ms';
  }

  @override
  String get helloStub => 'stub';

  @override
  String get helloRefresh => 'Ask again';

  @override
  String get settingsTitle => 'Settings';

  @override
  String get settingsEnvironment => 'Environment';

  @override
  String get settingsIssuer => 'Identity provider';

  @override
  String get settingsApi => 'API';

  @override
  String get settingsVersion => 'Version';

  @override
  String get signOut => 'Sign out';

  @override
  String get errorNetwork => 'The server could not be reached.';

  @override
  String get errorUnauthenticated => 'The session ended. Sign in again.';

  @override
  String get errorForbidden => 'This account may not do that.';

  @override
  String errorServer(int status) {
    return 'The server answered with an error ($status).';
  }

  @override
  String get errorCancelled => 'Cancelled.';

  @override
  String get errorConfig => 'The app was built without its configuration.';

  @override
  String get retry => 'Try again';
}
