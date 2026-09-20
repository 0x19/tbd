import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:tbd_auth/src/auth_repository.dart';
import 'package:tbd_auth/src/token_store.dart';
import 'package:tbd_auth/src/tokens.dart';
import 'package:tbd_core/tbd_core.dart';

/// The session as a closed set of states the router and the views switch on.
@immutable
sealed class SessionState {
  const SessionState();
}

/// Before the store has been read.
final class SessionUnknown extends SessionState {
  /// See [SessionState].
  const SessionUnknown();
}

/// No session. [error] says why the last attempt failed, when one did.
final class SignedOut extends SessionState {
  /// See [SessionState].
  const SignedOut({this.error});

  /// What ended or refused the last sign-in, for the view to explain.
  final AppError? error;
}

/// The browser is open.
final class SigningIn extends SessionState {
  /// See [SessionState].
  const SigningIn();
}

/// A session with tokens in the store.
final class SignedIn extends SessionState {
  /// See [SessionState].
  const SignedIn(this.principal);

  /// Who, from the ID token.
  final Principal principal;
}

/// The one owner of the session: it drives the repository, keeps the store
/// current, hands the client its bearer, and tells everyone when the state
/// changes. Implements the client's token source directly so there is no
/// second copy of a token anywhere.
///
/// The token-source side: `accessToken()` refreshes ahead of expiry, and
/// `refresh()` (the client's reaction to a 401) does a real refresh; both are
/// single-flight here as well, and a refused refresh signs out.
final class Session extends ChangeNotifier {
  /// [clock] is the test seam for expiry.
  Session({
    required AuthRepository repository,
    required this._store,
    Clock? clock,
  }) : _repo = repository,
       _clock = clock ?? const SystemClock();

  final AuthRepository _repo;
  final TokenStore _store;
  final Clock _clock;

  SessionState _state = const SessionUnknown();
  Tokens? _tokens;
  Future<String?>? _refreshing;

  /// The current state.
  SessionState get state => _state;

  /// The tokens, for the settings screen to show the expiry; never for a
  /// feature to send itself.
  Tokens? get tokens => _tokens;

  /// Read the store: a previous run's session, or nothing.
  Future<void> restore() async {
    final stored = await _store.read();
    if (stored == null) {
      _set(const SignedOut(), null);
      return;
    }
    _set(SignedIn(Principal.fromTokens(stored)), stored);
  }

  /// Open the browser; on return, the session is signed in or the state
  /// says why not.
  Future<void> signIn() async {
    if (_state is SigningIn) return;
    _set(const SigningIn(), null);
    final r = await _repo.signIn();
    switch (r) {
      case Ok(:final value):
        await _store.write(value);
        _set(SignedIn(Principal.fromTokens(value)), value);
      case Err(:final error):
        _set(SignedOut(error: error), null);
    }
  }

  /// Clear the store first, so the app is signed out whatever the issuer
  /// says; then end the browser session at the issuer, best effort.
  Future<void> signOut() async {
    final had = _tokens;
    await _store.clear();
    _set(const SignedOut(), null);
    if (had != null) await _repo.endSession(had);
  }

  /// The client's bearer: the current access token, refreshed first when it
  /// is about to expire and there is a refresh token to do it with.
  Future<String?> accessToken() async {
    final t = _tokens;
    if (t == null) return null;
    if (t.fresh(_clock.now()) || t.refreshToken == null) return t.accessToken;
    return await _refreshOnce();
  }

  /// The client saw a 401: get a new access token or end the session.
  Future<String?> refresh() => _refreshOnce();

  Future<String?> _refreshOnce() {
    final inFlight = _refreshing;
    if (inFlight != null) return inFlight;
    final started = _doRefresh().whenComplete(() => _refreshing = null);
    _refreshing = started;
    return started;
  }

  Future<String?> _doRefresh() async {
    final t = _tokens;
    final rt = t?.refreshToken;
    if (t == null || rt == null) {
      // Nothing to refresh with: a machine token is what it is.
      return t?.accessToken;
    }
    final r = await _repo.refresh(rt);
    switch (r) {
      case Ok(:final value):
        final merged = t.merge(value);
        await _store.write(merged);
        _set(SignedIn(Principal.fromTokens(merged)), merged);
        return merged.accessToken;
      case Err(:final error):
        if (error is UnauthenticatedError) {
          await _store.clear();
          _set(SignedOut(error: error), null);
          return null;
        }
        // The network, not the issuer: keep the session, the call fails on
        // its own terms and the next one tries again.
        return t.accessToken;
    }
  }

  void _set(SessionState s, Tokens? t) {
    _state = s;
    _tokens = t;
    notifyListeners();
  }
}
