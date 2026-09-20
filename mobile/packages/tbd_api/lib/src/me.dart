import 'package:meta/meta.dart';

/// `GET /v1/me`: who the edge verified this call to be.
///
/// Hand-written, because the protocol answers this route with its own JSON
/// rather than a proto message. `tool/openapi_check.dart` holds [fields]
/// against the `Me` schema in `docs/protocol/openapi.json`, so a field
/// added or renamed on the server fails the mobile check rather than
/// silently reading as null here.
@immutable
class Me {
  /// All fields as the route returns them; see [Me.fromJson].
  const Me({
    required this.subject,
    required this.kind,
    required this.scopes,
    this.clientId,
    this.org,
    this.role,
  });

  /// From the route's JSON body, exactly as the protocol writes it.
  factory Me.fromJson(Map<String, Object?> json) => Me(
    subject: json['subject']! as String,
    kind: json['kind']! as String,
    clientId: json['client_id'] as String?,
    org: json['org'] as String?,
    scopes: (json['scopes'] as List<Object?>? ?? const []).cast<String>(),
    role: json['role'] as String?,
  );

  /// The property names this type reads; compared with the OpenAPI schema.
  static const fields = [
    'subject',
    'kind',
    'client_id',
    'org',
    'key',
    'scopes',
    'role',
  ];

  /// The `sub` claim: an identity id for a person, a client id otherwise.
  final String subject;

  /// `person`, `client` or `service`.
  final String kind;

  /// The OAuth client the token was minted for.
  final String? clientId;

  /// The organisation claim, when the identity carries one.
  final String? org;

  /// The token's scopes as the edge read them.
  final List<String> scopes;

  /// `admin`, `editor`, `viewer`, or null for a machine.
  final String? role;

  /// A signed-in person, as opposed to a machine client.
  bool get isPerson => kind == 'person';
}
