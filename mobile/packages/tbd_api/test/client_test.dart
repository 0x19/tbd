import 'dart:async';

import 'package:dio/dio.dart';
import 'package:tbd_api/tbd_api.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:test/test.dart';

import 'fake_adapter.dart';

final env = Env.fromDefines(
  (k) =>
      {
        'env': 'test',
        'auth.issuer': 'https://auth.test',
        'auth.client_id': 'tbd-app',
        'auth.redirect_uri': 'tbd://callback',
        'auth.post_logout_redirect_uri': 'tbd://callback/signed-out',
        'auth.scopes': 'openid tbd.api',
        'auth.audience': 'tbd-api',
        'api.base_url': 'https://api.test',
        'api.grpc_authority': 'api.test:443',
        'api.timeout_ms': '1000',
        'telemetry.service_name': 'tbd-mobile-test',
      }[k] ??
      '',
);

/// A source whose refresh is counted and can be told to refuse.
final class CountingTokens implements TokenSource {
  CountingTokens({this.refuse = false});
  bool refuse;
  String token = 't1';
  int refreshes = 0;
  final _gate = Completer<void>();

  /// Let a pending refresh finish.
  void release() => _gate.complete();

  @override
  Future<String?> accessToken() async => token;

  @override
  Future<String?> refresh() async {
    refreshes++;
    if (!_gate.isCompleted) await _gate.future;
    if (refuse) return null;
    return token = 't${refreshes + 1}';
  }
}

ApiClient client(
  FakeAdapter adapter,
  TokenSource tokens, {
  void Function()? ended,
}) => ApiClient(
  env,
  tokens: tokens,
  adapter: adapter,
  onSessionEnded: ended,
  retryDelay: (_) async {},
);

void main() {
  test('the bearer and a traceparent ride on every call', () async {
    final adapter = FakeAdapter((o, _) => const Reply.json(200, {'ok': true}));
    final r = await client(
      adapter,
      const StaticTokenSource('abc'),
    ).getJson('/v1/me');
    expect(r.isOk, isTrue);
    final h = adapter.seen.single.headers;
    expect(h['authorization'], 'Bearer abc');
    expect(
      h['traceparent'],
      matches(RegExp(r'^00-[0-9a-f]{32}-[0-9a-f]{16}-01$')),
    );
    expect(adapter.seen.single.uri.toString(), 'https://api.test/v1/me');
  });

  test('a 401 refreshes once and retries with the new token', () async {
    final tokens = CountingTokens()..release();
    final adapter = FakeAdapter(
      (o, n) => o.headers['authorization'] == 'Bearer t1'
          ? const Reply.json(401, {
              'code': 'unauthenticated',
              'error': 'expired',
            })
          : const Reply.json(200, {'subject': 'me'}),
    );
    final r = await client(adapter, tokens).getJson('/v1/me');
    expect(r.when(ok: (v) => v['subject'], err: (e) => e), 'me');
    expect(tokens.refreshes, 1);
    expect(adapter.seen, hasLength(2));
    expect(adapter.seen.last.headers['authorization'], 'Bearer t2');
  });

  test('ten calls refused at once cause one refresh', () async {
    final tokens = CountingTokens();
    final adapter = FakeAdapter(
      (o, n) => o.headers['authorization'] == 'Bearer t1'
          ? const Reply.json(401, {
              'code': 'unauthenticated',
              'error': 'expired',
            })
          : Reply.json(200, {'n': n}),
    );
    final api = client(adapter, tokens);
    final calls = List.generate(10, (_) => api.getJson('/v1/me'));
    // Every call has hit the 401 and is waiting on the one refresh.
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);
    tokens.release();
    final results = await Future.wait(calls);
    expect(results.every((r) => r.isOk), isTrue);
    expect(tokens.refreshes, 1);
    expect(adapter.seen, hasLength(20));
  });

  test('a refused refresh ends the session', () async {
    final tokens = CountingTokens(refuse: true)..release();
    var ended = 0;
    final adapter = FakeAdapter(
      (o, n) =>
          const Reply.json(401, {'code': 'unauthenticated', 'error': 'no'}),
    );
    final r = await client(
      adapter,
      tokens,
      ended: () => ended++,
    ).getJson('/v1/me');
    expect(r.when(ok: (_) => null, err: (e) => e), isA<UnauthenticatedError>());
    expect(ended, 1);
    expect(adapter.seen, hasLength(1));
  });

  test('a problem body becomes a ServerError with its code', () async {
    final adapter = FakeAdapter(
      (o, n) => const Reply.json(422, {
        'code': 'invalid_argument',
        'error': 'message must not be empty',
        'details': <Object?>[],
      }),
    );
    final r = await client(
      adapter,
      const StaticTokenSource('t'),
    ).postJson('/v1/things', {'message': ''});
    final e = r.when(ok: (_) => null, err: (e) => e);
    expect(e, isA<ServerError>());
    expect((e! as ServerError).status, 422);
    expect((e as ServerError).code, 'invalid_argument');
    expect(e.message, 'message must not be empty');
  });

  test(
    '403 is ForbiddenError; a GET is retried on 503, a POST is not',
    () async {
      final adapter = FakeAdapter(
        (o, n) => o.method == 'GET' && n == 1
            ? const Reply(503, 'upstream')
            : o.method == 'GET'
            ? Reply.json(200, {'after': n})
            : const Reply.json(403, {'code': 'forbidden', 'error': 'no'}),
      );
      final api = client(adapter, const StaticTokenSource('t'));
      final get = await api.getJson('/v1/me');
      expect(get.when(ok: (v) => v['after'], err: (e) => e), 2);
      final post = await api.postJson('/v1/things', {});
      expect(post.when(ok: (_) => null, err: (e) => e), isA<ForbiddenError>());
      expect(adapter.seen.where((o) => o.method == 'POST'), hasLength(1));
    },
  );

  test('a connection failure is a NetworkError', () async {
    final adapter = FakeAdapter(
      (o, n) => throw DioException.connectionError(
        requestOptions: o,
        reason: 'refused',
      ),
    );
    final r = await client(
      adapter,
      const StaticTokenSource('t'),
    ).getJson('/v1/me');
    expect(r.when(ok: (_) => null, err: (e) => e), isA<NetworkError>());
    // Two retries after the first attempt.
    expect(adapter.seen, hasLength(3));
  });
}
