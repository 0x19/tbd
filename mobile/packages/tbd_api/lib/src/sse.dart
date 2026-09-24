import 'dart:async';
import 'dart:convert';

import 'package:dio/dio.dart';
import 'package:tbd_api/src/client.dart';

/// One server-sent event: the protocol writes one JSON message per `data:`
/// frame, with `event:` naming the kind (`docs/protocol/README.md`).
final class SseEvent {
  /// [event] is the frame's `event:` line, empty when absent.
  const SseEvent({required this.event, required this.data, this.id});

  /// The frame's `event:` field.
  final String event;

  /// The `data:` lines joined with newlines.
  final String data;

  /// The `id:` field, when sent.
  final String? id;
}

/// `GET` a stream of events through the client, so the bearer and the trace
/// header ride along. The stream ends when the server closes it or [cancel]
/// fires; reconnecting is the caller's policy.
Stream<SseEvent> sse(
  ApiClient api,
  String path, {
  Map<String, Object?>? query,
  CancelToken? cancel,
}) async* {
  final r = await api.dio.get<ResponseBody>(
    path,
    queryParameters: query,
    cancelToken: cancel,
    options: Options(
      responseType: ResponseType.stream,
      headers: const {'accept': 'text/event-stream'},
      receiveTimeout: Duration.zero,
    ),
  );
  final body = r.data;
  if (body == null) return;
  yield* parseSse(body.stream.cast<List<int>>());
}

/// The event-stream format, as bytes come: `\n\n` ends a frame, `:` starts a
/// comment, a field without a value is allowed.
Stream<SseEvent> parseSse(Stream<List<int>> bytes) async* {
  var event = '';
  final data = <String>[];
  String? id;
  await for (final line
      in bytes.transform(utf8.decoder).transform(const LineSplitter())) {
    if (line.isEmpty) {
      if (data.isNotEmpty) {
        yield SseEvent(event: event, data: data.join('\n'), id: id);
      }
      event = '';
      data.clear();
      continue;
    }
    if (line.startsWith(':')) continue;
    final colon = line.indexOf(':');
    final field = colon < 0 ? line : line.substring(0, colon);
    var value = colon < 0 ? '' : line.substring(colon + 1);
    if (value.startsWith(' ')) value = value.substring(1);
    switch (field) {
      case 'event':
        event = value;
      case 'data':
        data.add(value);
      case 'id':
        id = value;
      default:
        break;
    }
  }
  if (data.isNotEmpty) {
    yield SseEvent(event: event, data: data.join('\n'), id: id);
  }
}
