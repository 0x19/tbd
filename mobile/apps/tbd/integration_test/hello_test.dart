// The ecosystem, end to end, on a real edge: sign in through the fake with a
// machine token (mise run auth:token), reach /v1/me over REST and Ping over
// gRPC through Envoy, be greeted, sign out.
//
//   mise run mobile:e2e local     (MOBILE_TOKEN in the environment)
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_mobile/app/app.dart';
import 'package:tbd_mobile/app/env.dart';
import 'package:tbd_mobile/app/providers.dart';
import 'package:tbd_mobile/app/router.dart';
import 'package:tbd_testing/tbd_testing.dart';

const token = String.fromEnvironment('MOBILE_TOKEN');

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets(
    'sign in with a machine token, hello over REST and gRPC, sign out',
    (tester) async {
      final env = readEnv();
      final repo = FakeAuthRepository(token: token, refreshToken: null);
      final session = Session(repository: repo, store: MemoryTokenStore());
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            envProvider.overrideWithValue(env),
            sessionProvider.overrideWithValue(session),
          ],
          child: App(router: buildRouter(session)),
        ),
      );
      await session.restore();
      await tester.pumpAndSettle();
      await tester.tap(find.byType(FilledButton));
      await tester.pumpAndSettle(const Duration(seconds: 1));
      expect(find.textContaining('Hello,'), findsOneWidget);

      // Both cards answered: a subject from /v1/me and the ping's echo.
      await tester.pumpAndSettle(const Duration(seconds: 2));
      expect(find.text('hello from the phone'), findsOneWidget);
      expect(find.textContaining('Role:'), findsOneWidget);
      expect(find.textContaining('could not be reached'), findsNothing);

      await tester.tap(find.byIcon(Icons.settings_outlined));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Sign out'));
      await tester.pumpAndSettle();
      expect(find.byType(FilledButton), findsOneWidget);
    },
    skip: token.isEmpty,
  );
}
