import 'package:grpc/grpc.dart';
import 'package:tbd_api/src/client.dart';
import 'package:tbd_api/src/grpc.dart';
import 'package:tbd_api/src/me.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:tbd_proto/tbd_proto.dart';

/// What the protocol itself answers: who the caller is, over REST, and a
/// ping over gRPC. The two together prove one token through both doors.
final class ProtocolApi {
  /// [api] for REST, [grpc] for the proto service.
  ProtocolApi(this.api, this.grpc);

  /// The REST client.
  final ApiClient api;

  /// The gRPC edge.
  final GrpcEdge grpc;

  /// `GET /v1/me`.
  Future<Result<Me>> me() async =>
      (await api.getJson('/v1/me')).map(Me.fromJson);

  /// `tbd.protocol.v1.ProtocolService/Ping`.
  Future<Result<PingResponse>> ping(String message) async {
    try {
      final client = ProtocolServiceClient(grpc.channel);
      return Ok(
        await client.ping(
          PingRequest(message: message),
          options: grpc.options(),
        ),
      );
    } on GrpcError catch (e) {
      return Err(errorFromGrpc(e));
    }
  }
}
