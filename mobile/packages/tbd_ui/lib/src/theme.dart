import 'package:flutter/material.dart';
import 'package:tbd_ui/src/tokens.g.dart';

/// The kit's palette under the kit's names, reachable from any widget as
/// `TbdColors.of(context)`. Material's `ColorScheme` gets the tokens it has
/// words for; everything else (`muted`, the chart ramp, `success`, the
/// priorities) lives here, so a widget never bends a Material role to mean
/// something the web kit says differently.
@immutable
class TbdColors extends ThemeExtension<TbdColors> {
  /// Wrap one generated scheme.
  const TbdColors(this.t);

  /// The generated tokens themselves, for a token without a getter here.
  final TokenColors t;

  /// The scheme of the theme in use.
  static TbdColors of(BuildContext context) =>
      Theme.of(context).extension<TbdColors>() ??
      (Theme.of(context).brightness == Brightness.dark ? dark : light);

  /// `:root`.
  static const light = TbdColors(lightTokens);

  /// `.dark`.
  static const dark = TbdColors(darkTokens);

  /// The page.
  Color get background => t.background;

  /// Text on the page.
  Color get foreground => t.foreground;

  /// A card or panel.
  Color get card => t.card;

  /// Text on a card.
  Color get cardForeground => t.cardForeground;

  /// A quiet surface: a hint row, a track.
  Color get muted => t.muted;

  /// Secondary text.
  Color get mutedForeground => t.mutedForeground;

  /// A hovered or selected row.
  Color get accent => t.accent;

  /// Text on an accent.
  Color get accentForeground => t.accentForeground;

  /// Hairlines and outlines.
  Color get border => t.border;

  /// An input's border.
  Color get input => t.input;

  /// The focus ring.
  Color get ring => t.ring;

  /// Danger and errors.
  Color get destructive => t.destructive;

  /// Positive states.
  Color get success => t.success;

  /// Attention.
  Color get warning => t.warning;

  /// The chart ramp, `--chart-1` to `--chart-5`.
  List<Color> get chart => [t.chart1, t.chart2, t.chart3, t.chart4, t.chart5];

  /// `--priority-1` to `--priority-4`, urgent first.
  List<Color> get priority => [
    t.priority1,
    t.priority2,
    t.priority3,
    t.priority4,
  ];

  @override
  TbdColors copyWith({TokenColors? t}) => TbdColors(t ?? this.t);

  /// Tokens are discrete schemes; a theme change snaps rather than blends.
  @override
  TbdColors lerp(ThemeExtension<TbdColors>? other, double t) =>
      t < 0.5 || other is! TbdColors ? this : other;
}

/// Material 3 themes built from the tokens: `TbdTheme.light()` and
/// `TbdTheme.dark()`, used with `ThemeMode.system` like the web's default.
abstract final class TbdTheme {
  /// The text face, bundled.
  static const fontFamily = 'Inter';

  /// The mono face, bundled.
  static const monoFamily = 'Geist Mono';

  /// The light theme.
  static ThemeData light() => _build(lightTokens, Brightness.light);

  /// The dark theme.
  static ThemeData dark() => _build(darkTokens, Brightness.dark);

  static ThemeData _build(TokenColors t, Brightness brightness) {
    final scheme = ColorScheme(
      brightness: brightness,
      primary: t.primary,
      onPrimary: t.primaryForeground,
      secondary: t.secondary,
      onSecondary: t.secondaryForeground,
      error: t.destructive,
      onError: t.destructiveForeground,
      surface: t.background,
      onSurface: t.foreground,
      surfaceContainerLowest: t.background,
      surfaceContainerLow: t.card,
      surfaceContainer: t.card,
      surfaceContainerHigh: t.muted,
      surfaceContainerHighest: t.accent,
      onSurfaceVariant: t.mutedForeground,
      outline: t.border,
      outlineVariant: t.border,
      shadow: Colors.black,
      scrim: Colors.black,
      inverseSurface: t.foreground,
      onInverseSurface: t.background,
      inversePrimary: t.primaryForeground,
      surfaceTint: Colors.transparent,
    );
    final radius = BorderRadius.circular(tokenRadius);
    final base = ThemeData(
      useMaterial3: true,
      brightness: brightness,
      colorScheme: scheme,
      fontFamily: fontFamily,
      scaffoldBackgroundColor: t.background,
      canvasColor: t.background,
      dividerColor: t.border,
      splashFactory: InkSparkle.splashFactory,
      visualDensity: VisualDensity.standard,
      extensions: [TbdColors(t)],
    );
    return base.copyWith(
      appBarTheme: AppBarTheme(
        backgroundColor: t.background,
        foregroundColor: t.foreground,
        elevation: 0,
        scrolledUnderElevation: 0,
        centerTitle: false,
        titleTextStyle: base.textTheme.titleLarge?.copyWith(
          color: t.foreground,
          fontWeight: FontWeight.w600,
        ),
      ),
      cardTheme: CardThemeData(
        color: t.card,
        elevation: 0,
        margin: EdgeInsets.zero,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(tokenRadiusXl),
          side: BorderSide(color: t.border),
        ),
      ),
      dividerTheme: DividerThemeData(color: t.border, thickness: 1, space: 1),
      filledButtonTheme: FilledButtonThemeData(
        style: FilledButton.styleFrom(
          backgroundColor: t.primary,
          foregroundColor: t.primaryForeground,
          shape: RoundedRectangleBorder(borderRadius: radius),
          minimumSize: const Size(0, 44),
          textStyle: const TextStyle(fontWeight: FontWeight.w500),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: t.foreground,
          side: BorderSide(color: t.border),
          shape: RoundedRectangleBorder(borderRadius: radius),
          minimumSize: const Size(0, 44),
        ),
      ),
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          foregroundColor: t.foreground,
          shape: RoundedRectangleBorder(borderRadius: radius),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: t.background,
        border: OutlineInputBorder(
          borderRadius: radius,
          borderSide: BorderSide(color: t.input),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: radius,
          borderSide: BorderSide(color: t.input),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: radius,
          borderSide: BorderSide(color: t.ring),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: radius,
          borderSide: BorderSide(color: t.destructive),
        ),
        hintStyle: TextStyle(color: t.mutedForeground),
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: t.foreground,
        contentTextStyle: TextStyle(
          color: t.background,
          fontFamily: fontFamily,
        ),
        shape: RoundedRectangleBorder(borderRadius: radius),
        behavior: SnackBarBehavior.floating,
      ),
      listTileTheme: ListTileThemeData(
        iconColor: t.mutedForeground,
        textColor: t.foreground,
        shape: RoundedRectangleBorder(borderRadius: radius),
      ),
      progressIndicatorTheme: ProgressIndicatorThemeData(
        color: t.foreground,
        linearTrackColor: t.muted,
      ),
    );
  }
}

/// A line of ids, hashes or numbers in the kit's mono face, tabular.
TextStyle mono(BuildContext context, {double? size, Color? color}) => TextStyle(
  fontFamily: TbdTheme.monoFamily,
  fontFeatures: const [FontFeature.tabularFigures()],
  fontSize: size,
  color: color ?? TbdColors.of(context).foreground,
);
