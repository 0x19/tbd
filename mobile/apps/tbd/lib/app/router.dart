import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_mobile/features/auth/sign_in_view.dart';
import 'package:tbd_mobile/features/home/home_view.dart';

/// Where a route lives; features add theirs here.
abstract final class Routes {
  /// The home screen, signed in.
  static const home = '/';

  /// Sign-in, signed out.
  static const signIn = '/sign-in';

  /// While the store is being read.
  static const splash = '/splash';
}

/// The router, guarded by the session: a signed-out person sees sign-in
/// whatever they asked for, a signed-in one never sees it. The session is a
/// `Listenable`, so a change re-runs the redirect.
GoRouter buildRouter(Session session) => GoRouter(
  initialLocation: Routes.splash,
  refreshListenable: session,
  redirect: (context, state) {
    final at = state.matchedLocation;
    return switch (session.state) {
      SessionUnknown() => at == Routes.splash ? null : Routes.splash,
      SignedOut() || SigningIn() => at == Routes.signIn ? null : Routes.signIn,
      SignedIn() =>
        at == Routes.signIn || at == Routes.splash ? Routes.home : null,
    };
  },
  routes: [
    GoRoute(
      path: Routes.splash,
      builder: (_, _) =>
          const Scaffold(body: Center(child: CircularProgressIndicator())),
    ),
    GoRoute(path: Routes.signIn, builder: (_, _) => const SignInView()),
    GoRoute(path: Routes.home, builder: (_, _) => const HomeView()),
  ],
);
