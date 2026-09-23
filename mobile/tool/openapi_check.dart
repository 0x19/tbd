// The REST types this workspace writes by hand must match what the server
// serves. There is one today, `Me`; its property names are compared with the
// `Me` schema in docs/protocol/openapi.json, which the protocol's own test
// keeps equal to what it serves. Run by `mise run mobile:check`.
import 'dart:convert';
import 'dart:io';

/// Hand-written types and the schema each mirrors.
const checks = {
  'Me': [
    'subject',
    'kind',
    'client_id',
    'org',
    'key',
    'scopes',
    'role',
    'email',
    'name',
  ],
};

void main() {
  final root = _repoRoot();
  final doc =
      jsonDecode(File('$root/docs/protocol/openapi.json').readAsStringSync())
          as Map<String, Object?>?;
  if (doc == null) {
    stderr.writeln('openapi.json is empty');
    exit(1);
  }
  final components = doc['components'] as Map<String, Object?>? ?? {};
  final schemas = components['schemas'] as Map<String, Object?>? ?? {};
  var failed = false;
  checks.forEach((name, fields) {
    final schema = schemas[name] as Map<String, Object?>?;
    if (schema == null) {
      stderr.writeln('$name: no such schema in openapi.json');
      failed = true;
      return;
    }
    final server = ((schema['properties'] as Map<String, Object?>?) ?? {}).keys
        .toSet();
    final client = fields.toSet();
    final missing = server.difference(client);
    final extra = client.difference(server);
    if (missing.isNotEmpty || extra.isNotEmpty) {
      stderr.writeln(
        '$name drifted: server has '
        '${missing.isEmpty ? 'nothing new' : missing.join(', ')}; '
        'client has ${extra.isEmpty ? 'nothing extra' : extra.join(', ')}',
      );
      failed = true;
    } else {
      stdout.writeln('$name: ${fields.length} properties match openapi.json');
    }
  });
  if (failed) exit(1);
}

String _repoRoot() {
  var dir = Directory.current;
  while (!File('${dir.path}/docs/protocol/openapi.json').existsSync()) {
    if (dir.parent.path == dir.path) {
      stderr.writeln(
        'docs/protocol/openapi.json not found above ${Directory.current.path}',
      );
      exit(2);
    }
    dir = dir.parent;
  }
  return dir.path;
}
