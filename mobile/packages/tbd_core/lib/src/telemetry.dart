import 'dart:math';

import 'package:meta/meta.dart';

/// W3C Trace Context for outgoing requests, so a tap on the phone and the
/// spans Envoy, the protocol and the engine record for it share one trace.
///
/// This is propagation only. Exporting the app's own spans needs an OTLP
/// door that does not exist yet (`Env.otlpEndpoint` is empty until it does);
/// when it opens, an OpenTelemetry SDK slots in behind the same interface.
abstract interface class Tracer {
  /// A fresh trace context for one outgoing request.
  TraceContext next();
}

/// Random ids, one trace per request: the right thing while nothing on the
/// device stitches screens into a longer story.
final class RandomTracer implements Tracer {
  /// [random] is for tests; the app uses a secure one.
  RandomTracer([Random? random]) : _random = random ?? Random.secure();
  final Random _random;

  @override
  TraceContext next() => TraceContext(traceId: _hex(16), spanId: _hex(8));

  String _hex(int bytes) {
    final sb = StringBuffer();
    for (var i = 0; i < bytes; i++) {
      sb.write(_random.nextInt(256).toRadixString(16).padLeft(2, '0'));
    }
    return sb.toString();
  }
}

/// One `traceparent` value.
@immutable
class TraceContext {
  /// Ids as lowercase hex, as the header carries them.
  const TraceContext({
    required this.traceId,
    required this.spanId,
    this.sampled = true,
  });

  /// 32 hex chars.
  final String traceId;

  /// 16 hex chars.
  final String spanId;

  /// The sampled flag; on, so the edge keeps the trace.
  final bool sampled;

  /// `00-<trace>-<span>-01`, the header the edge reads.
  String get traceparent => '00-$traceId-$spanId-${sampled ? '01' : '00'}';
}
