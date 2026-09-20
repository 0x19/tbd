import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:dio/dio.dart';

/// One programmed answer.
final class Reply {
  const Reply(this.status, [this.body, this.headers]);

  const Reply.json(int status, Map<String, Object?> body)
    : this(status, body, const {
        'content-type': ['application/json'],
      });

  final int status;
  final Object? body;
  final Map<String, List<String>>? headers;
}

/// A dio adapter that answers from a script and records what it saw.
/// [next] decides per request from the request and how many were seen.
final class FakeAdapter implements HttpClientAdapter {
  FakeAdapter(this.next);

  final FutureOr<Reply> Function(RequestOptions o, int n) next;
  final List<RequestOptions> seen = [];

  @override
  Future<ResponseBody> fetch(
    RequestOptions options,
    Stream<Uint8List>? requestStream,
    Future<void>? cancelFuture,
  ) async {
    seen.add(options);
    final r = await next(options, seen.length);
    final text = switch (r.body) {
      null => '',
      final String s => s,
      final Object o => jsonEncode(o),
    };
    return ResponseBody.fromString(text, r.status, headers: r.headers);
  }

  @override
  void close({bool force = false}) {}
}
