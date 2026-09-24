import 'dart:async';

import 'package:dio/dio.dart';
import 'package:tbd_api/src/interceptors.dart';
import 'package:tbd_api/src/problem.dart';
import 'package:tbd_api/src/tokens.dart';
import 'package:tbd_core/tbd_core.dart';

/// The one HTTP client every feature reaches the edge through: base URL and
/// timeouts from [Env], the bearer with single-flight refresh, `traceparent`,
/// retries for idempotent calls, and the protocol's problem body turned into
/// [AppError]. Features do not construct a `Dio`; they take this.
final class ApiClient {
  /// [adapter] is the test seam: a fake `HttpClientAdapter` answers without
  /// a network. [onSessionEnded] is how the session learns a refresh was
  /// refused.
  ApiClient(
    this.env, {
    required TokenSource tokens,
    Tracer? tracer,
    HttpClientAdapter? adapter,
    void Function()? onSessionEnded,
    Future<void> Function(int attempt)? retryDelay,
  }) : dio = Dio(
         BaseOptions(
           baseUrl: env.apiBaseUrl.toString(),
           connectTimeout: env.apiTimeout,
           receiveTimeout: env.apiTimeout,
           sendTimeout: env.apiTimeout,
           headers: const {'accept': 'application/json'},
           // Every status reaches the interceptors; 4xx/5xx become a
           // DioException there, after the bearer interceptor has had its say.
           validateStatus: (s) => s != null && s < 400,
         ),
       ) {
    if (adapter != null) dio.httpClientAdapter = adapter;
    dio.interceptors.addAll([
      TraceInterceptor(tracer ?? RandomTracer()),
      BearerInterceptor(tokens, dio, onSessionEnded: onSessionEnded),
      if (retryDelay == null)
        RetryInterceptor(dio)
      else
        RetryInterceptor(dio, delay: retryDelay),
    ]);
  }

  /// The build's configuration.
  final Env env;

  /// The configured client. Reach for it directly only for a stream; the
  /// typed helpers below cover everything else.
  final Dio dio;

  /// `GET` a JSON object.
  Future<Result<Map<String, Object?>>> getJson(
    String path, {
    Map<String, Object?>? query,
    CancelToken? cancel,
  }) => _run(() async {
    final r = await dio.get<Map<String, Object?>>(
      path,
      queryParameters: query,
      cancelToken: cancel,
    );
    return r.data ?? const {};
  });

  /// `POST` a JSON body, read a JSON object back.
  Future<Result<Map<String, Object?>>> postJson(
    String path,
    Map<String, Object?> body, {
    CancelToken? cancel,
  }) => _run(() async {
    final r = await dio.post<Map<String, Object?>>(
      path,
      data: body,
      cancelToken: cancel,
    );
    return r.data ?? const {};
  });

  /// Turn what dio throws into the closed error set. Anything that is not
  /// a `DioException` is a bug and propagates.
  static Future<Result<T>> _run<T>(Future<T> Function() body) async {
    try {
      return Ok(await body());
    } on DioException catch (e) {
      final inner = e.error;
      return Err(inner is AppError ? inner : errorFrom(e));
    }
  }
}
