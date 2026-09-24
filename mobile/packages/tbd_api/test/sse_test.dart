import 'dart:convert';

import 'package:tbd_api/tbd_api.dart';
import 'package:test/test.dart';

void main() {
  test('frames split on blank lines; comments and ids are handled', () async {
    const text =
        ': keep-alive\n'
        'event: connector\n'
        'id: 7\n'
        'data: {"a":1}\n'
        '\n'
        'data: line one\n'
        'data: line two\n'
        '\n'
        'data: tail without newline';
    final events = await parseSse(
      Stream.fromIterable([
        utf8.encode(text.substring(0, 20)),
        utf8.encode(text.substring(20)),
      ]),
    ).toList();
    expect(events, hasLength(3));
    expect(events[0].event, 'connector');
    expect(events[0].id, '7');
    expect(events[0].data, '{"a":1}');
    expect(events[1].data, 'line one\nline two');
    expect(events[2].data, 'tail without newline');
  });
}
