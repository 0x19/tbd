// Merge configs/mobile/base.json with configs/mobile/<env>.json into the
// flat --dart-define-from-file the app is built with.
//
// The rule is the one every binary in this repository follows: every key lives
// in base, an environment file carries only differences, and the merged
// result is what runs. Objects merge key by key; anything else replaces.
// Keys are flattened with dots (`auth.issuer`) because dart-define values are
// strings, and `Env` in tbd_core reads them by those names.
//
//   dart run tool/env.dart local            # writes build/env.local.json
//   dart run tool/env.dart local --print    # prints the merged result instead
import 'dart:convert';
import 'dart:io';

void main(List<String> args) {
  if (args.isEmpty) {
    stderr.writeln('usage: dart run tool/env.dart <env> [--print]');
    exit(2);
  }
  final env = args.first;
  final print_ = args.contains('--print');
  final root = _repoRoot();
  final base = _read('$root/configs/mobile/base.json');
  final over = _read('$root/configs/mobile/$env.json');
  final merged = _merge(base, over);
  final flat = <String, String>{};
  _flatten('', merged..remove('_comment'), flat);
  final out = const JsonEncoder.withIndent('  ').convert(flat);
  if (print_) {
    stdout.writeln(out);
    return;
  }
  final target = File('$root/mobile/build/env.$env.json')
    ..parent.createSync(recursive: true)
    ..writeAsStringSync('$out\n');
  stdout.writeln('wrote ${target.path} (${flat.length} keys)');
}

/// The repository root: the nearest ancestor holding configs/mobile.
String _repoRoot() {
  var dir = Directory.current;
  while (true) {
    if (Directory('${dir.path}/configs/mobile').existsSync()) return dir.path;
    final parent = dir.parent;
    if (parent.path == dir.path) {
      stderr.writeln(
        'configs/mobile not found above ${Directory.current.path}',
      );
      exit(2);
    }
    dir = parent;
  }
}

Map<String, Object?> _read(String path) {
  final file = File(path);
  if (!file.existsSync()) {
    stderr.writeln('no such config: $path');
    exit(2);
  }
  return jsonDecode(file.readAsStringSync()) as Map<String, Object?>;
}

Map<String, Object?> _merge(
  Map<String, Object?> base,
  Map<String, Object?> over,
) {
  final out = Map<String, Object?>.from(base);
  over.forEach((key, value) {
    final existing = out[key];
    if (value is Map<String, Object?> && existing is Map<String, Object?>) {
      out[key] = _merge(existing, value);
    } else {
      out[key] = value;
    }
  });
  return out;
}

void _flatten(
  String prefix,
  Map<String, Object?> node,
  Map<String, String> out,
) {
  node.forEach((key, value) {
    if (key == '_comment') return;
    final name = prefix.isEmpty ? key : '$prefix.$key';
    if (value is Map<String, Object?>) {
      _flatten(name, value, out);
    } else {
      out[name] = value?.toString() ?? '';
    }
  });
}
