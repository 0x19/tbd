import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:tbd_ui/tbd_ui.dart';

void main() {
  test('both themes build from the tokens and carry the extension', () {
    final light = TbdTheme.light();
    final dark = TbdTheme.dark();
    expect(light.extension<TbdColors>()!.t, same(lightTokens));
    expect(dark.extension<TbdColors>()!.t, same(darkTokens));
    expect(light.colorScheme.primary, lightTokens.primary);
    expect(dark.scaffoldBackgroundColor, darkTokens.background);
    expect(light.textTheme.bodyMedium?.fontFamily, TbdTheme.fontFamily);
  });

  test('the tokens are the web sheet: neutral scale, blue chart ramp', () {
    // Tailwind neutral-900 / neutral-200 and blue-500, what shadcn "neutral" resolves to.
    expect(lightTokens.primary, const Color(0xFF171717));
    expect(lightTokens.border, const Color(0xFFE5E5E5));
    expect(lightTokens.chart2, const Color(0xFF2B7FFF));
    expect(darkTokens.background, const Color(0xFF0A0A0A));
    // `.dark` borders are white at 10%: alpha survives the conversion.
    expect(darkTokens.border.a, closeTo(0.1, 0.01));
    expect(tokenRadius, 10);
    expect(tokenNames, contains('success'));
  });

  testWidgets('TbdColors.of follows the theme in use', (tester) async {
    late TbdColors seen;
    await tester.pumpWidget(
      MaterialApp(
        theme: TbdTheme.dark(),
        home: Builder(
          builder: (context) {
            seen = TbdColors.of(context);
            return const SizedBox();
          },
        ),
      ),
    );
    expect(seen.background, darkTokens.background);
    expect(seen.chart, hasLength(5));
  });
}
