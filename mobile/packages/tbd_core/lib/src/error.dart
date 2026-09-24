import 'package:meta/meta.dart';

/// Every outcome a screen may have to explain, as a closed set. A feature
/// adds a subtype here only when no existing one says what happened; the
/// view layer maps each to a localized sentence, never the raw message.
@immutable
sealed class AppError implements Exception {
  /// [message] is for logs; [cause] the exception underneath, if any.
  const AppError(this.message, {this.cause});

  /// For logs and support, not for people.
  final String message;

  /// What was caught underneath, kept for the log line.
  final Object? cause;

  /// The subtype's name, the message, and the cause when there is one.
  String describe(String kind) =>
      '$kind: $message${cause == null ? '' : ' ($cause)'}';
}

/// The device could not reach the edge, or the edge did not answer in time.
final class NetworkError extends AppError {
  /// See [AppError].
  const NetworkError(super.message, {super.cause});

  @override
  String toString() => describe('NetworkError');
}

/// No valid session: never signed in, refresh refused, or signed out.
final class UnauthenticatedError extends AppError {
  /// See [AppError].
  const UnauthenticatedError(super.message, {super.cause});

  @override
  String toString() => describe('UnauthenticatedError');
}

/// Signed in, but this call is not allowed for this principal (403).
final class ForbiddenError extends AppError {
  /// See [AppError].
  const ForbiddenError(super.message, {super.cause});

  @override
  String toString() => describe('ForbiddenError');
}

/// The server said no with a reason: an `application/problem+json` body,
/// or a gRPC status. [status] is the HTTP or gRPC code; [code] the server's
/// own machine-readable word when it gave one.
final class ServerError extends AppError {
  /// See [AppError].
  const ServerError(
    super.message, {
    required this.status,
    this.code,
    super.cause,
  });

  /// The HTTP status, or the gRPC status code.
  final int status;

  /// The server's machine-readable word for it (`problem.type`), if any.
  final String? code;

  @override
  String toString() =>
      describe('ServerError($status${code == null ? '' : ' $code'})');
}

/// The person cancelled something that needed them: the sign-in browser.
final class CancelledError extends AppError {
  /// See [AppError].
  const CancelledError(super.message);

  @override
  String toString() => describe('CancelledError');
}

/// The build is wrong: a missing define, a malformed URL. Fails at start.
final class ConfigError extends AppError {
  /// See [AppError].
  const ConfigError(super.message, {super.cause});

  @override
  String toString() => describe('ConfigError');
}
