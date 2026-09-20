import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:tbd_mobile/app/app.dart';
import 'package:tbd_mobile/app/providers.dart';
import 'package:tbd_mobile/app/router.dart';
import 'package:tbd_testing/tbd_testing.dart';

final env = Env.fromDefines(
  (k) =>
      {
        'env': 'test',
        'auth.issuer': 'https://auth.test',
        'auth.client_id': 'tbd-app',
        'auth.redirect_uri': 'tbd://callback',
        'auth.post_logout_redirect_uri': 'tbd://signed-out',
        'auth.scopes': 'openid tbd.api',
        'auth.audience': 'tbd-api',
        'api.base_url': 'https://api.test',
        'api.grpc_authority': 'api.test:443',
        'api.timeout_ms': '1000',
        'telemetry.service_name': 'tbd-mobile-test',
      }[k] ??
      '',
);

(Widget, FakeAuthRepository, Session) app() {
  final repo = FakeAuthRepository(
    token: 'access',
    idToken: fakeJwt({
      'sub': 'u-1',
      'email': 'n@example.test',
      'name': 'Nevio',
    }),
  );
  final session = Session(repository: repo, store: MemoryTokenStore());
  final widget = ProviderScope(
    overrides: [
      envProvider.overrideWithValue(env),
      sessionProvider.overrideWithValue(session),
    ],
    child: App(router: buildRouter(session)),
  );
  return (widget, repo, session);
}

void main() {
  testWidgets('signed out lands on sign-in; sign-in greets; sign-out returns', (
    tester,
  ) async {
    final (widget, repo, session) = app();
    await tester.pumpWidget(widget);
    // Splash until the store is read.
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    await session.restore();
    await tester.pumpAndSettle();
    expect(find.text('Sign in'), findsWidgets);

    await tester.tap(find.byType(FilledButton));
    await tester.pumpAndSettle();
    expect(find.text('Hello, Nevio'), findsOneWidget);
    expect(find.text('n@example.test'), findsOneWidget);
    expect(repo.signIns, 1);

    await tester.tap(find.byIcon(Icons.logout));
    await tester.pumpAndSettle();
    expect(find.byType(FilledButton), findsOneWidget);
    expect(repo.ended, 1);
  });

  testWidgets('a cancelled browser says so, in Croatian too', (tester) async {
    final (widget, repo, session) = app();
    await tester.pumpWidget(widget);
    await tester.binding.setLocale('hr', '');
    await session.restore();
    await tester.pumpAndSettle();
    expect(find.text('Prijavi se'), findsOneWidget);
    repo.cancelNext = true;
    await tester.tap(find.byType(FilledButton));
    await tester.pumpAndSettle();
    expect(find.text('Prijava je prekinuta.'), findsOneWidget);
  });

  testWidgets('a stored session skips sign-in', (tester) async {
    final (widget, _, session) = app();
    await tester.pumpWidget(widget);
    await session.restore();
    await tester.pumpAndSettle();
    expect(find.byType(FilledButton), findsOneWidget);
  });

  testWidgets('dark mode follows the platform', (tester) async {
    final (widget, _, session) = app();
    tester.platformDispatcher.platformBrightnessTestValue = Brightness.dark;
    await tester.pumpWidget(widget);
    await session.restore();
    await tester.pumpAndSettle();
    expect(
      Theme.of(tester.element(find.byType(Scaffold))).brightness,
      Brightness.dark,
    );
  });
}
