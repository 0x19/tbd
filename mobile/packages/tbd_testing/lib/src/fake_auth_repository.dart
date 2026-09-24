import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_core/tbd_core.dart';

/// Signs in with the token it was given: unit tests, and the integration
/// test against a real edge with a machine token from `mise run auth:token`
/// (no browser on a CI runner). Every failure mode is a knob.
final class FakeAuthRepository implements AuthRepository {
  /// [token] is what a sign-in hands back; [idToken] makes it a person.
  FakeAuthRepository({
    required this.token,
    this.idToken,
    this.refreshToken = 'refresh-1',
    this.lifetime = const Duration(hours: 1),
    Clock? clock,
  }) : _clock = clock ?? const SystemClock();

  /// The access token a sign-in and a refresh mint (a counter is appended
  /// on refresh so a test can tell them apart).
  String token;

  /// The ID token, when the fake plays a person.
  String? idToken;

  /// The refresh token; null makes the session unrefreshable.
  String? refreshToken;

  /// How long an access token lives.
  Duration lifetime;
  final Clock _clock;

  /// The next sign-in is cancelled by the person.
  bool cancelNext = false;

  /// The next refresh is refused by the issuer.
  bool refuseRefresh = false;

  /// The next call fails with the network.
  bool offline = false;

  /// How many refreshes were asked.
  int refreshes = 0;

  /// How many sign-ins were asked.
  int signIns = 0;

  /// How many sessions were ended at the issuer.
  int ended = 0;

  @override
  Future<Result<Tokens>> signIn() async {
    signIns++;
    if (cancelNext) {
      cancelNext = false;
      return const Err(CancelledError('cancelled'));
    }
    if (offline) return const Err(NetworkError('offline'));
    return Ok(_mint(token));
  }

  @override
  Future<Result<Tokens>> refresh(String refreshToken) async {
    refreshes++;
    if (offline) return const Err(NetworkError('offline'));
    if (refuseRefresh || refreshToken != this.refreshToken) {
      return const Err(UnauthenticatedError('invalid_grant'));
    }
    return Ok(_mint('$token-r$refreshes'));
  }

  @override
  Future<Result<void>> endSession(Tokens tokens) async {
    ended++;
    return const Ok(null);
  }

  Tokens _mint(String access) => Tokens(
    accessToken: access,
    refreshToken: refreshToken,
    idToken: idToken,
    expiresAt: _clock.now().add(lifetime),
  );
}
