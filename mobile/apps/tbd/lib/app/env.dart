import 'package:flutter/foundation.dart';
import 'package:tbd_core/tbd_core.dart';

/// The build's configuration, read from the compile-time defines that
/// `--dart-define-from-file=build/env.<env>.json` provides. This is the only
/// file that names a define; every other file asks [Env].
///
/// `String.fromEnvironment` must be given a literal key, so the keys are
/// listed here rather than looked up: `mise run mobile:config` shows the same
/// set from the JSON side.
Env readEnv() => Env.fromDefines(
  (key) => switch (key) {
    'env' => const String.fromEnvironment('env'),
    'auth.issuer' => const String.fromEnvironment('auth.issuer'),
    'auth.client_id' => const String.fromEnvironment('auth.client_id'),
    'auth.redirect_uri' => const String.fromEnvironment('auth.redirect_uri'),
    'auth.post_logout_redirect_uri' => const String.fromEnvironment(
      'auth.post_logout_redirect_uri',
    ),
    'auth.scopes' => const String.fromEnvironment('auth.scopes'),
    'auth.audience' => const String.fromEnvironment('auth.audience'),
    'api.base_url' => const String.fromEnvironment('api.base_url'),
    'api.grpc_authority' => const String.fromEnvironment('api.grpc_authority'),
    'api.timeout_ms' => const String.fromEnvironment('api.timeout_ms'),
    'telemetry.otlp_endpoint' => const String.fromEnvironment(
      'telemetry.otlp_endpoint',
    ),
    'telemetry.service_name' => const String.fromEnvironment(
      'telemetry.service_name',
    ),
    _ => '',
  },
);

/// Whether the build carries a configuration at all; a `flutter test` does not.
bool get hasEnv => const String.fromEnvironment('env').isNotEmpty;

/// A test build without defines still has to show something: the placeholder
/// says so on the screen instead of pretending.
const bool isDebugBuild = kDebugMode;
