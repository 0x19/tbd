/// Reaching the edge. One client, one place for every cross-cutting
/// concern: the bearer with single-flight refresh, `traceparent`, timeouts,
/// retries for idempotent calls, the protocol's problem body as `AppError`;
/// an SSE reader through the same client; a gRPC channel with the same
/// metadata; and the few hand-written types the REST surface has that the
/// protos do not.
library;

export 'package:dio/dio.dart' show CancelToken;

export 'src/client.dart';
export 'src/grpc.dart';
export 'src/interceptors.dart';
export 'src/me.dart';
export 'src/problem.dart';
export 'src/protocol_api.dart';
export 'src/sse.dart';
export 'src/tokens.dart';
