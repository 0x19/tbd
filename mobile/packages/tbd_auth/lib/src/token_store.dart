import 'dart:convert';

import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:tbd_auth/src/tokens.dart';

/// Where the session's tokens rest. One implementation talks to the
/// platform's secure store; the other is memory, for tests.
abstract interface class TokenStore {
  /// The stored tokens, or null when signed out.
  Future<Tokens?> read();

  /// Replace what is stored.
  Future<void> write(Tokens tokens);

  /// Sign out at rest.
  Future<void> clear();
}

/// Keychain on iOS (this device only, first unlock), Keystore-backed
/// encrypted preferences on Android. One key, one JSON value.
final class SecureTokenStore implements TokenStore {
  /// [storage] is the plugin; a test passes its own.
  SecureTokenStore({FlutterSecureStorage? storage})
    : _storage =
          storage ??
          const FlutterSecureStorage(
            iOptions: IOSOptions(
              accessibility: KeychainAccessibility.first_unlock_this_device,
            ),
          );

  final FlutterSecureStorage _storage;

  /// The one key.
  static const key = 'tbd.session';

  @override
  Future<Tokens?> read() async {
    final raw = await _storage.read(key: key);
    if (raw == null || raw.isEmpty) return null;
    try {
      return Tokens.fromJson(jsonDecode(raw) as Map<String, Object?>);
    } on Object {
      // A value this version cannot read is a sign-out, not a crash loop.
      await _storage.delete(key: key);
      return null;
    }
  }

  @override
  Future<void> write(Tokens tokens) =>
      _storage.write(key: key, value: jsonEncode(tokens.toJson()));

  @override
  Future<void> clear() => _storage.delete(key: key);
}

/// Tokens in memory: tests, and nothing else.
final class MemoryTokenStore implements TokenStore {
  /// Optionally pre-filled, as if a previous run had signed in.
  MemoryTokenStore([this._tokens]);

  Tokens? _tokens;

  /// How many writes happened, for a test to count.
  int writes = 0;

  @override
  Future<Tokens?> read() async => _tokens;

  @override
  Future<void> write(Tokens tokens) async {
    writes++;
    _tokens = tokens;
  }

  @override
  Future<void> clear() async => _tokens = null;
}
