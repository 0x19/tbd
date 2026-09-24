import 'package:tbd_api/tbd_api.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:tbd_proto/tbd_proto.dart';

/// The protocol's hellos without a network: each answer is a knob, and the
/// calls are counted.
final class FakeProtocolApi implements ProtocolApi {
  /// Defaults answer as the real edge would for a signed-in person.
  FakeProtocolApi({Result<Me>? me, Result<PingResponse>? ping})
    : meResult =
          me ??
          const Ok(
            Me(
              subject: 'u-1',
              kind: 'person',
              scopes: ['tbd.api'],
              clientId: 'tbd-app',
              role: 'admin',
            ),
          ),
      pingResult = ping ?? _echo;

  static Result<PingResponse> get _echo =>
      Ok(PingResponse(message: 'pong', protocolVersion: '0.1.0-test'));

  /// What `me` answers.
  Result<Me> meResult;

  /// What `ping` answers.
  Result<PingResponse> pingResult;

  /// Calls so far.
  int meCalls = 0;

  /// Calls so far.
  int pingCalls = 0;

  @override
  Future<Result<Me>> me() async {
    meCalls++;
    return meResult;
  }

  @override
  Future<Result<PingResponse>> ping(String message) async {
    pingCalls++;
    return pingResult;
  }
}
