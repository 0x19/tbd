import 'package:flutter_test/flutter_test.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:tbd_testing/tbd_testing.dart';

void main() {
  final clock = FixedClock(DateTime.utc(2026, 9, 20, 12));
  final person = fakeJwt({
    'sub': 'u-1',
    'email': 'nevio@example.test',
    'name': 'Nevio',
  });

  (Session, FakeAuthRepository, MemoryTokenStore) make({Tokens? stored}) {
    final repo = FakeAuthRepository(
      token: 'access-1',
      idToken: person,
      clock: clock,
    );
    final store = MemoryTokenStore(stored);
    return (Session(repository: repo, store: store, clock: clock), repo, store);
  }

  test('an empty store restores to signed out', () async {
    final (session, _, _) = make();
    expect(session.state, isA<SessionUnknown>());
    await session.restore();
    expect(session.state, isA<SignedOut>());
    expect(await session.accessToken(), isNull);
  });

  test('sign-in stores the tokens and greets from the ID token', () async {
    final (session, repo, store) = make();
    final states = <SessionState>[];
    session.addListener(() => states.add(session.state));
    await session.restore();
    await session.signIn();
    final s = session.state;
    expect(s, isA<SignedIn>());
    expect((s as SignedIn).principal.displayName, 'Nevio');
    expect(s.principal.email, 'nevio@example.test');
    expect(store.writes, 1);
    expect(repo.signIns, 1);
    expect(states.whereType<SigningIn>(), hasLength(1));
    expect(await session.accessToken(), 'access-1');
  });

  test(
    'a cancelled browser leaves the session signed out with the reason',
    () async {
      final (session, repo, _) = make();
      await session.restore();
      repo.cancelNext = true;
      await session.signIn();
      expect(session.state, isA<SignedOut>());
      expect((session.state as SignedOut).error, isA<CancelledError>());
    },
  );

  test('a previous run restores as signed in', () async {
    final stored = Tokens(
      accessToken: 'old',
      refreshToken: 'refresh-1',
      idToken: person,
      expiresAt: clock.now().add(const Duration(minutes: 30)),
    );
    final (session, repo, _) = make(stored: stored);
    await session.restore();
    expect(session.state, isA<SignedIn>());
    expect(await session.accessToken(), 'old');
    expect(repo.refreshes, 0);
  });

  test(
    'a token about to expire is refreshed once, ahead of the call',
    () async {
      final (session, repo, store) = make();
      await session.restore();
      await session.signIn();
      clock.advance(const Duration(minutes: 59, seconds: 30));
      final tokens = await Future.wait([
        session.accessToken(),
        session.accessToken(),
        session.accessToken(),
      ]);
      expect(tokens, everyElement('access-1-r1'));
      expect(repo.refreshes, 1);
      expect(store.writes, 2);
      expect(session.state, isA<SignedIn>());
      clock.advance(const Duration(minutes: 60));
      expect(await session.accessToken(), 'access-1-r2');
    },
  );

  test('a refresh the issuer refuses signs out and clears the store', () async {
    final (session, repo, store) = make();
    await session.restore();
    await session.signIn();
    repo.refuseRefresh = true;
    expect(await session.refresh(), isNull);
    expect(session.state, isA<SignedOut>());
    expect((session.state as SignedOut).error, isA<UnauthenticatedError>());
    expect(await store.read(), isNull);
  });

  test('a refresh the network loses keeps the session', () async {
    final (session, repo, _) = make();
    await session.restore();
    await session.signIn();
    repo.offline = true;
    expect(await session.refresh(), 'access-1');
    expect(session.state, isA<SignedIn>());
  });

  test('a machine token has nothing to refresh with and stays', () async {
    final (session, repo, _) = make();
    repo
      ..idToken = null
      ..refreshToken = null
      ..token = fakeJwt({'sub': 'tbd-chaos'});
    await session.restore();
    await session.signIn();
    expect((session.state as SignedIn).principal.displayName, 'tbd-chaos');
    clock.advance(const Duration(hours: 2));
    expect(await session.accessToken(), repo.token);
    expect(repo.refreshes, 0);
  });

  test('sign-out clears the store first and then tells the issuer', () async {
    final (session, repo, store) = make();
    await session.restore();
    await session.signIn();
    await session.signOut();
    expect(session.state, isA<SignedOut>());
    expect(await store.read(), isNull);
    expect(repo.ended, 1);
    expect(await session.accessToken(), isNull);
  });
}
