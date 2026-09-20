import 'package:meta/meta.dart';

import 'package:tbd_core/src/error.dart';

/// Either a value or an [AppError]. A repository never throws for an outcome
/// a screen must handle; it returns this, and the view model switches on it.
/// Exceptions are for bugs.
@immutable
sealed class Result<T> {
  /// See [Ok] and [Err].
  const Result();

  /// A value.
  const factory Result.ok(T value) = Ok<T>;

  /// An error a screen has to explain.
  const factory Result.err(AppError error) = Err<T>;

  /// Run [body] and turn an [AppError] thrown or a value returned into a
  /// result; any other exception is a bug and propagates.
  static Future<Result<T>> guard<T>(Future<T> Function() body) async {
    try {
      return Ok(await body());
    } on AppError catch (e) {
      return Err(e);
    }
  }

  /// Whether this is an [Ok].
  bool get isOk => this is Ok<T>;

  /// One branch for each case; the compiler makes sure both are there.
  R when<R>({
    required R Function(T value) ok,
    required R Function(AppError error) err,
  }) => switch (this) {
    Ok(:final value) => ok(value),
    Err(:final error) => err(error),
  };

  /// Transform the value, keep the error.
  Result<U> map<U>(U Function(T value) f) => switch (this) {
    Ok(:final value) => Ok(f(value)),
    Err(:final error) => Err(error),
  };
}

/// The success case.
final class Ok<T> extends Result<T> {
  /// Wrap [value].
  const Ok(this.value);

  /// What the operation produced.
  final T value;

  @override
  String toString() => 'Ok($value)';
}

/// The failure case.
final class Err<T> extends Result<T> {
  /// Wrap [error].
  const Err(this.error);

  /// Why it failed, as a closed set the view can switch on.
  final AppError error;

  @override
  String toString() => 'Err($error)';
}
