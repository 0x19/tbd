import 'package:flutter_test/flutter_test.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_testing/tbd_testing.dart';

void main() {
  test('tokens round-trip through the store format', () {
    final t = Tokens(
      accessToken: 'a',
      refreshToken: 'r',
      idToken: 'i',
      expiresAt: DateTime.utc(2026, 9, 20, 12),
    );
    final back = Tokens.fromJson(t.toJson());
    expect(back.accessToken, 'a');
    expect(back.refreshToken, 'r');
    expect(back.idToken, 'i');
    expect(back.expiresAt, t.expiresAt);
  });

  test('a refresh without an ID token keeps the old one', () {
    final first = Tokens(
      accessToken: 'a1',
      refreshToken: 'r1',
      idToken: 'id',
      expiresAt: DateTime.utc(2026),
    );
    final next = Tokens(accessToken: 'a2', expiresAt: DateTime.utc(2027));
    final merged = first.merge(next);
    expect(merged.accessToken, 'a2');
    expect(merged.refreshToken, 'r1');
    expect(merged.idToken, 'id');
  });

  test('a principal greets by name, then email, then subject', () {
    Principal of(Map<String, Object?> claims) => Principal.fromTokens(
      Tokens(accessToken: fakeJwt(claims), expiresAt: DateTime.utc(2027)),
    );
    expect(of({'sub': 's', 'email': 'e', 'name': 'n'}).displayName, 'n');
    expect(of({'sub': 's', 'email': 'e'}).displayName, 'e');
    expect(of({'sub': 's'}).displayName, 's');
    expect(jwtClaims('not a jwt'), isEmpty);
  });
}
