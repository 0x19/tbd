// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Croatian (`hr`).
class AppLocalizationsHr extends AppLocalizations {
  AppLocalizationsHr([String locale = 'hr']) : super(locale);

  @override
  String get appTitle => 'tbd';

  @override
  String get signInTitle => 'Prijava';

  @override
  String get signInBody =>
      'Prijavite se svojim računom. Otvara se preglednik, tamo se prijavite i vraćate se ovamo.';

  @override
  String get signInButton => 'Prijavi se';

  @override
  String get signInBusy => 'Čekam preglednik…';

  @override
  String get signInCancelled => 'Prijava je prekinuta.';

  @override
  String get helloTitle => 'Pozdrav';

  @override
  String helloName(String name) {
    return 'Pozdrav, $name';
  }

  @override
  String helloRole(String role) {
    return 'Uloga: $role';
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
  String get helloRefresh => 'Pitaj ponovno';

  @override
  String get settingsTitle => 'Postavke';

  @override
  String get settingsEnvironment => 'Okruženje';

  @override
  String get settingsIssuer => 'Davatelj identiteta';

  @override
  String get settingsApi => 'API';

  @override
  String get signOut => 'Odjava';

  @override
  String get errorNetwork => 'Poslužitelj nije dostupan.';

  @override
  String get errorUnauthenticated => 'Sesija je istekla. Prijavite se ponovno.';

  @override
  String get errorForbidden => 'Ovaj račun to ne smije.';

  @override
  String errorServer(int status) {
    return 'Poslužitelj je odgovorio greškom ($status).';
  }

  @override
  String get errorCancelled => 'Prekinuto.';

  @override
  String get errorConfig => 'Aplikacija je izgrađena bez konfiguracije.';

  @override
  String get retry => 'Pokušaj ponovno';

  @override
  String get helloBody => 'Pozadina, kroz oba ulaza, s jednim tokenom.';

  @override
  String get helloLoading => 'Pitam…';

  @override
  String helloVersion(String version) {
    return 'protokol $version';
  }

  @override
  String get settingsExpires => 'Token istječe';

  @override
  String get openSettings => 'Postavke';
}
