import 'dart:convert';

import 'package:meta/meta.dart';

/// What a sign-in or a refresh hands back, and what the secure store keeps.
@immutable
class Tokens {
  /// [expiresAt] is when the access token stops working, UTC.
  const Tokens({
    required this.accessToken,
    required this.expiresAt,
    this.refreshToken,
    this.idToken,
  });

  /// From the store's JSON.
  factory Tokens.fromJson(Map<String, Object?> json) => Tokens(
    accessToken: json['access_token']! as String,
    refreshToken: json['refresh_token'] as String?,
    idToken: json['id_token'] as String?,
    expiresAt: DateTime.parse(json['expires_at']! as String),
  );

  /// The bearer the edge verifies.
  final String accessToken;

  /// Mints the next access token; null for a session that cannot refresh
  /// (a machine token handed to the fake).
  final String? refreshToken;

  /// Who signed in, as Hydra asserts it; null for a machine token.
  final String? idToken;

  /// When [accessToken] stops working.
  final DateTime expiresAt;

  /// Whether the access token is good for at least [margin] more.
  bool fresh(DateTime now, {Duration margin = const Duration(seconds: 60)}) =>
      expiresAt.isAfter(now.add(margin));

  /// A refresh keeps the ID token when the response carries none.
  Tokens merge(Tokens next) => Tokens(
    accessToken: next.accessToken,
    refreshToken: next.refreshToken ?? refreshToken,
    idToken: next.idToken ?? idToken,
    expiresAt: next.expiresAt,
  );

  /// For the store. The names are OAuth's, so a reader recognises them.
  Map<String, Object?> toJson() => {
    'access_token': accessToken,
    'refresh_token': refreshToken,
    'id_token': idToken,
    'expires_at': expiresAt.toUtc().toIso8601String(),
  };
}

/// Who is signed in, read from the ID token's claims (or the access token's
/// `sub` for a machine). The claims are not verified here: they arrived over
/// TLS from the issuer the app was built for, and the edge verifies the
/// access token on every call; this is for greeting, not for authorising.
@immutable
class Principal {
  /// See [Principal.fromTokens].
  const Principal({required this.subject, this.email, this.name});

  /// From the ID token when there is one, else the access token's `sub`.
  factory Principal.fromTokens(Tokens t) {
    final claims = jwtClaims(t.idToken ?? t.accessToken);
    return Principal(
      subject: claims['sub'] as String? ?? '',
      email: claims['email'] as String?,
      name: claims['name'] as String?,
    );
  }

  /// The `sub` claim.
  final String subject;

  /// The `email` claim, when the `email` scope was granted.
  final String? email;

  /// The `name` claim, when the `profile` scope was granted and Kratos has one.
  final String? name;

  /// What to greet by: the name, else the email, else the subject.
  String get displayName => name ?? email ?? subject;
}

/// The payload of a JWT, unverified. Empty for anything that is not one.
Map<String, Object?> jwtClaims(String jwt) {
  final parts = jwt.split('.');
  if (parts.length != 3) return const {};
  try {
    final json = utf8.decode(base64Url.decode(base64Url.normalize(parts[1])));
    return (jsonDecode(json) as Map<String, Object?>?) ?? const {};
  } on FormatException {
    return const {};
  }
}
