import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tbd_api/tbd_api.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_core/tbd_core.dart';

/// The dependency graph. Two roots are supplied at bootstrap (`main.dart`
/// overrides them with the real thing, a test with fakes); everything else
/// is derived here, once, and read by the view models.

/// The build's configuration. Overridden at the root; unusable otherwise.
final envProvider = Provider<Env>(
  (_) => throw StateError('envProvider is overridden at the root'),
);

/// The session: the one owner of tokens. Overridden at the root.
final sessionProvider = Provider<Session>(
  (_) => throw StateError('sessionProvider is overridden at the root'),
);

/// The session's state as a value widgets can watch; follows the session's
/// notifications.
final sessionStateProvider =
    NotifierProvider<SessionStateNotifier, SessionState>(
      SessionStateNotifier.new,
    );

/// See [sessionStateProvider].
class SessionStateNotifier extends Notifier<SessionState> {
  @override
  SessionState build() {
    final session = ref.watch(sessionProvider);
    void sync() => state = session.state;
    session.addListener(sync);
    ref.onDispose(() => session.removeListener(sync));
    return session.state;
  }
}

/// The session as the client's bearer source.
final tokenSourceProvider = Provider<TokenSource>(
  (ref) => _SessionTokens(ref.watch(sessionProvider)),
);

final class _SessionTokens implements TokenSource {
  _SessionTokens(this.session);
  final Session session;

  @override
  Future<String?> accessToken() => session.accessToken();

  @override
  Future<String?> refresh() => session.refresh();
}

/// The REST client for the edge.
final apiClientProvider = Provider<ApiClient>(
  (ref) =>
      ApiClient(ref.watch(envProvider), tokens: ref.watch(tokenSourceProvider)),
);

/// The gRPC edge.
final grpcEdgeProvider = Provider<GrpcEdge>((ref) {
  final edge = GrpcEdge(
    ref.watch(envProvider),
    tokens: ref.watch(tokenSourceProvider),
  );
  ref.onDispose(edge.shutdown);
  return edge;
});

/// The protocol's own calls.
final protocolApiProvider = Provider<ProtocolApi>(
  (ref) => EdgeProtocolApi(
    ref.watch(apiClientProvider),
    ref.watch(grpcEdgeProvider),
  ),
);
