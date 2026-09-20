import 'dart:async';

import 'package:dio/dio.dart';
import 'package:tbd_api/src/tokens.dart';
import 'package:tbd_core/tbd_core.dart';

/// The request option that marks a retry after a refresh, so a second 401
/// ends the session instead of looping.
const _retriedAfterRefresh = 'tbd.retried_after_refresh';

/// `traceparent` on every request, one trace per request, so the spans the
/// edge and the services record link back to the tap.
final class TraceInterceptor extends Interceptor {
  /// See [Tracer].
  TraceInterceptor(this.tracer);

  /// Where the ids come from.
  final Tracer tracer;

  @override
  void onRequest(RequestOptions options, RequestInterceptorHandler handler) {
    options.headers.putIfAbsent('traceparent', () => tracer.next().traceparent);
    handler.next(options);
  }
}

/// The bearer on every request, and one refresh when the edge says 401.
///
/// Ten calls refused at once cause one refresh: the first starts it and the
/// rest await the same future; a call refused with a token that has since
/// been replaced retries with the current one and refreshes nothing. The
/// retry happens once; a 401 on it, or a refresh the token endpoint refuses,
/// is the end of the session ([onSessionEnded] fires, the call fails
/// [UnauthenticatedError]).
final class BearerInterceptor extends Interceptor {
  /// [dio] is the client the retry goes through.
  BearerInterceptor(this.tokens, this.dio, {this.onSessionEnded});

  /// Where tokens come from; never held here.
  final TokenSource tokens;

  /// The client itself, for the retry.
  final Dio dio;

  /// Told once when a refresh is refused.
  final void Function()? onSessionEnded;

  Future<String?>? _refreshing;

  @override
  Future<void> onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) async {
    final token = await tokens.accessToken();
    if (token != null) {
      options.headers['authorization'] = 'Bearer $token';
    }
    handler.next(options);
  }

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    final status = err.response?.statusCode;
    final already = err.requestOptions.extra[_retriedAfterRefresh] == true;
    if (status != 401 || already) {
      handler.next(err);
      return;
    }
    final refused = _bearer(err.requestOptions);
    final current = await tokens.accessToken();
    final fresh = current != null && current != refused
        ? current
        : await _refreshOnce();
    if (fresh == null) {
      onSessionEnded?.call();
      handler.reject(
        DioException(
          requestOptions: err.requestOptions,
          response: err.response,
          type: DioExceptionType.badResponse,
          error: const UnauthenticatedError('refresh refused'),
        ),
      );
      return;
    }
    final retry = err.requestOptions
      ..headers['authorization'] = 'Bearer $fresh'
      ..extra[_retriedAfterRefresh] = true;
    try {
      handler.resolve(await dio.fetch<Object?>(retry));
    } on DioException catch (e) {
      handler.reject(e);
    }
  }

  static String? _bearer(RequestOptions o) {
    final h = o.headers['authorization'];
    return h is String && h.startsWith('Bearer ') ? h.substring(7) : null;
  }

  /// Single flight: whoever comes second awaits the first's refresh.
  Future<String?> _refreshOnce() {
    final inFlight = _refreshing;
    if (inFlight != null) return inFlight;
    final started = tokens.refresh().whenComplete(() => _refreshing = null);
    _refreshing = started;
    return started;
  }
}

/// Idempotent calls (GET, HEAD, OPTIONS) are retried on a network failure or
/// a 502/503/504, with a short backoff, [attempts] times at most. Nothing
/// that changes state is ever retried here; the caller decides that with an
/// idempotency key of its own.
final class RetryInterceptor extends Interceptor {
  /// [dio] is the client the retry goes through; [delay] is a test seam.
  RetryInterceptor(this.dio, {this.attempts = 2, this.delay = _backoff});

  /// The client itself.
  final Dio dio;

  /// Retries after the first attempt.
  final int attempts;

  /// The wait before attempt `n` (1-based).
  final Future<void> Function(int attempt) delay;

  static const _attempt = 'tbd.retry_attempt';
  static const _methods = {'GET', 'HEAD', 'OPTIONS'};

  static Future<void> _backoff(int attempt) =>
      Future<void>.delayed(Duration(milliseconds: 200 * attempt));

  @override
  Future<void> onError(
    DioException err,
    ErrorInterceptorHandler handler,
  ) async {
    final o = err.requestOptions;
    final attempt = (o.extra[_attempt] as int? ?? 0) + 1;
    if (!_methods.contains(o.method.toUpperCase()) ||
        attempt > attempts ||
        !_transient(err)) {
      handler.next(err);
      return;
    }
    await delay(attempt);
    o.extra[_attempt] = attempt;
    try {
      handler.resolve(await dio.fetch<Object?>(o));
    } on DioException catch (e) {
      handler.reject(e);
    }
  }

  static bool _transient(DioException e) => switch (e.type) {
    DioExceptionType.connectionTimeout ||
    DioExceptionType.receiveTimeout ||
    DioExceptionType.connectionError => true,
    DioExceptionType.badResponse => const {
      502,
      503,
      504,
    }.contains(e.response?.statusCode),
    _ => false,
  };
}
