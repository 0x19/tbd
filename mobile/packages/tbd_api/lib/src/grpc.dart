import 'package:grpc/grpc.dart';
import 'package:tbd_api/src/tokens.dart';
import 'package:tbd_core/tbd_core.dart';

/// A channel to the edge for the proto services, and the call options that
/// put the same bearer and `traceparent` on a gRPC call as on a REST one.
/// Port 443 means TLS against the public roots; anything else is plaintext,
/// for an emulator against a forwarded local edge.
final class GrpcEdge {
  /// See [channel] and [options].
  GrpcEdge(this.env, {required this.tokens, Tracer? tracer})
    : _tracer = tracer ?? RandomTracer();

  /// The build's configuration.
  final Env env;

  /// Where the bearer comes from.
  final TokenSource tokens;
  final Tracer _tracer;

  ClientChannel? _channel;

  /// The one channel, opened on first use.
  ClientChannel get channel {
    final existing = _channel;
    if (existing != null) return existing;
    final (host, port) = _authority(env.apiGrpcAuthority);
    final created = ClientChannel(
      host,
      port: port,
      options: ChannelOptions(
        credentials: port == 443
            ? const ChannelCredentials.secure()
            : const ChannelCredentials.insecure(),
        connectTimeout: env.apiTimeout,
      ),
    );
    return _channel = created;
  }

  /// Per-call metadata: the bearer as it is *now* (a refresh elsewhere is
  /// picked up by the next call) and a fresh trace context.
  CallOptions options({Duration? timeout}) => CallOptions(
    timeout: timeout ?? env.apiTimeout,
    providers: [
      (metadata, _) async {
        final token = await tokens.accessToken();
        if (token != null) metadata['authorization'] = 'Bearer $token';
        metadata['traceparent'] = _tracer.next().traceparent;
      },
    ],
  );

  /// Close the channel; the next use opens a new one.
  Future<void> shutdown() async {
    await _channel?.shutdown();
    _channel = null;
  }

  static (String, int) _authority(String authority) {
    final i = authority.lastIndexOf(':');
    if (i < 0) return (authority, 443);
    return (authority.substring(0, i), int.parse(authority.substring(i + 1)));
  }
}

/// A gRPC failure as the closed error set.
AppError errorFromGrpc(GrpcError e) => switch (e.code) {
  StatusCode.unauthenticated => UnauthenticatedError(
    e.message ?? 'unauthenticated',
  ),
  StatusCode.permissionDenied => ForbiddenError(e.message ?? 'forbidden'),
  StatusCode.cancelled => const CancelledError('call cancelled'),
  StatusCode.unavailable || StatusCode.deadlineExceeded => NetworkError(
    e.message ?? e.codeName,
    cause: e,
  ),
  _ => ServerError(e.message ?? e.codeName, status: e.code, code: e.codeName),
};
