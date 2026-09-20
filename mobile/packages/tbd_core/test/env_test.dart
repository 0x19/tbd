import 'package:tbd_core/tbd_core.dart';
import 'package:test/test.dart';

void main() {
  const defines = {
    'env': 'local',
    'auth.issuer': 'https://auth.example.test',
    'auth.client_id': 'tbd-app',
    'auth.redirect_uri': 'tbd://callback',
    'auth.post_logout_redirect_uri': 'tbd://signed-out',
    'auth.scopes': 'openid offline_access email profile tbd.api',
    'auth.audience': 'tbd-api',
    'api.base_url': 'https://api.example.test',
    'api.grpc_authority': 'api.example.test:443',
    'api.timeout_ms': '10000',
    'telemetry.otlp_endpoint': '',
    'telemetry.service_name': 'tbd-mobile',
  };

  test('every key the build carries is read, '
      'and the discovery url follows the issuer', () {
    final env = Env.fromDefines((k) => defines[k] ?? '');
    expect(env.name, 'local');
    expect(env.authScopes, [
      'openid',
      'offline_access',
      'email',
      'profile',
      'tbd.api',
    ]);
    expect(
      env.authDiscoveryUrl.toString(),
      'https://auth.example.test/.well-known/openid-configuration',
    );
    expect(env.apiTimeout, const Duration(seconds: 10));
    expect(
      env.otlpEndpoint,
      isEmpty,
      reason: 'no exporter until the collector has a public door',
    );
    expect(env.isProduction, isFalse);
  });

  test('a missing key fails at start, naming the key and the fix', () {
    final without = Map.of(defines)..remove('api.base_url');
    expect(
      () => Env.fromDefines((k) => without[k] ?? ''),
      throwsA(
        isA<StateError>().having(
          (e) => e.message,
          'message',
          contains('api.base_url'),
        ),
      ),
    );
  });

  test('a result carries a value or an error, never both', () async {
    final ok = await Result.guard(() async => 1);
    final err = await Result.guard<int>(
      () async => throw const NetworkError('offline'),
    );
    expect(ok.when(ok: (v) => v, err: (_) => -1), 1);
    expect(err.when(ok: (v) => v, err: (e) => e is NetworkError ? -1 : -2), -1);
    expect(ok.map((v) => v + 1).isOk, isTrue);
  });

  test('a traceparent is well formed', () {
    final ctx = RandomTracer().next();
    expect(
      ctx.traceparent,
      matches(RegExp(r'^00-[0-9a-f]{32}-[0-9a-f]{16}-01$')),
    );
  });
}
