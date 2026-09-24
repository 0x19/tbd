import 'package:meta/meta.dart';

/// The build's configuration: `configs/mobile/base.json` merged with the
/// environment's file by `mise run mobile:env`, flattened to dotted keys and
/// passed as `--dart-define-from-file`.
///
/// Read once at start through [Env.fromDefines]; a missing key is a build
/// mistake and fails loudly there, not on the screen that first needs it.
/// Nothing in an app names a URL directly; it asks this.
@immutable
class Env {
  /// Every value the build carries; see [Env.fromDefines].
  const Env({
    required this.name,
    required this.authIssuer,
    required this.authClientId,
    required this.authRedirectUri,
    required this.authPostLogoutRedirectUri,
    required this.authScopes,
    required this.authAudience,
    required this.apiBaseUrl,
    required this.apiGrpcAuthority,
    required this.apiTimeout,
    required this.otlpEndpoint,
    required this.serviceName,
  });

  /// From the values the build was given. [read] exists so tests can hand in
  /// a map; the app passes the compile-time defines.
  factory Env.fromDefines(String Function(String key) read) {
    String need(String key) {
      final v = read(key);
      if (v.isEmpty) {
        throw StateError(
          'env: "$key" is not set; '
          'run mise run mobile:env <env> and build with it',
        );
      }
      return v;
    }

    return Env(
      name: need('env'),
      authIssuer: Uri.parse(need('auth.issuer')),
      authClientId: need('auth.client_id'),
      authRedirectUri: need('auth.redirect_uri'),
      authPostLogoutRedirectUri: need('auth.post_logout_redirect_uri'),
      authScopes: need(
        'auth.scopes',
      ).split(' ').where((s) => s.isNotEmpty).toList(growable: false),
      authAudience: need('auth.audience'),
      apiBaseUrl: Uri.parse(need('api.base_url')),
      apiGrpcAuthority: need('api.grpc_authority'),
      apiTimeout: Duration(milliseconds: int.parse(need('api.timeout_ms'))),
      // Empty is allowed: no exporter until the collector has a public door.
      otlpEndpoint: read('telemetry.otlp_endpoint'),
      serviceName: need('telemetry.service_name'),
    );
  }

  /// `local`, `prod`: the file that was merged over base.
  final String name;

  /// Hydra's public URL; the `iss` every token carries.
  final Uri authIssuer;

  /// The public OAuth client registered for apps (`tbd-app`).
  final String authClientId;

  /// Where the browser returns after sign-in (`tbd://callback`).
  final String authRedirectUri;

  /// Where the browser returns after sign-out.
  final String authPostLogoutRedirectUri;

  /// The scopes asked for, as registered on the client.
  final List<String> authScopes;

  /// The audience the access token must carry to pass the edge.
  final String authAudience;

  /// The REST edge: `api.<domain>`.
  final Uri apiBaseUrl;

  /// `host:port` for the gRPC channel; the same edge as [apiBaseUrl].
  final String apiGrpcAuthority;

  /// Per request, connect and receive.
  final Duration apiTimeout;

  /// Empty until traces can leave the device.
  final String otlpEndpoint;

  /// `service.name` on every span.
  final String serviceName;

  /// Discovery document, derived: never configured separately, so it cannot
  /// disagree with the issuer.
  Uri get authDiscoveryUrl =>
      authIssuer.replace(path: '/.well-known/openid-configuration');

  /// The `prod` environment.
  bool get isProduction => name == 'prod';
}
