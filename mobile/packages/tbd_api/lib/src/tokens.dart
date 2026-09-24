/// Where the client gets its bearer from. `tbd_auth`'s session implements
/// it; a test hands in a fixed token. The client never stores a token
/// itself, so there is exactly one place tokens live.
abstract interface class TokenSource {
  /// The access token to send, or null when there is no session.
  Future<String?> accessToken();

  /// Get a new access token after the edge refused the current one. Null
  /// means the refresh was refused too: the session is over. Called once
  /// per refused token however many calls were in flight (the client
  /// serialises it).
  Future<String?> refresh();
}

/// A token that never changes: tests, and the integration test's machine
/// token.
final class StaticTokenSource implements TokenSource {
  /// [token] is sent on every call; a refresh hands back the same token.
  const StaticTokenSource(this.token);

  /// The bearer.
  final String? token;

  @override
  Future<String?> accessToken() async => token;

  @override
  Future<String?> refresh() async => token;
}
