import 'dart:convert';

/// An unsigned JWT with these claims, for tests that read claims and never
/// verify (the app does not verify either; the edge does).
String fakeJwt(Map<String, Object?> claims) {
  String part(Object o) =>
      base64Url.encode(utf8.encode(jsonEncode(o))).replaceAll('=', '');
  return '${part({'alg': 'none'})}.${part(claims)}.sig';
}
