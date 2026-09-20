import 'package:grpc/grpc.dart';
import 'package:tbd_api/tbd_api.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:test/test.dart';

import 'client_test.dart' show env;

void main() {
  test('call options carry the bearer and a traceparent', () async {
    final edge = GrpcEdge(env, tokens: const StaticTokenSource('abc'));
    final md = <String, String>{};
    for (final p in edge.options().metadataProviders) {
      await p(md, 'https://api.test/tbd.protocol.v1.ProtocolService/Ping');
    }
    expect(md['authorization'], 'Bearer abc');
    expect(md['traceparent'], startsWith('00-'));
    expect(edge.options().timeout, env.apiTimeout);
  });

  test('gRPC statuses map onto the closed error set', () {
    expect(
      errorFromGrpc(const GrpcError.unauthenticated('x')),
      isA<UnauthenticatedError>(),
    );
    expect(
      errorFromGrpc(const GrpcError.permissionDenied('x')),
      isA<ForbiddenError>(),
    );
    expect(
      errorFromGrpc(const GrpcError.unavailable('x')),
      isA<NetworkError>(),
    );
    final e =
        errorFromGrpc(const GrpcError.invalidArgument('x')) as ServerError;
    expect(e.status, StatusCode.invalidArgument);
    expect(e.code, 'INVALID_ARGUMENT');
  });
}
