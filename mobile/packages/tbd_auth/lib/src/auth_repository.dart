import 'package:flutter_appauth/flutter_appauth.dart';
import 'package:tbd_auth/src/tokens.dart';
import 'package:tbd_core/tbd_core.dart';

/// The identity provider, behind one interface: sign in, refresh, end the
/// session. [AppAuthRepository] is the real one; `FakeAuthRepository` in
/// `tbd_testing` signs in with a token it was given.
abstract interface class AuthRepository {
  /// Authorization code with PKCE in the system browser; returns when the
  /// browser comes back on the redirect and the code is exchanged.
  Future<Result<Tokens>> signIn();

  /// A new access token from [refreshToken]. [UnauthenticatedError] when
  /// the token endpoint refuses: the session is over.
  Future<Result<Tokens>> refresh(String refreshToken);

  /// RP-initiated logout at the issuer, so the browser session ends too.
  /// Best effort: the local session is gone whatever this returns.
  Future<Result<void>> endSession(Tokens tokens);
}

/// AppAuth (the AppAuth-iOS and AppAuth-Android libraries) against Hydra's
/// discovery document. Never an embedded web view: iOS uses
/// `ASWebAuthenticationSession`, Android a Custom Tab.
final class AppAuthRepository implements AuthRepository {
  /// [appAuth] is the plugin; a test passes its own.
  AppAuthRepository(this.env, {FlutterAppAuth? appAuth, Clock? clock})
    : _appAuth = appAuth ?? const FlutterAppAuth(),
      _clock = clock ?? const SystemClock();

  /// The build's configuration: issuer, client, redirect, scopes, audience.
  final Env env;
  final FlutterAppAuth _appAuth;
  final Clock _clock;

  @override
  Future<Result<Tokens>> signIn() => _guard(() async {
    final r = await _appAuth.authorizeAndExchangeCode(
      AuthorizationTokenRequest(
        env.authClientId,
        env.authRedirectUri,
        discoveryUrl: env.authDiscoveryUrl.toString(),
        scopes: env.authScopes,
        // Hydra mints the API audience only when the request names it.
        additionalParameters: {'audience': env.authAudience},
        // Every sign-in is a fresh one: no silent reuse of a browser session
        // that belongs to someone else on a shared device.
        promptValues: const ['login'],
      ),
    );
    return _tokens(r);
  });

  @override
  Future<Result<Tokens>> refresh(String refreshToken) => _guard(() async {
    final r = await _appAuth.token(
      TokenRequest(
        env.authClientId,
        env.authRedirectUri,
        discoveryUrl: env.authDiscoveryUrl.toString(),
        refreshToken: refreshToken,
        scopes: env.authScopes,
        additionalParameters: {'audience': env.authAudience},
      ),
    );
    return _tokens(r);
  });

  @override
  Future<Result<void>> endSession(Tokens tokens) => _guard(() async {
    await _appAuth.endSession(
      EndSessionRequest(
        idTokenHint: tokens.idToken,
        postLogoutRedirectUrl: env.authPostLogoutRedirectUri,
        discoveryUrl: env.authDiscoveryUrl.toString(),
      ),
    );
  });

  Tokens _tokens(TokenResponse r) {
    final access = r.accessToken;
    if (access == null || access.isEmpty) {
      throw const UnauthenticatedError('token response without access token');
    }
    return Tokens(
      accessToken: access,
      refreshToken: r.refreshToken,
      idToken: r.idToken,
      expiresAt:
          r.accessTokenExpirationDateTime?.toUtc() ??
          _clock.now().add(const Duration(minutes: 5)),
    );
  }

  /// The plugin's exceptions as the closed set: the person closing the
  /// browser is a cancel; an `invalid_grant` on refresh is the end of the
  /// session; anything else is the network or the provider.
  static Future<Result<T>> _guard<T>(Future<T> Function() body) async {
    try {
      return Ok(await body());
    } on FlutterAppAuthUserCancelledException {
      return const Err(CancelledError('sign-in cancelled'));
    } on FlutterAppAuthPlatformException catch (e) {
      final details = e.platformErrorDetails;
      final oauth = details.error ?? '';
      if (oauth == 'invalid_grant' || oauth == 'invalid_client') {
        return Err(UnauthenticatedError(details.errorDescription ?? oauth));
      }
      return Err(
        NetworkError(details.errorDescription ?? e.message ?? oauth, cause: e),
      );
    } on AppError catch (e) {
      return Err(e);
    }
  }
}
