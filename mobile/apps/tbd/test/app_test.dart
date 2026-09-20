import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tbd_mobile/app/app.dart';
import 'package:tbd_mobile/main.dart';

void main() {
  testWidgets('the shell shows the placeholder in both languages', (
    tester,
  ) async {
    await tester.pumpWidget(const App(home: Foundation(envName: 'local')));
    expect(find.text('Hello'), findsOneWidget);
    expect(find.text('local'), findsOneWidget);

    await tester.binding.setLocale('hr', '');
    await tester.pumpAndSettle();
    expect(find.text('Pozdrav'), findsOneWidget);
  });

  testWidgets('a build without configuration says so instead of pretending', (
    tester,
  ) async {
    await tester.pumpWidget(const App(home: Foundation(envName: null)));
    expect(find.textContaining('without its configuration'), findsOneWidget);
  });

  testWidgets('dark mode follows the platform', (tester) async {
    tester.platformDispatcher.platformBrightnessTestValue = Brightness.dark;
    await tester.pumpWidget(const App(home: Foundation(envName: 'local')));
    final scaffold = tester.widget<Scaffold>(find.byType(Scaffold));
    expect(
      Theme.of(tester.element(find.byType(Scaffold))).brightness,
      Brightness.dark,
    );
    expect(scaffold, isNotNull);
  });
}
