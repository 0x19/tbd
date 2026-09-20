import 'package:dio/dio.dart';
import 'package:tbd_core/tbd_core.dart';

/// The protocol's error body: `{"code","error","details"}`
/// (`crates/protocol/src/error.rs`, `Problem` in `docs/protocol/openapi.json`).
/// Turned into the closed [AppError] set so a view switches on a type, not
/// on a status number.
AppError errorFrom(DioException e) {
  switch (e.type) {
    case DioExceptionType.connectionTimeout:
    case DioExceptionType.sendTimeout:
    case DioExceptionType.receiveTimeout:
    case DioExceptionType.connectionError:
    case DioExceptionType.badCertificate:
      return NetworkError(e.message ?? e.type.name, cause: e.error);
    case DioExceptionType.cancel:
      return const CancelledError('request cancelled');
    case DioExceptionType.badResponse:
      return errorFromResponse(e.response!);
    case DioExceptionType.transformTimeout:
    case DioExceptionType.unknown:
      return NetworkError(e.message ?? 'request failed', cause: e.error);
  }
}

/// A response the edge or the protocol refused.
AppError errorFromResponse(Response<Object?> r) {
  final status = r.statusCode ?? 0;
  final body = r.data;
  String? code;
  var message = 'HTTP $status';
  if (body is Map<String, Object?>) {
    code = body['code'] as String?;
    message = body['error'] as String? ?? message;
  } else if (body is String && body.isNotEmpty) {
    message = body;
  }
  return switch (status) {
    401 => UnauthenticatedError(message),
    403 => ForbiddenError(message),
    _ => ServerError(message, status: status, code: code),
  };
}
