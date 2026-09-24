import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:tbd_mobile/app/providers.dart';

/// What the sign-in screen shows: whether the browser is open, and why the
/// last attempt ended, if it did.
final class SignInUiState {
  const SignInUiState({required this.busy, this.error});

  final bool busy;
  final AppError? error;
}

/// The sign-in screen's view model: a projection of the session, and the one
/// action.
final signInViewModelProvider =
    NotifierProvider<SignInViewModel, SignInUiState>(SignInViewModel.new);

class SignInViewModel extends Notifier<SignInUiState> {
  @override
  SignInUiState build() => switch (ref.watch(sessionStateProvider)) {
    SigningIn() => const SignInUiState(busy: true),
    SignedOut(:final error) => SignInUiState(busy: false, error: error),
    _ => const SignInUiState(busy: false),
  };

  Future<void> signIn() => ref.read(sessionProvider).signIn();
}
