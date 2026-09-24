// This is a generated file - do not edit.
//
// Generated from tbd/finance/v1/finance.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports
// ignore_for_file: unused_import

import 'dart:convert' as $convert;
import 'dart:core' as $core;
import 'dart:typed_data' as $typed_data;

@$core.Deprecated('Use connectorKindDescriptor instead')
const ConnectorKind$json = {
  '1': 'ConnectorKind',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'label', '3': 2, '4': 1, '5': 9, '10': 'label'},
    {'1': 'description', '3': 3, '4': 1, '5': 9, '10': 'description'},
    {'1': 'auth', '3': 4, '4': 1, '5': 9, '10': 'auth'},
    {'1': 'consent_note', '3': 5, '4': 1, '5': 9, '10': 'consentNote'},
    {'1': 'configured', '3': 6, '4': 1, '5': 8, '10': 'configured'},
  ],
};

/// Descriptor for `ConnectorKind`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List connectorKindDescriptor = $convert.base64Decode(
    'Cg1Db25uZWN0b3JLaW5kEhIKBG5hbWUYASABKAlSBG5hbWUSFAoFbGFiZWwYAiABKAlSBWxhYm'
    'VsEiAKC2Rlc2NyaXB0aW9uGAMgASgJUgtkZXNjcmlwdGlvbhISCgRhdXRoGAQgASgJUgRhdXRo'
    'EiEKDGNvbnNlbnRfbm90ZRgFIAEoCVILY29uc2VudE5vdGUSHgoKY29uZmlndXJlZBgGIAEoCF'
    'IKY29uZmlndXJlZA==');

@$core.Deprecated('Use listConnectorKindsRequestDescriptor instead')
const ListConnectorKindsRequest$json = {
  '1': 'ListConnectorKindsRequest',
};

/// Descriptor for `ListConnectorKindsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConnectorKindsRequestDescriptor =
    $convert.base64Decode('ChlMaXN0Q29ubmVjdG9yS2luZHNSZXF1ZXN0');

@$core.Deprecated('Use listConnectorKindsResponseDescriptor instead')
const ListConnectorKindsResponse$json = {
  '1': 'ListConnectorKindsResponse',
  '2': [
    {
      '1': 'kinds',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.ConnectorKind',
      '10': 'kinds'
    },
  ],
};

/// Descriptor for `ListConnectorKindsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConnectorKindsResponseDescriptor =
    $convert.base64Decode(
        'ChpMaXN0Q29ubmVjdG9yS2luZHNSZXNwb25zZRIzCgVraW5kcxgBIAMoCzIdLnRiZC5maW5hbm'
        'NlLnYxLkNvbm5lY3RvcktpbmRSBWtpbmRz');

@$core.Deprecated('Use connectorDescriptor instead')
const Connector$json = {
  '1': 'Connector',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'kind', '3': 3, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'label', '3': 4, '4': 1, '5': 9, '10': 'label'},
    {'1': 'status', '3': 5, '4': 1, '5': 9, '10': 'status'},
    {'1': 'config', '3': 6, '4': 1, '5': 9, '10': 'config'},
    {'1': 'external_id', '3': 7, '4': 1, '5': 9, '10': 'externalId'},
    {'1': 'linked_at', '3': 8, '4': 1, '5': 9, '10': 'linkedAt'},
    {'1': 'last_sync_at', '3': 9, '4': 1, '5': 9, '10': 'lastSyncAt'},
    {'1': 'last_sync_status', '3': 10, '4': 1, '5': 9, '10': 'lastSyncStatus'},
    {'1': 'last_sync_error', '3': 11, '4': 1, '5': 9, '10': 'lastSyncError'},
    {'1': 'failure', '3': 12, '4': 1, '5': 9, '10': 'failure'},
    {'1': 'created_at', '3': 13, '4': 1, '5': 9, '10': 'createdAt'},
    {'1': 'can_send', '3': 14, '4': 1, '5': 8, '10': 'canSend'},
  ],
};

/// Descriptor for `Connector`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List connectorDescriptor = $convert.base64Decode(
    'CglDb25uZWN0b3ISDgoCaWQYASABKAlSAmlkEhkKCHBhcnR5X2lkGAIgASgJUgdwYXJ0eUlkEh'
    'IKBGtpbmQYAyABKAlSBGtpbmQSFAoFbGFiZWwYBCABKAlSBWxhYmVsEhYKBnN0YXR1cxgFIAEo'
    'CVIGc3RhdHVzEhYKBmNvbmZpZxgGIAEoCVIGY29uZmlnEh8KC2V4dGVybmFsX2lkGAcgASgJUg'
    'pleHRlcm5hbElkEhsKCWxpbmtlZF9hdBgIIAEoCVIIbGlua2VkQXQSIAoMbGFzdF9zeW5jX2F0'
    'GAkgASgJUgpsYXN0U3luY0F0EigKEGxhc3Rfc3luY19zdGF0dXMYCiABKAlSDmxhc3RTeW5jU3'
    'RhdHVzEiYKD2xhc3Rfc3luY19lcnJvchgLIAEoCVINbGFzdFN5bmNFcnJvchIYCgdmYWlsdXJl'
    'GAwgASgJUgdmYWlsdXJlEh0KCmNyZWF0ZWRfYXQYDSABKAlSCWNyZWF0ZWRBdBIZCghjYW5fc2'
    'VuZBgOIAEoCFIHY2FuU2VuZA==');

@$core.Deprecated('Use listConnectorsRequestDescriptor instead')
const ListConnectorsRequest$json = {
  '1': 'ListConnectorsRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `ListConnectorsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConnectorsRequestDescriptor = $convert.base64Decode(
    'ChVMaXN0Q29ubmVjdG9yc1JlcXVlc3QSGwoJcGFydHlfaWRzGAEgAygJUghwYXJ0eUlkcw==');

@$core.Deprecated('Use listConnectorsResponseDescriptor instead')
const ListConnectorsResponse$json = {
  '1': 'ListConnectorsResponse',
  '2': [
    {
      '1': 'connectors',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Connector',
      '10': 'connectors'
    },
  ],
};

/// Descriptor for `ListConnectorsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConnectorsResponseDescriptor =
    $convert.base64Decode(
        'ChZMaXN0Q29ubmVjdG9yc1Jlc3BvbnNlEjkKCmNvbm5lY3RvcnMYASADKAsyGS50YmQuZmluYW'
        '5jZS52MS5Db25uZWN0b3JSCmNvbm5lY3RvcnM=');

@$core.Deprecated('Use startConnectorRequestDescriptor instead')
const StartConnectorRequest$json = {
  '1': 'StartConnectorRequest',
  '2': [
    {'1': 'party_id', '3': 1, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'kind', '3': 2, '4': 1, '5': 9, '10': 'kind'},
  ],
};

/// Descriptor for `StartConnectorRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List startConnectorRequestDescriptor = $convert.base64Decode(
    'ChVTdGFydENvbm5lY3RvclJlcXVlc3QSGQoIcGFydHlfaWQYASABKAlSB3BhcnR5SWQSEgoEa2'
    'luZBgCIAEoCVIEa2luZA==');

@$core.Deprecated('Use startConnectorResponseDescriptor instead')
const StartConnectorResponse$json = {
  '1': 'StartConnectorResponse',
  '2': [
    {'1': 'connector_id', '3': 1, '4': 1, '5': 9, '10': 'connectorId'},
    {'1': 'url', '3': 2, '4': 1, '5': 9, '10': 'url'},
  ],
};

/// Descriptor for `StartConnectorResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List startConnectorResponseDescriptor =
    $convert.base64Decode(
        'ChZTdGFydENvbm5lY3RvclJlc3BvbnNlEiEKDGNvbm5lY3Rvcl9pZBgBIAEoCVILY29ubmVjdG'
        '9ySWQSEAoDdXJsGAIgASgJUgN1cmw=');

@$core.Deprecated('Use completeConnectorRequestDescriptor instead')
const CompleteConnectorRequest$json = {
  '1': 'CompleteConnectorRequest',
  '2': [
    {'1': 'state', '3': 1, '4': 1, '5': 9, '10': 'state'},
    {'1': 'code', '3': 2, '4': 1, '5': 9, '10': 'code'},
  ],
};

/// Descriptor for `CompleteConnectorRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeConnectorRequestDescriptor =
    $convert.base64Decode(
        'ChhDb21wbGV0ZUNvbm5lY3RvclJlcXVlc3QSFAoFc3RhdGUYASABKAlSBXN0YXRlEhIKBGNvZG'
        'UYAiABKAlSBGNvZGU=');

@$core.Deprecated('Use completeConnectorResponseDescriptor instead')
const CompleteConnectorResponse$json = {
  '1': 'CompleteConnectorResponse',
  '2': [
    {
      '1': 'connector',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Connector',
      '10': 'connector'
    },
  ],
};

/// Descriptor for `CompleteConnectorResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeConnectorResponseDescriptor =
    $convert.base64Decode(
        'ChlDb21wbGV0ZUNvbm5lY3RvclJlc3BvbnNlEjcKCWNvbm5lY3RvchgBIAEoCzIZLnRiZC5maW'
        '5hbmNlLnYxLkNvbm5lY3RvclIJY29ubmVjdG9y');

@$core.Deprecated('Use testConnectorRequestDescriptor instead')
const TestConnectorRequest$json = {
  '1': 'TestConnectorRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `TestConnectorRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List testConnectorRequestDescriptor = $convert
    .base64Decode('ChRUZXN0Q29ubmVjdG9yUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use testConnectorResponseDescriptor instead')
const TestConnectorResponse$json = {
  '1': 'TestConnectorResponse',
  '2': [
    {'1': 'status', '3': 1, '4': 1, '5': 9, '10': 'status'},
  ],
};

/// Descriptor for `TestConnectorResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List testConnectorResponseDescriptor =
    $convert.base64Decode(
        'ChVUZXN0Q29ubmVjdG9yUmVzcG9uc2USFgoGc3RhdHVzGAEgASgJUgZzdGF0dXM=');

@$core.Deprecated('Use syncConnectorRequestDescriptor instead')
const SyncConnectorRequest$json = {
  '1': 'SyncConnectorRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `SyncConnectorRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncConnectorRequestDescriptor = $convert
    .base64Decode('ChRTeW5jQ29ubmVjdG9yUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use syncConnectorResponseDescriptor instead')
const SyncConnectorResponse$json = {
  '1': 'SyncConnectorResponse',
  '2': [
    {
      '1': 'run',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.ConnectorRun',
      '10': 'run'
    },
  ],
};

/// Descriptor for `SyncConnectorResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List syncConnectorResponseDescriptor = $convert.base64Decode(
    'ChVTeW5jQ29ubmVjdG9yUmVzcG9uc2USLgoDcnVuGAEgASgLMhwudGJkLmZpbmFuY2UudjEuQ2'
    '9ubmVjdG9yUnVuUgNydW4=');

@$core.Deprecated('Use configureConnectorRequestDescriptor instead')
const ConfigureConnectorRequest$json = {
  '1': 'ConfigureConnectorRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'config', '3': 2, '4': 1, '5': 9, '10': 'config'},
    {'1': 'party_id', '3': 3, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'label', '3': 4, '4': 1, '5': 9, '10': 'label'},
  ],
};

/// Descriptor for `ConfigureConnectorRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List configureConnectorRequestDescriptor = $convert.base64Decode(
    'ChlDb25maWd1cmVDb25uZWN0b3JSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZBIWCgZjb25maWcYAi'
    'ABKAlSBmNvbmZpZxIZCghwYXJ0eV9pZBgDIAEoCVIHcGFydHlJZBIUCgVsYWJlbBgEIAEoCVIF'
    'bGFiZWw=');

@$core.Deprecated('Use configureConnectorResponseDescriptor instead')
const ConfigureConnectorResponse$json = {
  '1': 'ConfigureConnectorResponse',
  '2': [
    {
      '1': 'connector',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Connector',
      '10': 'connector'
    },
  ],
};

/// Descriptor for `ConfigureConnectorResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List configureConnectorResponseDescriptor =
    $convert.base64Decode(
        'ChpDb25maWd1cmVDb25uZWN0b3JSZXNwb25zZRI3Cgljb25uZWN0b3IYASABKAsyGS50YmQuZm'
        'luYW5jZS52MS5Db25uZWN0b3JSCWNvbm5lY3Rvcg==');

@$core.Deprecated('Use deleteConnectorRequestDescriptor instead')
const DeleteConnectorRequest$json = {
  '1': 'DeleteConnectorRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteConnectorRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteConnectorRequestDescriptor = $convert
    .base64Decode('ChZEZWxldGVDb25uZWN0b3JSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use deleteConnectorResponseDescriptor instead')
const DeleteConnectorResponse$json = {
  '1': 'DeleteConnectorResponse',
};

/// Descriptor for `DeleteConnectorResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteConnectorResponseDescriptor =
    $convert.base64Decode('ChdEZWxldGVDb25uZWN0b3JSZXNwb25zZQ==');

@$core.Deprecated('Use mailTemplateDescriptor instead')
const MailTemplate$json = {
  '1': 'MailTemplate',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
    {'1': 'subject', '3': 4, '4': 1, '5': 9, '10': 'subject'},
    {'1': 'body', '3': 5, '4': 1, '5': 9, '10': 'body'},
    {'1': 'to', '3': 6, '4': 3, '5': 9, '10': 'to'},
    {'1': 'cc', '3': 7, '4': 3, '5': 9, '10': 'cc'},
    {'1': 'bcc', '3': 8, '4': 3, '5': 9, '10': 'bcc'},
    {'1': 'updated_at', '3': 9, '4': 1, '5': 9, '10': 'updatedAt'},
  ],
};

/// Descriptor for `MailTemplate`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mailTemplateDescriptor = $convert.base64Decode(
    'CgxNYWlsVGVtcGxhdGUSDgoCaWQYASABKAlSAmlkEhkKCHBhcnR5X2lkGAIgASgJUgdwYXJ0eU'
    'lkEhIKBG5hbWUYAyABKAlSBG5hbWUSGAoHc3ViamVjdBgEIAEoCVIHc3ViamVjdBISCgRib2R5'
    'GAUgASgJUgRib2R5Eg4KAnRvGAYgAygJUgJ0bxIOCgJjYxgHIAMoCVICY2MSEAoDYmNjGAggAy'
    'gJUgNiY2MSHQoKdXBkYXRlZF9hdBgJIAEoCVIJdXBkYXRlZEF0');

@$core.Deprecated('Use listMailTemplatesRequestDescriptor instead')
const ListMailTemplatesRequest$json = {
  '1': 'ListMailTemplatesRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `ListMailTemplatesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listMailTemplatesRequestDescriptor =
    $convert.base64Decode(
        'ChhMaXN0TWFpbFRlbXBsYXRlc1JlcXVlc3QSGwoJcGFydHlfaWRzGAEgAygJUghwYXJ0eUlkcw'
        '==');

@$core.Deprecated('Use listMailTemplatesResponseDescriptor instead')
const ListMailTemplatesResponse$json = {
  '1': 'ListMailTemplatesResponse',
  '2': [
    {
      '1': 'templates',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.MailTemplate',
      '10': 'templates'
    },
  ],
};

/// Descriptor for `ListMailTemplatesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listMailTemplatesResponseDescriptor =
    $convert.base64Decode(
        'ChlMaXN0TWFpbFRlbXBsYXRlc1Jlc3BvbnNlEjoKCXRlbXBsYXRlcxgBIAMoCzIcLnRiZC5maW'
        '5hbmNlLnYxLk1haWxUZW1wbGF0ZVIJdGVtcGxhdGVz');

@$core.Deprecated('Use upsertMailTemplateRequestDescriptor instead')
const UpsertMailTemplateRequest$json = {
  '1': 'UpsertMailTemplateRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
    {'1': 'subject', '3': 4, '4': 1, '5': 9, '10': 'subject'},
    {'1': 'body', '3': 5, '4': 1, '5': 9, '10': 'body'},
    {'1': 'to', '3': 6, '4': 3, '5': 9, '10': 'to'},
    {'1': 'cc', '3': 7, '4': 3, '5': 9, '10': 'cc'},
    {'1': 'bcc', '3': 8, '4': 3, '5': 9, '10': 'bcc'},
  ],
};

/// Descriptor for `UpsertMailTemplateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertMailTemplateRequestDescriptor = $convert.base64Decode(
    'ChlVcHNlcnRNYWlsVGVtcGxhdGVSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZBIZCghwYXJ0eV9pZB'
    'gCIAEoCVIHcGFydHlJZBISCgRuYW1lGAMgASgJUgRuYW1lEhgKB3N1YmplY3QYBCABKAlSB3N1'
    'YmplY3QSEgoEYm9keRgFIAEoCVIEYm9keRIOCgJ0bxgGIAMoCVICdG8SDgoCY2MYByADKAlSAm'
    'NjEhAKA2JjYxgIIAMoCVIDYmNj');

@$core.Deprecated('Use upsertMailTemplateResponseDescriptor instead')
const UpsertMailTemplateResponse$json = {
  '1': 'UpsertMailTemplateResponse',
  '2': [
    {
      '1': 'template',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.MailTemplate',
      '10': 'template'
    },
  ],
};

/// Descriptor for `UpsertMailTemplateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertMailTemplateResponseDescriptor =
    $convert.base64Decode(
        'ChpVcHNlcnRNYWlsVGVtcGxhdGVSZXNwb25zZRI4Cgh0ZW1wbGF0ZRgBIAEoCzIcLnRiZC5maW'
        '5hbmNlLnYxLk1haWxUZW1wbGF0ZVIIdGVtcGxhdGU=');

@$core.Deprecated('Use deleteMailTemplateRequestDescriptor instead')
const DeleteMailTemplateRequest$json = {
  '1': 'DeleteMailTemplateRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteMailTemplateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteMailTemplateRequestDescriptor =
    $convert.base64Decode(
        'ChlEZWxldGVNYWlsVGVtcGxhdGVSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use deleteMailTemplateResponseDescriptor instead')
const DeleteMailTemplateResponse$json = {
  '1': 'DeleteMailTemplateResponse',
};

/// Descriptor for `DeleteMailTemplateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteMailTemplateResponseDescriptor =
    $convert.base64Decode('ChpEZWxldGVNYWlsVGVtcGxhdGVSZXNwb25zZQ==');

@$core.Deprecated('Use mailDocumentDescriptor instead')
const MailDocument$json = {
  '1': 'MailDocument',
  '2': [
    {'1': 'document_id', '3': 1, '4': 1, '5': 9, '10': 'documentId'},
    {'1': 'filename', '3': 2, '4': 1, '5': 9, '10': 'filename'},
    {'1': 'content_type', '3': 3, '4': 1, '5': 9, '10': 'contentType'},
    {'1': 'size_bytes', '3': 4, '4': 1, '5': 3, '10': 'sizeBytes'},
  ],
};

/// Descriptor for `MailDocument`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mailDocumentDescriptor = $convert.base64Decode(
    'CgxNYWlsRG9jdW1lbnQSHwoLZG9jdW1lbnRfaWQYASABKAlSCmRvY3VtZW50SWQSGgoIZmlsZW'
    '5hbWUYAiABKAlSCGZpbGVuYW1lEiEKDGNvbnRlbnRfdHlwZRgDIAEoCVILY29udGVudFR5cGUS'
    'HQoKc2l6ZV9ieXRlcxgEIAEoA1IJc2l6ZUJ5dGVz');

@$core.Deprecated('Use mailDescriptor instead')
const Mail$json = {
  '1': 'Mail',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'connector_id', '3': 3, '4': 1, '5': 9, '10': 'connectorId'},
    {'1': 'direction', '3': 4, '4': 1, '5': 9, '10': 'direction'},
    {'1': 'from', '3': 5, '4': 1, '5': 9, '10': 'from'},
    {'1': 'to', '3': 6, '4': 3, '5': 9, '10': 'to'},
    {'1': 'cc', '3': 7, '4': 3, '5': 9, '10': 'cc'},
    {'1': 'bcc', '3': 8, '4': 3, '5': 9, '10': 'bcc'},
    {'1': 'subject', '3': 9, '4': 1, '5': 9, '10': 'subject'},
    {'1': 'body', '3': 10, '4': 1, '5': 9, '10': 'body'},
    {'1': 'status', '3': 11, '4': 1, '5': 9, '10': 'status'},
    {'1': 'error', '3': 12, '4': 1, '5': 9, '10': 'error'},
    {'1': 'sent_at', '3': 13, '4': 1, '5': 9, '10': 'sentAt'},
    {'1': 'received_at', '3': 14, '4': 1, '5': 9, '10': 'receivedAt'},
    {'1': 'template_id', '3': 15, '4': 1, '5': 9, '10': 'templateId'},
    {'1': 'parent_id', '3': 16, '4': 1, '5': 9, '10': 'parentId'},
    {'1': 'replies', '3': 17, '4': 1, '5': 13, '10': 'replies'},
    {
      '1': 'documents',
      '3': 18,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.MailDocument',
      '10': 'documents'
    },
    {'1': 'thread_key', '3': 19, '4': 1, '5': 9, '10': 'threadKey'},
    {'1': 'bundle', '3': 20, '4': 1, '5': 9, '10': 'bundle'},
  ],
};

/// Descriptor for `Mail`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mailDescriptor = $convert.base64Decode(
    'CgRNYWlsEg4KAmlkGAEgASgJUgJpZBIZCghwYXJ0eV9pZBgCIAEoCVIHcGFydHlJZBIhCgxjb2'
    '5uZWN0b3JfaWQYAyABKAlSC2Nvbm5lY3RvcklkEhwKCWRpcmVjdGlvbhgEIAEoCVIJZGlyZWN0'
    'aW9uEhIKBGZyb20YBSABKAlSBGZyb20SDgoCdG8YBiADKAlSAnRvEg4KAmNjGAcgAygJUgJjYx'
    'IQCgNiY2MYCCADKAlSA2JjYxIYCgdzdWJqZWN0GAkgASgJUgdzdWJqZWN0EhIKBGJvZHkYCiAB'
    'KAlSBGJvZHkSFgoGc3RhdHVzGAsgASgJUgZzdGF0dXMSFAoFZXJyb3IYDCABKAlSBWVycm9yEh'
    'cKB3NlbnRfYXQYDSABKAlSBnNlbnRBdBIfCgtyZWNlaXZlZF9hdBgOIAEoCVIKcmVjZWl2ZWRB'
    'dBIfCgt0ZW1wbGF0ZV9pZBgPIAEoCVIKdGVtcGxhdGVJZBIbCglwYXJlbnRfaWQYECABKAlSCH'
    'BhcmVudElkEhgKB3JlcGxpZXMYESABKA1SB3JlcGxpZXMSOgoJZG9jdW1lbnRzGBIgAygLMhwu'
    'dGJkLmZpbmFuY2UudjEuTWFpbERvY3VtZW50Uglkb2N1bWVudHMSHQoKdGhyZWFkX2tleRgTIA'
    'EoCVIJdGhyZWFkS2V5EhYKBmJ1bmRsZRgUIAEoCVIGYnVuZGxl');

@$core.Deprecated('Use sendMailRequestDescriptor instead')
const SendMailRequest$json = {
  '1': 'SendMailRequest',
  '2': [
    {'1': 'connector_id', '3': 1, '4': 1, '5': 9, '10': 'connectorId'},
    {'1': 'template_id', '3': 2, '4': 1, '5': 9, '10': 'templateId'},
    {'1': 'to', '3': 3, '4': 3, '5': 9, '10': 'to'},
    {'1': 'cc', '3': 4, '4': 3, '5': 9, '10': 'cc'},
    {'1': 'bcc', '3': 5, '4': 3, '5': 9, '10': 'bcc'},
    {'1': 'subject', '3': 6, '4': 1, '5': 9, '10': 'subject'},
    {'1': 'body', '3': 7, '4': 1, '5': 9, '10': 'body'},
    {'1': 'html', '3': 8, '4': 1, '5': 9, '10': 'html'},
    {
      '1': 'attachment_document_ids',
      '3': 9,
      '4': 3,
      '5': 9,
      '10': 'attachmentDocumentIds'
    },
    {
      '1': 'in_reply_to_mail_id',
      '3': 10,
      '4': 1,
      '5': 9,
      '10': 'inReplyToMailId'
    },
    {
      '1': 'bundle',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.MailBundle',
      '10': 'bundle'
    },
  ],
};

/// Descriptor for `SendMailRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sendMailRequestDescriptor = $convert.base64Decode(
    'Cg9TZW5kTWFpbFJlcXVlc3QSIQoMY29ubmVjdG9yX2lkGAEgASgJUgtjb25uZWN0b3JJZBIfCg'
    't0ZW1wbGF0ZV9pZBgCIAEoCVIKdGVtcGxhdGVJZBIOCgJ0bxgDIAMoCVICdG8SDgoCY2MYBCAD'
    'KAlSAmNjEhAKA2JjYxgFIAMoCVIDYmNjEhgKB3N1YmplY3QYBiABKAlSB3N1YmplY3QSEgoEYm'
    '9keRgHIAEoCVIEYm9keRISCgRodG1sGAggASgJUgRodG1sEjYKF2F0dGFjaG1lbnRfZG9jdW1l'
    'bnRfaWRzGAkgAygJUhVhdHRhY2htZW50RG9jdW1lbnRJZHMSLAoTaW5fcmVwbHlfdG9fbWFpbF'
    '9pZBgKIAEoCVIPaW5SZXBseVRvTWFpbElkEjIKBmJ1bmRsZRgLIAEoCzIaLnRiZC5maW5hbmNl'
    'LnYxLk1haWxCdW5kbGVSBmJ1bmRsZQ==');

@$core.Deprecated('Use mailBundleDescriptor instead')
const MailBundle$json = {
  '1': 'MailBundle',
  '2': [
    {'1': 'filename', '3': 1, '4': 1, '5': 9, '10': 'filename'},
    {
      '1': 'files',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.MailBundleFile',
      '10': 'files'
    },
    {
      '1': 'receipts',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.MailBundleReceipt',
      '10': 'receipts'
    },
  ],
};

/// Descriptor for `MailBundle`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mailBundleDescriptor = $convert.base64Decode(
    'CgpNYWlsQnVuZGxlEhoKCGZpbGVuYW1lGAEgASgJUghmaWxlbmFtZRI0CgVmaWxlcxgCIAMoCz'
    'IeLnRiZC5maW5hbmNlLnYxLk1haWxCdW5kbGVGaWxlUgVmaWxlcxI9CghyZWNlaXB0cxgDIAMo'
    'CzIhLnRiZC5maW5hbmNlLnYxLk1haWxCdW5kbGVSZWNlaXB0UghyZWNlaXB0cw==');

@$core.Deprecated('Use mailBundleFileDescriptor instead')
const MailBundleFile$json = {
  '1': 'MailBundleFile',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'bytes', '3': 2, '4': 1, '5': 12, '10': 'bytes'},
  ],
};

/// Descriptor for `MailBundleFile`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mailBundleFileDescriptor = $convert.base64Decode(
    'Cg5NYWlsQnVuZGxlRmlsZRISCgRuYW1lGAEgASgJUgRuYW1lEhQKBWJ5dGVzGAIgASgMUgVieX'
    'Rlcw==');

@$core.Deprecated('Use mailBundleReceiptDescriptor instead')
const MailBundleReceipt$json = {
  '1': 'MailBundleReceipt',
  '2': [
    {'1': 'document_id', '3': 1, '4': 1, '5': 9, '10': 'documentId'},
    {'1': 'name', '3': 2, '4': 1, '5': 9, '10': 'name'},
  ],
};

/// Descriptor for `MailBundleReceipt`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List mailBundleReceiptDescriptor = $convert.base64Decode(
    'ChFNYWlsQnVuZGxlUmVjZWlwdBIfCgtkb2N1bWVudF9pZBgBIAEoCVIKZG9jdW1lbnRJZBISCg'
    'RuYW1lGAIgASgJUgRuYW1l');

@$core.Deprecated('Use sendMailResponseDescriptor instead')
const SendMailResponse$json = {
  '1': 'SendMailResponse',
  '2': [
    {
      '1': 'mail',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Mail',
      '10': 'mail'
    },
  ],
};

/// Descriptor for `SendMailResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sendMailResponseDescriptor = $convert.base64Decode(
    'ChBTZW5kTWFpbFJlc3BvbnNlEigKBG1haWwYASABKAsyFC50YmQuZmluYW5jZS52MS5NYWlsUg'
    'RtYWls');

@$core.Deprecated('Use listMailRequestDescriptor instead')
const ListMailRequest$json = {
  '1': 'ListMailRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
    {'1': 'connector_id', '3': 2, '4': 1, '5': 9, '10': 'connectorId'},
    {'1': 'direction', '3': 3, '4': 1, '5': 9, '10': 'direction'},
    {'1': 'q', '3': 4, '4': 1, '5': 9, '10': 'q'},
    {'1': 'limit', '3': 5, '4': 1, '5': 13, '10': 'limit'},
    {'1': 'offset', '3': 6, '4': 1, '5': 13, '10': 'offset'},
  ],
};

/// Descriptor for `ListMailRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listMailRequestDescriptor = $convert.base64Decode(
    'Cg9MaXN0TWFpbFJlcXVlc3QSGwoJcGFydHlfaWRzGAEgAygJUghwYXJ0eUlkcxIhCgxjb25uZW'
    'N0b3JfaWQYAiABKAlSC2Nvbm5lY3RvcklkEhwKCWRpcmVjdGlvbhgDIAEoCVIJZGlyZWN0aW9u'
    'EgwKAXEYBCABKAlSAXESFAoFbGltaXQYBSABKA1SBWxpbWl0EhYKBm9mZnNldBgGIAEoDVIGb2'
    'Zmc2V0');

@$core.Deprecated('Use listMailResponseDescriptor instead')
const ListMailResponse$json = {
  '1': 'ListMailResponse',
  '2': [
    {
      '1': 'mails',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Mail',
      '10': 'mails'
    },
    {'1': 'total', '3': 2, '4': 1, '5': 13, '10': 'total'},
  ],
};

/// Descriptor for `ListMailResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listMailResponseDescriptor = $convert.base64Decode(
    'ChBMaXN0TWFpbFJlc3BvbnNlEioKBW1haWxzGAEgAygLMhQudGJkLmZpbmFuY2UudjEuTWFpbF'
    'IFbWFpbHMSFAoFdG90YWwYAiABKA1SBXRvdGFs');

@$core.Deprecated('Use getMailRequestDescriptor instead')
const GetMailRequest$json = {
  '1': 'GetMailRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetMailRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getMailRequestDescriptor =
    $convert.base64Decode('Cg5HZXRNYWlsUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use getMailResponseDescriptor instead')
const GetMailResponse$json = {
  '1': 'GetMailResponse',
  '2': [
    {
      '1': 'mail',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Mail',
      '10': 'mail'
    },
    {
      '1': 'thread',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Mail',
      '10': 'thread'
    },
  ],
};

/// Descriptor for `GetMailResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getMailResponseDescriptor = $convert.base64Decode(
    'Cg9HZXRNYWlsUmVzcG9uc2USKAoEbWFpbBgBIAEoCzIULnRiZC5maW5hbmNlLnYxLk1haWxSBG'
    '1haWwSLAoGdGhyZWFkGAIgAygLMhQudGJkLmZpbmFuY2UudjEuTWFpbFIGdGhyZWFk');

@$core.Deprecated('Use connectorRunDescriptor instead')
const ConnectorRun$json = {
  '1': 'ConnectorRun',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'started_at', '3': 2, '4': 1, '5': 9, '10': 'startedAt'},
    {'1': 'finished_at', '3': 3, '4': 1, '5': 9, '10': 'finishedAt'},
    {'1': 'trigger', '3': 4, '4': 1, '5': 9, '10': 'trigger'},
    {'1': 'outcome', '3': 5, '4': 1, '5': 9, '10': 'outcome'},
    {'1': 'found', '3': 6, '4': 1, '5': 13, '10': 'found'},
    {'1': 'stored', '3': 7, '4': 1, '5': 13, '10': 'stored'},
    {'1': 'skipped', '3': 8, '4': 1, '5': 13, '10': 'skipped'},
    {'1': 'error', '3': 9, '4': 1, '5': 9, '10': 'error'},
  ],
};

/// Descriptor for `ConnectorRun`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List connectorRunDescriptor = $convert.base64Decode(
    'CgxDb25uZWN0b3JSdW4SDgoCaWQYASABKAlSAmlkEh0KCnN0YXJ0ZWRfYXQYAiABKAlSCXN0YX'
    'J0ZWRBdBIfCgtmaW5pc2hlZF9hdBgDIAEoCVIKZmluaXNoZWRBdBIYCgd0cmlnZ2VyGAQgASgJ'
    'Ugd0cmlnZ2VyEhgKB291dGNvbWUYBSABKAlSB291dGNvbWUSFAoFZm91bmQYBiABKA1SBWZvdW'
    '5kEhYKBnN0b3JlZBgHIAEoDVIGc3RvcmVkEhgKB3NraXBwZWQYCCABKA1SB3NraXBwZWQSFAoF'
    'ZXJyb3IYCSABKAlSBWVycm9y');

@$core.Deprecated('Use watchConnectorsRequestDescriptor instead')
const WatchConnectorsRequest$json = {
  '1': 'WatchConnectorsRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `WatchConnectorsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchConnectorsRequestDescriptor =
    $convert.base64Decode(
        'ChZXYXRjaENvbm5lY3RvcnNSZXF1ZXN0EhsKCXBhcnR5X2lkcxgBIAMoCVIIcGFydHlJZHM=');

@$core.Deprecated('Use watchConnectorsResponseDescriptor instead')
const WatchConnectorsResponse$json = {
  '1': 'WatchConnectorsResponse',
  '2': [
    {
      '1': 'connector',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Connector',
      '10': 'connector'
    },
    {
      '1': 'run',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.ConnectorRun',
      '10': 'run'
    },
    {'1': 'deleted', '3': 3, '4': 1, '5': 8, '10': 'deleted'},
  ],
};

/// Descriptor for `WatchConnectorsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchConnectorsResponseDescriptor = $convert.base64Decode(
    'ChdXYXRjaENvbm5lY3RvcnNSZXNwb25zZRI3Cgljb25uZWN0b3IYASABKAsyGS50YmQuZmluYW'
    '5jZS52MS5Db25uZWN0b3JSCWNvbm5lY3RvchIuCgNydW4YAiABKAsyHC50YmQuZmluYW5jZS52'
    'MS5Db25uZWN0b3JSdW5SA3J1bhIYCgdkZWxldGVkGAMgASgIUgdkZWxldGVk');

@$core.Deprecated('Use listConnectorRunsRequestDescriptor instead')
const ListConnectorRunsRequest$json = {
  '1': 'ListConnectorRunsRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `ListConnectorRunsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConnectorRunsRequestDescriptor = $convert
    .base64Decode('ChhMaXN0Q29ubmVjdG9yUnVuc1JlcXVlc3QSDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use listConnectorRunsResponseDescriptor instead')
const ListConnectorRunsResponse$json = {
  '1': 'ListConnectorRunsResponse',
  '2': [
    {
      '1': 'runs',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.ConnectorRun',
      '10': 'runs'
    },
  ],
};

/// Descriptor for `ListConnectorRunsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConnectorRunsResponseDescriptor =
    $convert.base64Decode(
        'ChlMaXN0Q29ubmVjdG9yUnVuc1Jlc3BvbnNlEjAKBHJ1bnMYASADKAsyHC50YmQuZmluYW5jZS'
        '52MS5Db25uZWN0b3JSdW5SBHJ1bnM=');

@$core.Deprecated('Use documentDescriptor instead')
const Document$json = {
  '1': 'Document',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'kind', '3': 3, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'filename', '3': 4, '4': 1, '5': 9, '10': 'filename'},
    {'1': 'content_type', '3': 5, '4': 1, '5': 9, '10': 'contentType'},
    {'1': 'size_bytes', '3': 6, '4': 1, '5': 3, '10': 'sizeBytes'},
    {'1': 'sha256', '3': 7, '4': 1, '5': 9, '10': 'sha256'},
    {'1': 'vendor', '3': 8, '4': 1, '5': 9, '10': 'vendor'},
    {'1': 'doc_date', '3': 9, '4': 1, '5': 9, '10': 'docDate'},
    {'1': 'total_minor', '3': 10, '4': 1, '5': 9, '10': 'totalMinor'},
    {'1': 'currency', '3': 11, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'created_at', '3': 12, '4': 1, '5': 9, '10': 'createdAt'},
    {
      '1': 'sources',
      '3': 13,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.DocumentSource',
      '10': 'sources'
    },
    {'1': 'invoice_no', '3': 14, '4': 1, '5': 9, '10': 'invoiceNo'},
    {'1': 'extracted_at', '3': 15, '4': 1, '5': 9, '10': 'extractedAt'},
    {'1': 'declared', '3': 16, '4': 1, '5': 8, '10': 'declared'},
    {
      '1': 'found_by',
      '3': 17,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Document.FoundByEntry',
      '10': 'foundBy'
    },
  ],
  '3': [Document_FoundByEntry$json],
};

@$core.Deprecated('Use documentDescriptor instead')
const Document_FoundByEntry$json = {
  '1': 'FoundByEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `Document`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List documentDescriptor = $convert.base64Decode(
    'CghEb2N1bWVudBIOCgJpZBgBIAEoCVICaWQSGQoIcGFydHlfaWQYAiABKAlSB3BhcnR5SWQSEg'
    'oEa2luZBgDIAEoCVIEa2luZBIaCghmaWxlbmFtZRgEIAEoCVIIZmlsZW5hbWUSIQoMY29udGVu'
    'dF90eXBlGAUgASgJUgtjb250ZW50VHlwZRIdCgpzaXplX2J5dGVzGAYgASgDUglzaXplQnl0ZX'
    'MSFgoGc2hhMjU2GAcgASgJUgZzaGEyNTYSFgoGdmVuZG9yGAggASgJUgZ2ZW5kb3ISGQoIZG9j'
    'X2RhdGUYCSABKAlSB2RvY0RhdGUSHwoLdG90YWxfbWlub3IYCiABKAlSCnRvdGFsTWlub3ISGg'
    'oIY3VycmVuY3kYCyABKAlSCGN1cnJlbmN5Eh0KCmNyZWF0ZWRfYXQYDCABKAlSCWNyZWF0ZWRB'
    'dBI4Cgdzb3VyY2VzGA0gAygLMh4udGJkLmZpbmFuY2UudjEuRG9jdW1lbnRTb3VyY2VSB3NvdX'
    'JjZXMSHQoKaW52b2ljZV9ubxgOIAEoCVIJaW52b2ljZU5vEiEKDGV4dHJhY3RlZF9hdBgPIAEo'
    'CVILZXh0cmFjdGVkQXQSGgoIZGVjbGFyZWQYECABKAhSCGRlY2xhcmVkEkAKCGZvdW5kX2J5GB'
    'EgAygLMiUudGJkLmZpbmFuY2UudjEuRG9jdW1lbnQuRm91bmRCeUVudHJ5Ugdmb3VuZEJ5GjoK'
    'DEZvdW5kQnlFbnRyeRIQCgNrZXkYASABKAlSA2tleRIUCgV2YWx1ZRgCIAEoCVIFdmFsdWU6Aj'
    'gB');

@$core.Deprecated('Use documentSourceDescriptor instead')
const DocumentSource$json = {
  '1': 'DocumentSource',
  '2': [
    {'1': 'connector_id', '3': 1, '4': 1, '5': 9, '10': 'connectorId'},
    {'1': 'external_ref', '3': 2, '4': 1, '5': 9, '10': 'externalRef'},
    {'1': 'subject', '3': 3, '4': 1, '5': 9, '10': 'subject'},
    {'1': 'sender', '3': 4, '4': 1, '5': 9, '10': 'sender'},
    {'1': 'received_at', '3': 5, '4': 1, '5': 9, '10': 'receivedAt'},
  ],
};

/// Descriptor for `DocumentSource`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List documentSourceDescriptor = $convert.base64Decode(
    'Cg5Eb2N1bWVudFNvdXJjZRIhCgxjb25uZWN0b3JfaWQYASABKAlSC2Nvbm5lY3RvcklkEiEKDG'
    'V4dGVybmFsX3JlZhgCIAEoCVILZXh0ZXJuYWxSZWYSGAoHc3ViamVjdBgDIAEoCVIHc3ViamVj'
    'dBIWCgZzZW5kZXIYBCABKAlSBnNlbmRlchIfCgtyZWNlaXZlZF9hdBgFIAEoCVIKcmVjZWl2ZW'
    'RBdA==');

@$core.Deprecated('Use listDocumentsRequestDescriptor instead')
const ListDocumentsRequest$json = {
  '1': 'ListDocumentsRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
    {'1': 'kind', '3': 2, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'limit', '3': 3, '4': 1, '5': 13, '10': 'limit'},
    {'1': 'offset', '3': 4, '4': 1, '5': 13, '10': 'offset'},
    {'1': 'q', '3': 5, '4': 1, '5': 9, '10': 'q'},
    {'1': 'from', '3': 6, '4': 1, '5': 9, '10': 'from'},
    {'1': 'to', '3': 7, '4': 1, '5': 9, '10': 'to'},
    {'1': 'vendor', '3': 8, '4': 1, '5': 9, '10': 'vendor'},
  ],
};

/// Descriptor for `ListDocumentsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listDocumentsRequestDescriptor = $convert.base64Decode(
    'ChRMaXN0RG9jdW1lbnRzUmVxdWVzdBIbCglwYXJ0eV9pZHMYASADKAlSCHBhcnR5SWRzEhIKBG'
    'tpbmQYAiABKAlSBGtpbmQSFAoFbGltaXQYAyABKA1SBWxpbWl0EhYKBm9mZnNldBgEIAEoDVIG'
    'b2Zmc2V0EgwKAXEYBSABKAlSAXESEgoEZnJvbRgGIAEoCVIEZnJvbRIOCgJ0bxgHIAEoCVICdG'
    '8SFgoGdmVuZG9yGAggASgJUgZ2ZW5kb3I=');

@$core.Deprecated('Use listDocumentsResponseDescriptor instead')
const ListDocumentsResponse$json = {
  '1': 'ListDocumentsResponse',
  '2': [
    {
      '1': 'documents',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Document',
      '10': 'documents'
    },
    {'1': 'total', '3': 2, '4': 1, '5': 13, '10': 'total'},
    {
      '1': 'vendors',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.VendorCount',
      '10': 'vendors'
    },
  ],
};

/// Descriptor for `ListDocumentsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listDocumentsResponseDescriptor = $convert.base64Decode(
    'ChVMaXN0RG9jdW1lbnRzUmVzcG9uc2USNgoJZG9jdW1lbnRzGAEgAygLMhgudGJkLmZpbmFuY2'
    'UudjEuRG9jdW1lbnRSCWRvY3VtZW50cxIUCgV0b3RhbBgCIAEoDVIFdG90YWwSNQoHdmVuZG9y'
    'cxgDIAMoCzIbLnRiZC5maW5hbmNlLnYxLlZlbmRvckNvdW50Ugd2ZW5kb3Jz');

@$core.Deprecated('Use vendorCountDescriptor instead')
const VendorCount$json = {
  '1': 'VendorCount',
  '2': [
    {'1': 'vendor', '3': 1, '4': 1, '5': 9, '10': 'vendor'},
    {'1': 'count', '3': 2, '4': 1, '5': 13, '10': 'count'},
  ],
};

/// Descriptor for `VendorCount`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List vendorCountDescriptor = $convert.base64Decode(
    'CgtWZW5kb3JDb3VudBIWCgZ2ZW5kb3IYASABKAlSBnZlbmRvchIUCgVjb3VudBgCIAEoDVIFY2'
    '91bnQ=');

@$core.Deprecated('Use monthlyReconciliationRequestDescriptor instead')
const MonthlyReconciliationRequest$json = {
  '1': 'MonthlyReconciliationRequest',
  '2': [
    {'1': 'party_id', '3': 1, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'month', '3': 2, '4': 1, '5': 9, '10': 'month'},
  ],
};

/// Descriptor for `MonthlyReconciliationRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List monthlyReconciliationRequestDescriptor =
    $convert.base64Decode(
        'ChxNb250aGx5UmVjb25jaWxpYXRpb25SZXF1ZXN0EhkKCHBhcnR5X2lkGAEgASgJUgdwYXJ0eU'
        'lkEhQKBW1vbnRoGAIgASgJUgVtb250aA==');

@$core.Deprecated('Use monthlyReconciliationResponseDescriptor instead')
const MonthlyReconciliationResponse$json = {
  '1': 'MonthlyReconciliationResponse',
  '2': [
    {
      '1': 'rows',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.ReconciliationRow',
      '10': 'rows'
    },
    {
      '1': 'summary',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.ReconciliationSummary',
      '10': 'summary'
    },
    {
      '1': 'policies',
      '3': 3,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.CounterpartyPolicy',
      '10': 'policies'
    },
  ],
};

/// Descriptor for `MonthlyReconciliationResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List monthlyReconciliationResponseDescriptor = $convert.base64Decode(
    'Ch1Nb250aGx5UmVjb25jaWxpYXRpb25SZXNwb25zZRI1CgRyb3dzGAEgAygLMiEudGJkLmZpbm'
    'FuY2UudjEuUmVjb25jaWxpYXRpb25Sb3dSBHJvd3MSPwoHc3VtbWFyeRgCIAEoCzIlLnRiZC5m'
    'aW5hbmNlLnYxLlJlY29uY2lsaWF0aW9uU3VtbWFyeVIHc3VtbWFyeRI+Cghwb2xpY2llcxgDIA'
    'MoCzIiLnRiZC5maW5hbmNlLnYxLkNvdW50ZXJwYXJ0eVBvbGljeVIIcG9saWNpZXM=');

@$core.Deprecated('Use reconciliationRowDescriptor instead')
const ReconciliationRow$json = {
  '1': 'ReconciliationRow',
  '2': [
    {
      '1': 'transaction',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Transaction',
      '10': 'transaction'
    },
    {'1': 'need', '3': 2, '4': 1, '5': 9, '10': 'need'},
    {'1': 'need_reason', '3': 3, '4': 1, '5': 9, '10': 'needReason'},
    {'1': 'status', '3': 4, '4': 1, '5': 9, '10': 'status'},
    {
      '1': 'documents',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.LinkedDocument',
      '10': 'documents'
    },
    {
      '1': 'suggestions',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.LinkedDocument',
      '10': 'suggestions'
    },
    {'1': 'policy_id', '3': 7, '4': 1, '5': 9, '10': 'policyId'},
    {
      '1': 'original_amount_minor',
      '3': 8,
      '4': 1,
      '5': 9,
      '10': 'originalAmountMinor'
    },
    {
      '1': 'original_currency',
      '3': 9,
      '4': 1,
      '5': 9,
      '10': 'originalCurrency'
    },
    {
      '1': 'need_why',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Reason',
      '10': 'needWhy'
    },
    {'1': 'note', '3': 11, '4': 1, '5': 9, '10': 'note'},
  ],
};

/// Descriptor for `ReconciliationRow`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reconciliationRowDescriptor = $convert.base64Decode(
    'ChFSZWNvbmNpbGlhdGlvblJvdxI9Cgt0cmFuc2FjdGlvbhgBIAEoCzIbLnRiZC5maW5hbmNlLn'
    'YxLlRyYW5zYWN0aW9uUgt0cmFuc2FjdGlvbhISCgRuZWVkGAIgASgJUgRuZWVkEh8KC25lZWRf'
    'cmVhc29uGAMgASgJUgpuZWVkUmVhc29uEhYKBnN0YXR1cxgEIAEoCVIGc3RhdHVzEjwKCWRvY3'
    'VtZW50cxgFIAMoCzIeLnRiZC5maW5hbmNlLnYxLkxpbmtlZERvY3VtZW50Uglkb2N1bWVudHMS'
    'QAoLc3VnZ2VzdGlvbnMYBiADKAsyHi50YmQuZmluYW5jZS52MS5MaW5rZWREb2N1bWVudFILc3'
    'VnZ2VzdGlvbnMSGwoJcG9saWN5X2lkGAcgASgJUghwb2xpY3lJZBIyChVvcmlnaW5hbF9hbW91'
    'bnRfbWlub3IYCCABKAlSE29yaWdpbmFsQW1vdW50TWlub3ISKwoRb3JpZ2luYWxfY3VycmVuY3'
    'kYCSABKAlSEG9yaWdpbmFsQ3VycmVuY3kSMQoIbmVlZF93aHkYCiABKAsyFi50YmQuZmluYW5j'
    'ZS52MS5SZWFzb25SB25lZWRXaHkSEgoEbm90ZRgLIAEoCVIEbm90ZQ==');

@$core.Deprecated('Use reasonDescriptor instead')
const Reason$json = {
  '1': 'Reason',
  '2': [
    {'1': 'code', '3': 1, '4': 1, '5': 9, '10': 'code'},
    {
      '1': 'args',
      '3': 2,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Reason.ArgsEntry',
      '10': 'args'
    },
  ],
  '3': [Reason_ArgsEntry$json],
};

@$core.Deprecated('Use reasonDescriptor instead')
const Reason_ArgsEntry$json = {
  '1': 'ArgsEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `Reason`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reasonDescriptor = $convert.base64Decode(
    'CgZSZWFzb24SEgoEY29kZRgBIAEoCVIEY29kZRI0CgRhcmdzGAIgAygLMiAudGJkLmZpbmFuY2'
    'UudjEuUmVhc29uLkFyZ3NFbnRyeVIEYXJncxo3CglBcmdzRW50cnkSEAoDa2V5GAEgASgJUgNr'
    'ZXkSFAoFdmFsdWUYAiABKAlSBXZhbHVlOgI4AQ==');

@$core.Deprecated('Use linkedDocumentDescriptor instead')
const LinkedDocument$json = {
  '1': 'LinkedDocument',
  '2': [
    {'1': 'document_id', '3': 1, '4': 1, '5': 9, '10': 'documentId'},
    {'1': 'vendor', '3': 2, '4': 1, '5': 9, '10': 'vendor'},
    {'1': 'doc_date', '3': 3, '4': 1, '5': 9, '10': 'docDate'},
    {'1': 'total_minor', '3': 4, '4': 1, '5': 9, '10': 'totalMinor'},
    {'1': 'currency', '3': 5, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'filename', '3': 6, '4': 1, '5': 9, '10': 'filename'},
    {'1': 'invoice_no', '3': 7, '4': 1, '5': 9, '10': 'invoiceNo'},
    {'1': 'source', '3': 8, '4': 1, '5': 9, '10': 'source'},
    {'1': 'confidence', '3': 9, '4': 1, '5': 13, '10': 'confidence'},
    {'1': 'reason', '3': 10, '4': 1, '5': 9, '10': 'reason'},
    {
      '1': 'why',
      '3': 11,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Reason',
      '10': 'why'
    },
  ],
};

/// Descriptor for `LinkedDocument`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List linkedDocumentDescriptor = $convert.base64Decode(
    'Cg5MaW5rZWREb2N1bWVudBIfCgtkb2N1bWVudF9pZBgBIAEoCVIKZG9jdW1lbnRJZBIWCgZ2ZW'
    '5kb3IYAiABKAlSBnZlbmRvchIZCghkb2NfZGF0ZRgDIAEoCVIHZG9jRGF0ZRIfCgt0b3RhbF9t'
    'aW5vchgEIAEoCVIKdG90YWxNaW5vchIaCghjdXJyZW5jeRgFIAEoCVIIY3VycmVuY3kSGgoIZm'
    'lsZW5hbWUYBiABKAlSCGZpbGVuYW1lEh0KCmludm9pY2Vfbm8YByABKAlSCWludm9pY2VObxIW'
    'CgZzb3VyY2UYCCABKAlSBnNvdXJjZRIeCgpjb25maWRlbmNlGAkgASgNUgpjb25maWRlbmNlEh'
    'YKBnJlYXNvbhgKIAEoCVIGcmVhc29uEigKA3doeRgLIAMoCzIWLnRiZC5maW5hbmNlLnYxLlJl'
    'YXNvblIDd2h5');

@$core.Deprecated('Use reconciliationSummaryDescriptor instead')
const ReconciliationSummary$json = {
  '1': 'ReconciliationSummary',
  '2': [
    {'1': 'transactions', '3': 1, '4': 1, '5': 13, '10': 'transactions'},
    {'1': 'eracun', '3': 2, '4': 1, '5': 13, '10': 'eracun'},
    {'1': 'receipt_covered', '3': 3, '4': 1, '5': 13, '10': 'receiptCovered'},
    {'1': 'receipt_missing', '3': 4, '4': 1, '5': 13, '10': 'receiptMissing'},
    {'1': 'none', '3': 5, '4': 1, '5': 13, '10': 'none'},
    {'1': 'personal', '3': 6, '4': 1, '5': 13, '10': 'personal'},
    {'1': 'income', '3': 7, '4': 1, '5': 13, '10': 'income'},
    {'1': 'internal', '3': 8, '4': 1, '5': 13, '10': 'internal'},
    {
      '1': 'missing_minor',
      '3': 9,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.ReconciliationSummary.MissingMinorEntry',
      '10': 'missingMinor'
    },
  ],
  '3': [ReconciliationSummary_MissingMinorEntry$json],
};

@$core.Deprecated('Use reconciliationSummaryDescriptor instead')
const ReconciliationSummary_MissingMinorEntry$json = {
  '1': 'MissingMinorEntry',
  '2': [
    {'1': 'key', '3': 1, '4': 1, '5': 9, '10': 'key'},
    {'1': 'value', '3': 2, '4': 1, '5': 9, '10': 'value'},
  ],
  '7': {'7': true},
};

/// Descriptor for `ReconciliationSummary`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List reconciliationSummaryDescriptor = $convert.base64Decode(
    'ChVSZWNvbmNpbGlhdGlvblN1bW1hcnkSIgoMdHJhbnNhY3Rpb25zGAEgASgNUgx0cmFuc2FjdG'
    'lvbnMSFgoGZXJhY3VuGAIgASgNUgZlcmFjdW4SJwoPcmVjZWlwdF9jb3ZlcmVkGAMgASgNUg5y'
    'ZWNlaXB0Q292ZXJlZBInCg9yZWNlaXB0X21pc3NpbmcYBCABKA1SDnJlY2VpcHRNaXNzaW5nEh'
    'IKBG5vbmUYBSABKA1SBG5vbmUSGgoIcGVyc29uYWwYBiABKA1SCHBlcnNvbmFsEhYKBmluY29t'
    'ZRgHIAEoDVIGaW5jb21lEhoKCGludGVybmFsGAggASgNUghpbnRlcm5hbBJcCg1taXNzaW5nX2'
    '1pbm9yGAkgAygLMjcudGJkLmZpbmFuY2UudjEuUmVjb25jaWxpYXRpb25TdW1tYXJ5Lk1pc3Np'
    'bmdNaW5vckVudHJ5UgxtaXNzaW5nTWlub3IaPwoRTWlzc2luZ01pbm9yRW50cnkSEAoDa2V5GA'
    'EgASgJUgNrZXkSFAoFdmFsdWUYAiABKAlSBXZhbHVlOgI4AQ==');

@$core.Deprecated('Use counterpartyPolicyDescriptor instead')
const CounterpartyPolicy$json = {
  '1': 'CounterpartyPolicy',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'match', '3': 3, '4': 1, '5': 9, '10': 'match'},
    {'1': 'exact', '3': 4, '4': 1, '5': 8, '10': 'exact'},
    {'1': 'policy', '3': 5, '4': 1, '5': 9, '10': 'policy'},
    {'1': 'note', '3': 6, '4': 1, '5': 9, '10': 'note'},
  ],
};

/// Descriptor for `CounterpartyPolicy`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List counterpartyPolicyDescriptor = $convert.base64Decode(
    'ChJDb3VudGVycGFydHlQb2xpY3kSDgoCaWQYASABKAlSAmlkEhkKCHBhcnR5X2lkGAIgASgJUg'
    'dwYXJ0eUlkEhQKBW1hdGNoGAMgASgJUgVtYXRjaBIUCgVleGFjdBgEIAEoCFIFZXhhY3QSFgoG'
    'cG9saWN5GAUgASgJUgZwb2xpY3kSEgoEbm90ZRgGIAEoCVIEbm90ZQ==');

@$core.Deprecated('Use linkDocumentRequestDescriptor instead')
const LinkDocumentRequest$json = {
  '1': 'LinkDocumentRequest',
  '2': [
    {'1': 'transaction_id', '3': 1, '4': 1, '5': 9, '10': 'transactionId'},
    {'1': 'document_id', '3': 2, '4': 1, '5': 9, '10': 'documentId'},
    {'1': 'force', '3': 3, '4': 1, '5': 8, '10': 'force'},
  ],
};

/// Descriptor for `LinkDocumentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List linkDocumentRequestDescriptor = $convert.base64Decode(
    'ChNMaW5rRG9jdW1lbnRSZXF1ZXN0EiUKDnRyYW5zYWN0aW9uX2lkGAEgASgJUg10cmFuc2FjdG'
    'lvbklkEh8KC2RvY3VtZW50X2lkGAIgASgJUgpkb2N1bWVudElkEhQKBWZvcmNlGAMgASgIUgVm'
    'b3JjZQ==');

@$core.Deprecated('Use linkDocumentResponseDescriptor instead')
const LinkDocumentResponse$json = {
  '1': 'LinkDocumentResponse',
  '2': [
    {
      '1': 'row',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.ReconciliationRow',
      '10': 'row'
    },
  ],
};

/// Descriptor for `LinkDocumentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List linkDocumentResponseDescriptor = $convert.base64Decode(
    'ChRMaW5rRG9jdW1lbnRSZXNwb25zZRIzCgNyb3cYASABKAsyIS50YmQuZmluYW5jZS52MS5SZW'
    'NvbmNpbGlhdGlvblJvd1IDcm93');

@$core.Deprecated('Use unlinkDocumentRequestDescriptor instead')
const UnlinkDocumentRequest$json = {
  '1': 'UnlinkDocumentRequest',
  '2': [
    {'1': 'transaction_id', '3': 1, '4': 1, '5': 9, '10': 'transactionId'},
    {'1': 'document_id', '3': 2, '4': 1, '5': 9, '10': 'documentId'},
  ],
};

/// Descriptor for `UnlinkDocumentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List unlinkDocumentRequestDescriptor = $convert.base64Decode(
    'ChVVbmxpbmtEb2N1bWVudFJlcXVlc3QSJQoOdHJhbnNhY3Rpb25faWQYASABKAlSDXRyYW5zYW'
    'N0aW9uSWQSHwoLZG9jdW1lbnRfaWQYAiABKAlSCmRvY3VtZW50SWQ=');

@$core.Deprecated('Use unlinkDocumentResponseDescriptor instead')
const UnlinkDocumentResponse$json = {
  '1': 'UnlinkDocumentResponse',
  '2': [
    {
      '1': 'row',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.ReconciliationRow',
      '10': 'row'
    },
  ],
};

/// Descriptor for `UnlinkDocumentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List unlinkDocumentResponseDescriptor =
    $convert.base64Decode(
        'ChZVbmxpbmtEb2N1bWVudFJlc3BvbnNlEjMKA3JvdxgBIAEoCzIhLnRiZC5maW5hbmNlLnYxLl'
        'JlY29uY2lsaWF0aW9uUm93UgNyb3c=');

@$core.Deprecated('Use setTransactionNoteRequestDescriptor instead')
const SetTransactionNoteRequest$json = {
  '1': 'SetTransactionNoteRequest',
  '2': [
    {'1': 'transaction_id', '3': 1, '4': 1, '5': 9, '10': 'transactionId'},
    {'1': 'note', '3': 2, '4': 1, '5': 9, '10': 'note'},
  ],
};

/// Descriptor for `SetTransactionNoteRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setTransactionNoteRequestDescriptor =
    $convert.base64Decode(
        'ChlTZXRUcmFuc2FjdGlvbk5vdGVSZXF1ZXN0EiUKDnRyYW5zYWN0aW9uX2lkGAEgASgJUg10cm'
        'Fuc2FjdGlvbklkEhIKBG5vdGUYAiABKAlSBG5vdGU=');

@$core.Deprecated('Use setTransactionNoteResponseDescriptor instead')
const SetTransactionNoteResponse$json = {
  '1': 'SetTransactionNoteResponse',
  '2': [
    {
      '1': 'row',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.ReconciliationRow',
      '10': 'row'
    },
  ],
};

/// Descriptor for `SetTransactionNoteResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setTransactionNoteResponseDescriptor =
    $convert.base64Decode(
        'ChpTZXRUcmFuc2FjdGlvbk5vdGVSZXNwb25zZRIzCgNyb3cYASABKAsyIS50YmQuZmluYW5jZS'
        '52MS5SZWNvbmNpbGlhdGlvblJvd1IDcm93');

@$core.Deprecated('Use setCounterpartyPolicyRequestDescriptor instead')
const SetCounterpartyPolicyRequest$json = {
  '1': 'SetCounterpartyPolicyRequest',
  '2': [
    {'1': 'party_id', '3': 1, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'match', '3': 2, '4': 1, '5': 9, '10': 'match'},
    {'1': 'exact', '3': 3, '4': 1, '5': 8, '10': 'exact'},
    {'1': 'policy', '3': 4, '4': 1, '5': 9, '10': 'policy'},
    {'1': 'note', '3': 5, '4': 1, '5': 9, '10': 'note'},
  ],
};

/// Descriptor for `SetCounterpartyPolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setCounterpartyPolicyRequestDescriptor =
    $convert.base64Decode(
        'ChxTZXRDb3VudGVycGFydHlQb2xpY3lSZXF1ZXN0EhkKCHBhcnR5X2lkGAEgASgJUgdwYXJ0eU'
        'lkEhQKBW1hdGNoGAIgASgJUgVtYXRjaBIUCgVleGFjdBgDIAEoCFIFZXhhY3QSFgoGcG9saWN5'
        'GAQgASgJUgZwb2xpY3kSEgoEbm90ZRgFIAEoCVIEbm90ZQ==');

@$core.Deprecated('Use setCounterpartyPolicyResponseDescriptor instead')
const SetCounterpartyPolicyResponse$json = {
  '1': 'SetCounterpartyPolicyResponse',
  '2': [
    {
      '1': 'policy',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.CounterpartyPolicy',
      '10': 'policy'
    },
  ],
};

/// Descriptor for `SetCounterpartyPolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setCounterpartyPolicyResponseDescriptor =
    $convert.base64Decode(
        'Ch1TZXRDb3VudGVycGFydHlQb2xpY3lSZXNwb25zZRI6CgZwb2xpY3kYASABKAsyIi50YmQuZm'
        'luYW5jZS52MS5Db3VudGVycGFydHlQb2xpY3lSBnBvbGljeQ==');

@$core.Deprecated('Use deleteCounterpartyPolicyRequestDescriptor instead')
const DeleteCounterpartyPolicyRequest$json = {
  '1': 'DeleteCounterpartyPolicyRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteCounterpartyPolicyRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteCounterpartyPolicyRequestDescriptor =
    $convert.base64Decode(
        'Ch9EZWxldGVDb3VudGVycGFydHlQb2xpY3lSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use deleteCounterpartyPolicyResponseDescriptor instead')
const DeleteCounterpartyPolicyResponse$json = {
  '1': 'DeleteCounterpartyPolicyResponse',
};

/// Descriptor for `DeleteCounterpartyPolicyResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteCounterpartyPolicyResponseDescriptor =
    $convert.base64Decode('CiBEZWxldGVDb3VudGVycGFydHlQb2xpY3lSZXNwb25zZQ==');

@$core.Deprecated('Use updateDocumentRequestDescriptor instead')
const UpdateDocumentRequest$json = {
  '1': 'UpdateDocumentRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'vendor', '3': 2, '4': 1, '5': 9, '10': 'vendor'},
    {'1': 'doc_date', '3': 3, '4': 1, '5': 9, '10': 'docDate'},
    {'1': 'total_minor', '3': 4, '4': 1, '5': 9, '10': 'totalMinor'},
    {'1': 'currency', '3': 5, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'invoice_no', '3': 6, '4': 1, '5': 9, '10': 'invoiceNo'},
    {'1': 'party_id', '3': 7, '4': 1, '5': 9, '10': 'partyId'},
  ],
};

/// Descriptor for `UpdateDocumentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateDocumentRequestDescriptor = $convert.base64Decode(
    'ChVVcGRhdGVEb2N1bWVudFJlcXVlc3QSDgoCaWQYASABKAlSAmlkEhYKBnZlbmRvchgCIAEoCV'
    'IGdmVuZG9yEhkKCGRvY19kYXRlGAMgASgJUgdkb2NEYXRlEh8KC3RvdGFsX21pbm9yGAQgASgJ'
    'Ugp0b3RhbE1pbm9yEhoKCGN1cnJlbmN5GAUgASgJUghjdXJyZW5jeRIdCgppbnZvaWNlX25vGA'
    'YgASgJUglpbnZvaWNlTm8SGQoIcGFydHlfaWQYByABKAlSB3BhcnR5SWQ=');

@$core.Deprecated('Use updateDocumentResponseDescriptor instead')
const UpdateDocumentResponse$json = {
  '1': 'UpdateDocumentResponse',
  '2': [
    {
      '1': 'document',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Document',
      '10': 'document'
    },
  ],
};

/// Descriptor for `UpdateDocumentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateDocumentResponseDescriptor =
    $convert.base64Decode(
        'ChZVcGRhdGVEb2N1bWVudFJlc3BvbnNlEjQKCGRvY3VtZW50GAEgASgLMhgudGJkLmZpbmFuY2'
        'UudjEuRG9jdW1lbnRSCGRvY3VtZW50');

@$core.Deprecated('Use extractDocumentRequestDescriptor instead')
const ExtractDocumentRequest$json = {
  '1': 'ExtractDocumentRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `ExtractDocumentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List extractDocumentRequestDescriptor = $convert
    .base64Decode('ChZFeHRyYWN0RG9jdW1lbnRSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use extractDocumentResponseDescriptor instead')
const ExtractDocumentResponse$json = {
  '1': 'ExtractDocumentResponse',
  '2': [
    {
      '1': 'document',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Document',
      '10': 'document'
    },
  ],
};

/// Descriptor for `ExtractDocumentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List extractDocumentResponseDescriptor =
    $convert.base64Decode(
        'ChdFeHRyYWN0RG9jdW1lbnRSZXNwb25zZRI0Cghkb2N1bWVudBgBIAEoCzIYLnRiZC5maW5hbm'
        'NlLnYxLkRvY3VtZW50Ughkb2N1bWVudA==');

@$core.Deprecated('Use getDocumentRequestDescriptor instead')
const GetDocumentRequest$json = {
  '1': 'GetDocumentRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetDocumentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getDocumentRequestDescriptor =
    $convert.base64Decode('ChJHZXREb2N1bWVudFJlcXVlc3QSDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use getDocumentResponseDescriptor instead')
const GetDocumentResponse$json = {
  '1': 'GetDocumentResponse',
  '2': [
    {
      '1': 'document',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Document',
      '10': 'document'
    },
    {'1': 'bytes', '3': 2, '4': 1, '5': 12, '10': 'bytes'},
  ],
};

/// Descriptor for `GetDocumentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getDocumentResponseDescriptor = $convert.base64Decode(
    'ChNHZXREb2N1bWVudFJlc3BvbnNlEjQKCGRvY3VtZW50GAEgASgLMhgudGJkLmZpbmFuY2Uudj'
    'EuRG9jdW1lbnRSCGRvY3VtZW50EhQKBWJ5dGVzGAIgASgMUgVieXRlcw==');

@$core.Deprecated('Use uploadDocumentRequestDescriptor instead')
const UploadDocumentRequest$json = {
  '1': 'UploadDocumentRequest',
  '2': [
    {'1': 'party_id', '3': 1, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'filename', '3': 2, '4': 1, '5': 9, '10': 'filename'},
    {'1': 'content_type', '3': 3, '4': 1, '5': 9, '10': 'contentType'},
    {'1': 'bytes', '3': 4, '4': 1, '5': 12, '10': 'bytes'},
  ],
};

/// Descriptor for `UploadDocumentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List uploadDocumentRequestDescriptor = $convert.base64Decode(
    'ChVVcGxvYWREb2N1bWVudFJlcXVlc3QSGQoIcGFydHlfaWQYASABKAlSB3BhcnR5SWQSGgoIZm'
    'lsZW5hbWUYAiABKAlSCGZpbGVuYW1lEiEKDGNvbnRlbnRfdHlwZRgDIAEoCVILY29udGVudFR5'
    'cGUSFAoFYnl0ZXMYBCABKAxSBWJ5dGVz');

@$core.Deprecated('Use uploadDocumentResponseDescriptor instead')
const UploadDocumentResponse$json = {
  '1': 'UploadDocumentResponse',
  '2': [
    {
      '1': 'document',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Document',
      '10': 'document'
    },
  ],
};

/// Descriptor for `UploadDocumentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List uploadDocumentResponseDescriptor =
    $convert.base64Decode(
        'ChZVcGxvYWREb2N1bWVudFJlc3BvbnNlEjQKCGRvY3VtZW50GAEgASgLMhgudGJkLmZpbmFuY2'
        'UudjEuRG9jdW1lbnRSCGRvY3VtZW50');

@$core.Deprecated('Use lineTemplateDescriptor instead')
const LineTemplate$json = {
  '1': 'LineTemplate',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'client_id', '3': 2, '4': 1, '5': 9, '10': 'clientId'},
    {'1': 'position', '3': 3, '4': 1, '5': 5, '10': 'position'},
    {'1': 'description', '3': 4, '4': 1, '5': 9, '10': 'description'},
    {'1': 'mode', '3': 5, '4': 1, '5': 9, '10': 'mode'},
    {'1': 'quantity_milli', '3': 6, '4': 1, '5': 3, '10': 'quantityMilli'},
    {'1': 'unit_price_minor', '3': 7, '4': 1, '5': 3, '10': 'unitPriceMinor'},
    {'1': 'enabled', '3': 8, '4': 1, '5': 8, '10': 'enabled'},
  ],
};

/// Descriptor for `LineTemplate`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List lineTemplateDescriptor = $convert.base64Decode(
    'CgxMaW5lVGVtcGxhdGUSDgoCaWQYASABKAlSAmlkEhsKCWNsaWVudF9pZBgCIAEoCVIIY2xpZW'
    '50SWQSGgoIcG9zaXRpb24YAyABKAVSCHBvc2l0aW9uEiAKC2Rlc2NyaXB0aW9uGAQgASgJUgtk'
    'ZXNjcmlwdGlvbhISCgRtb2RlGAUgASgJUgRtb2RlEiUKDnF1YW50aXR5X21pbGxpGAYgASgDUg'
    '1xdWFudGl0eU1pbGxpEigKEHVuaXRfcHJpY2VfbWlub3IYByABKANSDnVuaXRQcmljZU1pbm9y'
    'EhgKB2VuYWJsZWQYCCABKAhSB2VuYWJsZWQ=');

@$core.Deprecated('Use listLineTemplatesRequestDescriptor instead')
const ListLineTemplatesRequest$json = {
  '1': 'ListLineTemplatesRequest',
  '2': [
    {'1': 'client_id', '3': 1, '4': 1, '5': 9, '10': 'clientId'},
  ],
};

/// Descriptor for `ListLineTemplatesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listLineTemplatesRequestDescriptor =
    $convert.base64Decode(
        'ChhMaXN0TGluZVRlbXBsYXRlc1JlcXVlc3QSGwoJY2xpZW50X2lkGAEgASgJUghjbGllbnRJZA'
        '==');

@$core.Deprecated('Use listLineTemplatesResponseDescriptor instead')
const ListLineTemplatesResponse$json = {
  '1': 'ListLineTemplatesResponse',
  '2': [
    {
      '1': 'templates',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.LineTemplate',
      '10': 'templates'
    },
  ],
};

/// Descriptor for `ListLineTemplatesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listLineTemplatesResponseDescriptor =
    $convert.base64Decode(
        'ChlMaXN0TGluZVRlbXBsYXRlc1Jlc3BvbnNlEjoKCXRlbXBsYXRlcxgBIAMoCzIcLnRiZC5maW'
        '5hbmNlLnYxLkxpbmVUZW1wbGF0ZVIJdGVtcGxhdGVz');

@$core.Deprecated('Use upsertLineTemplateRequestDescriptor instead')
const UpsertLineTemplateRequest$json = {
  '1': 'UpsertLineTemplateRequest',
  '2': [
    {'1': 'client_id', '3': 1, '4': 1, '5': 9, '10': 'clientId'},
    {
      '1': 'template',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.LineTemplate',
      '10': 'template'
    },
  ],
};

/// Descriptor for `UpsertLineTemplateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertLineTemplateRequestDescriptor = $convert.base64Decode(
    'ChlVcHNlcnRMaW5lVGVtcGxhdGVSZXF1ZXN0EhsKCWNsaWVudF9pZBgBIAEoCVIIY2xpZW50SW'
    'QSOAoIdGVtcGxhdGUYAiABKAsyHC50YmQuZmluYW5jZS52MS5MaW5lVGVtcGxhdGVSCHRlbXBs'
    'YXRl');

@$core.Deprecated('Use upsertLineTemplateResponseDescriptor instead')
const UpsertLineTemplateResponse$json = {
  '1': 'UpsertLineTemplateResponse',
  '2': [
    {
      '1': 'template',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.LineTemplate',
      '10': 'template'
    },
  ],
};

/// Descriptor for `UpsertLineTemplateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertLineTemplateResponseDescriptor =
    $convert.base64Decode(
        'ChpVcHNlcnRMaW5lVGVtcGxhdGVSZXNwb25zZRI4Cgh0ZW1wbGF0ZRgBIAEoCzIcLnRiZC5maW'
        '5hbmNlLnYxLkxpbmVUZW1wbGF0ZVIIdGVtcGxhdGU=');

@$core.Deprecated('Use deleteLineTemplateRequestDescriptor instead')
const DeleteLineTemplateRequest$json = {
  '1': 'DeleteLineTemplateRequest',
  '2': [
    {'1': 'client_id', '3': 1, '4': 1, '5': 9, '10': 'clientId'},
    {'1': 'id', '3': 2, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `DeleteLineTemplateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteLineTemplateRequestDescriptor =
    $convert.base64Decode(
        'ChlEZWxldGVMaW5lVGVtcGxhdGVSZXF1ZXN0EhsKCWNsaWVudF9pZBgBIAEoCVIIY2xpZW50SW'
        'QSDgoCaWQYAiABKAlSAmlk');

@$core.Deprecated('Use deleteLineTemplateResponseDescriptor instead')
const DeleteLineTemplateResponse$json = {
  '1': 'DeleteLineTemplateResponse',
};

/// Descriptor for `DeleteLineTemplateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List deleteLineTemplateResponseDescriptor =
    $convert.base64Decode('ChpEZWxldGVMaW5lVGVtcGxhdGVSZXNwb25zZQ==');

@$core.Deprecated('Use issuerProfileDescriptor instead')
const IssuerProfile$json = {
  '1': 'IssuerProfile',
  '2': [
    {'1': 'party_id', '3': 1, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'legal_name', '3': 2, '4': 1, '5': 9, '10': 'legalName'},
    {'1': 'address_lines', '3': 3, '4': 3, '5': 9, '10': 'addressLines'},
    {'1': 'oib', '3': 4, '4': 1, '5': 9, '10': 'oib'},
    {'1': 'vat_id', '3': 5, '4': 1, '5': 9, '10': 'vatId'},
    {'1': 'iban', '3': 6, '4': 1, '5': 9, '10': 'iban'},
    {'1': 'swift', '3': 7, '4': 1, '5': 9, '10': 'swift'},
    {'1': 'bank_name', '3': 8, '4': 1, '5': 9, '10': 'bankName'},
    {'1': 'court', '3': 9, '4': 1, '5': 9, '10': 'court'},
    {'1': 'registration_no', '3': 10, '4': 1, '5': 9, '10': 'registrationNo'},
    {'1': 'share_capital', '3': 11, '4': 1, '5': 9, '10': 'shareCapital'},
    {'1': 'board_member', '3': 12, '4': 1, '5': 9, '10': 'boardMember'},
    {'1': 'issued_by', '3': 13, '4': 1, '5': 9, '10': 'issuedBy'},
    {'1': 'place_of_issue', '3': 14, '4': 1, '5': 9, '10': 'placeOfIssue'},
    {'1': 'operator_id', '3': 15, '4': 1, '5': 9, '10': 'operatorId'},
    {'1': 'premises', '3': 16, '4': 1, '5': 9, '10': 'premises'},
    {'1': 'device', '3': 17, '4': 1, '5': 9, '10': 'device'},
    {'1': 'due_days', '3': 18, '4': 1, '5': 5, '10': 'dueDays'},
  ],
};

/// Descriptor for `IssuerProfile`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List issuerProfileDescriptor = $convert.base64Decode(
    'Cg1Jc3N1ZXJQcm9maWxlEhkKCHBhcnR5X2lkGAEgASgJUgdwYXJ0eUlkEh0KCmxlZ2FsX25hbW'
    'UYAiABKAlSCWxlZ2FsTmFtZRIjCg1hZGRyZXNzX2xpbmVzGAMgAygJUgxhZGRyZXNzTGluZXMS'
    'EAoDb2liGAQgASgJUgNvaWISFQoGdmF0X2lkGAUgASgJUgV2YXRJZBISCgRpYmFuGAYgASgJUg'
    'RpYmFuEhQKBXN3aWZ0GAcgASgJUgVzd2lmdBIbCgliYW5rX25hbWUYCCABKAlSCGJhbmtOYW1l'
    'EhQKBWNvdXJ0GAkgASgJUgVjb3VydBInCg9yZWdpc3RyYXRpb25fbm8YCiABKAlSDnJlZ2lzdH'
    'JhdGlvbk5vEiMKDXNoYXJlX2NhcGl0YWwYCyABKAlSDHNoYXJlQ2FwaXRhbBIhCgxib2FyZF9t'
    'ZW1iZXIYDCABKAlSC2JvYXJkTWVtYmVyEhsKCWlzc3VlZF9ieRgNIAEoCVIIaXNzdWVkQnkSJA'
    'oOcGxhY2Vfb2ZfaXNzdWUYDiABKAlSDHBsYWNlT2ZJc3N1ZRIfCgtvcGVyYXRvcl9pZBgPIAEo'
    'CVIKb3BlcmF0b3JJZBIaCghwcmVtaXNlcxgQIAEoCVIIcHJlbWlzZXMSFgoGZGV2aWNlGBEgAS'
    'gJUgZkZXZpY2USGQoIZHVlX2RheXMYEiABKAVSB2R1ZURheXM=');

@$core.Deprecated('Use getIssuerRequestDescriptor instead')
const GetIssuerRequest$json = {
  '1': 'GetIssuerRequest',
  '2': [
    {'1': 'party_id', '3': 1, '4': 1, '5': 9, '10': 'partyId'},
  ],
};

/// Descriptor for `GetIssuerRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getIssuerRequestDescriptor = $convert.base64Decode(
    'ChBHZXRJc3N1ZXJSZXF1ZXN0EhkKCHBhcnR5X2lkGAEgASgJUgdwYXJ0eUlk');

@$core.Deprecated('Use getIssuerResponseDescriptor instead')
const GetIssuerResponse$json = {
  '1': 'GetIssuerResponse',
  '2': [
    {
      '1': 'issuer',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.IssuerProfile',
      '10': 'issuer'
    },
  ],
};

/// Descriptor for `GetIssuerResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getIssuerResponseDescriptor = $convert.base64Decode(
    'ChFHZXRJc3N1ZXJSZXNwb25zZRI1CgZpc3N1ZXIYASABKAsyHS50YmQuZmluYW5jZS52MS5Jc3'
    'N1ZXJQcm9maWxlUgZpc3N1ZXI=');

@$core.Deprecated('Use upsertIssuerRequestDescriptor instead')
const UpsertIssuerRequest$json = {
  '1': 'UpsertIssuerRequest',
  '2': [
    {
      '1': 'issuer',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.IssuerProfile',
      '10': 'issuer'
    },
  ],
};

/// Descriptor for `UpsertIssuerRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertIssuerRequestDescriptor = $convert.base64Decode(
    'ChNVcHNlcnRJc3N1ZXJSZXF1ZXN0EjUKBmlzc3VlchgBIAEoCzIdLnRiZC5maW5hbmNlLnYxLk'
    'lzc3VlclByb2ZpbGVSBmlzc3Vlcg==');

@$core.Deprecated('Use upsertIssuerResponseDescriptor instead')
const UpsertIssuerResponse$json = {
  '1': 'UpsertIssuerResponse',
  '2': [
    {
      '1': 'issuer',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.IssuerProfile',
      '10': 'issuer'
    },
  ],
};

/// Descriptor for `UpsertIssuerResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertIssuerResponseDescriptor = $convert.base64Decode(
    'ChRVcHNlcnRJc3N1ZXJSZXNwb25zZRI1CgZpc3N1ZXIYASABKAsyHS50YmQuZmluYW5jZS52MS'
    '5Jc3N1ZXJQcm9maWxlUgZpc3N1ZXI=');

@$core.Deprecated('Use clientProfileDescriptor instead')
const ClientProfile$json = {
  '1': 'ClientProfile',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
    {'1': 'address_lines', '3': 4, '4': 3, '5': 9, '10': 'addressLines'},
    {'1': 'country_code', '3': 5, '4': 1, '5': 9, '10': 'countryCode'},
    {'1': 'tax_id', '3': 6, '4': 1, '5': 9, '10': 'taxId'},
    {'1': 'vat_treatment', '3': 7, '4': 1, '5': 9, '10': 'vatTreatment'},
    {'1': 'recipients', '3': 8, '4': 3, '5': 9, '10': 'recipients'},
    {'1': 'currency', '3': 9, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'archived', '3': 10, '4': 1, '5': 8, '10': 'archived'},
  ],
};

/// Descriptor for `ClientProfile`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List clientProfileDescriptor = $convert.base64Decode(
    'Cg1DbGllbnRQcm9maWxlEg4KAmlkGAEgASgJUgJpZBIZCghwYXJ0eV9pZBgCIAEoCVIHcGFydH'
    'lJZBISCgRuYW1lGAMgASgJUgRuYW1lEiMKDWFkZHJlc3NfbGluZXMYBCADKAlSDGFkZHJlc3NM'
    'aW5lcxIhCgxjb3VudHJ5X2NvZGUYBSABKAlSC2NvdW50cnlDb2RlEhUKBnRheF9pZBgGIAEoCV'
    'IFdGF4SWQSIwoNdmF0X3RyZWF0bWVudBgHIAEoCVIMdmF0VHJlYXRtZW50Eh4KCnJlY2lwaWVu'
    'dHMYCCADKAlSCnJlY2lwaWVudHMSGgoIY3VycmVuY3kYCSABKAlSCGN1cnJlbmN5EhoKCGFyY2'
    'hpdmVkGAogASgIUghhcmNoaXZlZA==');

@$core.Deprecated('Use listClientsRequestDescriptor instead')
const ListClientsRequest$json = {
  '1': 'ListClientsRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `ListClientsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listClientsRequestDescriptor =
    $convert.base64Decode(
        'ChJMaXN0Q2xpZW50c1JlcXVlc3QSGwoJcGFydHlfaWRzGAEgAygJUghwYXJ0eUlkcw==');

@$core.Deprecated('Use listClientsResponseDescriptor instead')
const ListClientsResponse$json = {
  '1': 'ListClientsResponse',
  '2': [
    {
      '1': 'clients',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.ClientProfile',
      '10': 'clients'
    },
  ],
};

/// Descriptor for `ListClientsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listClientsResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0Q2xpZW50c1Jlc3BvbnNlEjcKB2NsaWVudHMYASADKAsyHS50YmQuZmluYW5jZS52MS'
    '5DbGllbnRQcm9maWxlUgdjbGllbnRz');

@$core.Deprecated('Use upsertClientRequestDescriptor instead')
const UpsertClientRequest$json = {
  '1': 'UpsertClientRequest',
  '2': [
    {
      '1': 'client',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.ClientProfile',
      '10': 'client'
    },
  ],
};

/// Descriptor for `UpsertClientRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertClientRequestDescriptor = $convert.base64Decode(
    'ChNVcHNlcnRDbGllbnRSZXF1ZXN0EjUKBmNsaWVudBgBIAEoCzIdLnRiZC5maW5hbmNlLnYxLk'
    'NsaWVudFByb2ZpbGVSBmNsaWVudA==');

@$core.Deprecated('Use upsertClientResponseDescriptor instead')
const UpsertClientResponse$json = {
  '1': 'UpsertClientResponse',
  '2': [
    {
      '1': 'client',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.ClientProfile',
      '10': 'client'
    },
  ],
};

/// Descriptor for `UpsertClientResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertClientResponseDescriptor = $convert.base64Decode(
    'ChRVcHNlcnRDbGllbnRSZXNwb25zZRI1CgZjbGllbnQYASABKAsyHS50YmQuZmluYW5jZS52MS'
    '5DbGllbnRQcm9maWxlUgZjbGllbnQ=');

@$core.Deprecated('Use invoiceLineDescriptor instead')
const InvoiceLine$json = {
  '1': 'InvoiceLine',
  '2': [
    {'1': 'position', '3': 1, '4': 1, '5': 5, '10': 'position'},
    {'1': 'description', '3': 2, '4': 1, '5': 9, '10': 'description'},
    {'1': 'quantity_milli', '3': 3, '4': 1, '5': 3, '10': 'quantityMilli'},
    {'1': 'unit_price_minor', '3': 4, '4': 1, '5': 3, '10': 'unitPriceMinor'},
    {'1': 'amount_minor', '3': 5, '4': 1, '5': 3, '10': 'amountMinor'},
    {'1': 'template_id', '3': 6, '4': 1, '5': 9, '10': 'templateId'},
  ],
};

/// Descriptor for `InvoiceLine`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List invoiceLineDescriptor = $convert.base64Decode(
    'CgtJbnZvaWNlTGluZRIaCghwb3NpdGlvbhgBIAEoBVIIcG9zaXRpb24SIAoLZGVzY3JpcHRpb2'
    '4YAiABKAlSC2Rlc2NyaXB0aW9uEiUKDnF1YW50aXR5X21pbGxpGAMgASgDUg1xdWFudGl0eU1p'
    'bGxpEigKEHVuaXRfcHJpY2VfbWlub3IYBCABKANSDnVuaXRQcmljZU1pbm9yEiEKDGFtb3VudF'
    '9taW5vchgFIAEoA1ILYW1vdW50TWlub3ISHwoLdGVtcGxhdGVfaWQYBiABKAlSCnRlbXBsYXRl'
    'SWQ=');

@$core.Deprecated('Use invoiceDescriptor instead')
const Invoice$json = {
  '1': 'Invoice',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'client_id', '3': 3, '4': 1, '5': 9, '10': 'clientId'},
    {'1': 'status', '3': 4, '4': 1, '5': 9, '10': 'status'},
    {'1': 'number', '3': 5, '4': 1, '5': 9, '10': 'number'},
    {'1': 'year', '3': 6, '4': 1, '5': 5, '10': 'year'},
    {'1': 'issued_at', '3': 7, '4': 1, '5': 9, '10': 'issuedAt'},
    {'1': 'delivery_date', '3': 8, '4': 1, '5': 9, '10': 'deliveryDate'},
    {'1': 'due_date', '3': 9, '4': 1, '5': 9, '10': 'dueDate'},
    {'1': 'place_of_issue', '3': 10, '4': 1, '5': 9, '10': 'placeOfIssue'},
    {'1': 'currency', '3': 11, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'subtotal_minor', '3': 12, '4': 1, '5': 3, '10': 'subtotalMinor'},
    {'1': 'vat_minor', '3': 13, '4': 1, '5': 3, '10': 'vatMinor'},
    {'1': 'total_minor', '3': 14, '4': 1, '5': 3, '10': 'totalMinor'},
    {'1': 'vat_treatment', '3': 15, '4': 1, '5': 9, '10': 'vatTreatment'},
    {'1': 'vat_note', '3': 16, '4': 1, '5': 9, '10': 'vatNote'},
    {'1': 'note', '3': 17, '4': 1, '5': 9, '10': 'note'},
    {'1': 'content_hash', '3': 18, '4': 1, '5': 9, '10': 'contentHash'},
    {'1': 'approved_at', '3': 19, '4': 1, '5': 9, '10': 'approvedAt'},
    {'1': 'document_id', '3': 20, '4': 1, '5': 9, '10': 'documentId'},
    {'1': 'prefilled_from', '3': 21, '4': 1, '5': 9, '10': 'prefilledFrom'},
    {'1': 'cancelled_at', '3': 22, '4': 1, '5': 9, '10': 'cancelledAt'},
    {'1': 'created_at', '3': 23, '4': 1, '5': 9, '10': 'createdAt'},
    {'1': 'updated_at', '3': 24, '4': 1, '5': 9, '10': 'updatedAt'},
    {
      '1': 'lines',
      '3': 25,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.InvoiceLine',
      '10': 'lines'
    },
  ],
};

/// Descriptor for `Invoice`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List invoiceDescriptor = $convert.base64Decode(
    'CgdJbnZvaWNlEg4KAmlkGAEgASgJUgJpZBIZCghwYXJ0eV9pZBgCIAEoCVIHcGFydHlJZBIbCg'
    'ljbGllbnRfaWQYAyABKAlSCGNsaWVudElkEhYKBnN0YXR1cxgEIAEoCVIGc3RhdHVzEhYKBm51'
    'bWJlchgFIAEoCVIGbnVtYmVyEhIKBHllYXIYBiABKAVSBHllYXISGwoJaXNzdWVkX2F0GAcgAS'
    'gJUghpc3N1ZWRBdBIjCg1kZWxpdmVyeV9kYXRlGAggASgJUgxkZWxpdmVyeURhdGUSGQoIZHVl'
    'X2RhdGUYCSABKAlSB2R1ZURhdGUSJAoOcGxhY2Vfb2ZfaXNzdWUYCiABKAlSDHBsYWNlT2ZJc3'
    'N1ZRIaCghjdXJyZW5jeRgLIAEoCVIIY3VycmVuY3kSJQoOc3VidG90YWxfbWlub3IYDCABKANS'
    'DXN1YnRvdGFsTWlub3ISGwoJdmF0X21pbm9yGA0gASgDUgh2YXRNaW5vchIfCgt0b3RhbF9taW'
    '5vchgOIAEoA1IKdG90YWxNaW5vchIjCg12YXRfdHJlYXRtZW50GA8gASgJUgx2YXRUcmVhdG1l'
    'bnQSGQoIdmF0X25vdGUYECABKAlSB3ZhdE5vdGUSEgoEbm90ZRgRIAEoCVIEbm90ZRIhCgxjb2'
    '50ZW50X2hhc2gYEiABKAlSC2NvbnRlbnRIYXNoEh8KC2FwcHJvdmVkX2F0GBMgASgJUgphcHBy'
    'b3ZlZEF0Eh8KC2RvY3VtZW50X2lkGBQgASgJUgpkb2N1bWVudElkEiUKDnByZWZpbGxlZF9mcm'
    '9tGBUgASgJUg1wcmVmaWxsZWRGcm9tEiEKDGNhbmNlbGxlZF9hdBgWIAEoCVILY2FuY2VsbGVk'
    'QXQSHQoKY3JlYXRlZF9hdBgXIAEoCVIJY3JlYXRlZEF0Eh0KCnVwZGF0ZWRfYXQYGCABKAlSCX'
    'VwZGF0ZWRBdBIxCgVsaW5lcxgZIAMoCzIbLnRiZC5maW5hbmNlLnYxLkludm9pY2VMaW5lUgVs'
    'aW5lcw==');

@$core.Deprecated('Use listInvoicesRequestDescriptor instead')
const ListInvoicesRequest$json = {
  '1': 'ListInvoicesRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `ListInvoicesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listInvoicesRequestDescriptor =
    $convert.base64Decode(
        'ChNMaXN0SW52b2ljZXNSZXF1ZXN0EhsKCXBhcnR5X2lkcxgBIAMoCVIIcGFydHlJZHM=');

@$core.Deprecated('Use listInvoicesResponseDescriptor instead')
const ListInvoicesResponse$json = {
  '1': 'ListInvoicesResponse',
  '2': [
    {
      '1': 'invoices',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Invoice',
      '10': 'invoices'
    },
  ],
};

/// Descriptor for `ListInvoicesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listInvoicesResponseDescriptor = $convert.base64Decode(
    'ChRMaXN0SW52b2ljZXNSZXNwb25zZRIzCghpbnZvaWNlcxgBIAMoCzIXLnRiZC5maW5hbmNlLn'
    'YxLkludm9pY2VSCGludm9pY2Vz');

@$core.Deprecated('Use getInvoiceRequestDescriptor instead')
const GetInvoiceRequest$json = {
  '1': 'GetInvoiceRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetInvoiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getInvoiceRequestDescriptor =
    $convert.base64Decode('ChFHZXRJbnZvaWNlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQ=');

@$core.Deprecated('Use getInvoiceResponseDescriptor instead')
const GetInvoiceResponse$json = {
  '1': 'GetInvoiceResponse',
  '2': [
    {
      '1': 'invoice',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Invoice',
      '10': 'invoice'
    },
  ],
};

/// Descriptor for `GetInvoiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getInvoiceResponseDescriptor = $convert.base64Decode(
    'ChJHZXRJbnZvaWNlUmVzcG9uc2USMQoHaW52b2ljZRgBIAEoCzIXLnRiZC5maW5hbmNlLnYxLk'
    'ludm9pY2VSB2ludm9pY2U=');

@$core.Deprecated('Use createInvoiceRequestDescriptor instead')
const CreateInvoiceRequest$json = {
  '1': 'CreateInvoiceRequest',
  '2': [
    {'1': 'client_id', '3': 1, '4': 1, '5': 9, '10': 'clientId'},
  ],
};

/// Descriptor for `CreateInvoiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createInvoiceRequestDescriptor =
    $convert.base64Decode(
        'ChRDcmVhdGVJbnZvaWNlUmVxdWVzdBIbCgljbGllbnRfaWQYASABKAlSCGNsaWVudElk');

@$core.Deprecated('Use createInvoiceResponseDescriptor instead')
const CreateInvoiceResponse$json = {
  '1': 'CreateInvoiceResponse',
  '2': [
    {
      '1': 'invoice',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Invoice',
      '10': 'invoice'
    },
  ],
};

/// Descriptor for `CreateInvoiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List createInvoiceResponseDescriptor = $convert.base64Decode(
    'ChVDcmVhdGVJbnZvaWNlUmVzcG9uc2USMQoHaW52b2ljZRgBIAEoCzIXLnRiZC5maW5hbmNlLn'
    'YxLkludm9pY2VSB2ludm9pY2U=');

@$core.Deprecated('Use updateInvoiceRequestDescriptor instead')
const UpdateInvoiceRequest$json = {
  '1': 'UpdateInvoiceRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'delivery_date', '3': 2, '4': 1, '5': 9, '10': 'deliveryDate'},
    {'1': 'due_date', '3': 3, '4': 1, '5': 9, '10': 'dueDate'},
    {'1': 'place_of_issue', '3': 4, '4': 1, '5': 9, '10': 'placeOfIssue'},
    {'1': 'note', '3': 5, '4': 1, '5': 9, '10': 'note'},
    {
      '1': 'lines',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.InvoiceLine',
      '10': 'lines'
    },
  ],
};

/// Descriptor for `UpdateInvoiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateInvoiceRequestDescriptor = $convert.base64Decode(
    'ChRVcGRhdGVJbnZvaWNlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQSIwoNZGVsaXZlcnlfZGF0ZR'
    'gCIAEoCVIMZGVsaXZlcnlEYXRlEhkKCGR1ZV9kYXRlGAMgASgJUgdkdWVEYXRlEiQKDnBsYWNl'
    'X29mX2lzc3VlGAQgASgJUgxwbGFjZU9mSXNzdWUSEgoEbm90ZRgFIAEoCVIEbm90ZRIxCgVsaW'
    '5lcxgGIAMoCzIbLnRiZC5maW5hbmNlLnYxLkludm9pY2VMaW5lUgVsaW5lcw==');

@$core.Deprecated('Use updateInvoiceResponseDescriptor instead')
const UpdateInvoiceResponse$json = {
  '1': 'UpdateInvoiceResponse',
  '2': [
    {
      '1': 'invoice',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Invoice',
      '10': 'invoice'
    },
  ],
};

/// Descriptor for `UpdateInvoiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List updateInvoiceResponseDescriptor = $convert.base64Decode(
    'ChVVcGRhdGVJbnZvaWNlUmVzcG9uc2USMQoHaW52b2ljZRgBIAEoCzIXLnRiZC5maW5hbmNlLn'
    'YxLkludm9pY2VSB2ludm9pY2U=');

@$core.Deprecated('Use previewInvoiceRequestDescriptor instead')
const PreviewInvoiceRequest$json = {
  '1': 'PreviewInvoiceRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `PreviewInvoiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List previewInvoiceRequestDescriptor = $convert
    .base64Decode('ChVQcmV2aWV3SW52b2ljZVJlcXVlc3QSDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use previewInvoiceResponseDescriptor instead')
const PreviewInvoiceResponse$json = {
  '1': 'PreviewInvoiceResponse',
  '2': [
    {'1': 'content_hash', '3': 1, '4': 1, '5': 9, '10': 'contentHash'},
    {'1': 'number', '3': 2, '4': 1, '5': 9, '10': 'number'},
    {'1': 'pdf', '3': 3, '4': 1, '5': 12, '10': 'pdf'},
  ],
};

/// Descriptor for `PreviewInvoiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List previewInvoiceResponseDescriptor =
    $convert.base64Decode(
        'ChZQcmV2aWV3SW52b2ljZVJlc3BvbnNlEiEKDGNvbnRlbnRfaGFzaBgBIAEoCVILY29udGVudE'
        'hhc2gSFgoGbnVtYmVyGAIgASgJUgZudW1iZXISEAoDcGRmGAMgASgMUgNwZGY=');

@$core.Deprecated('Use approveInvoiceRequestDescriptor instead')
const ApproveInvoiceRequest$json = {
  '1': 'ApproveInvoiceRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'content_hash', '3': 2, '4': 1, '5': 9, '10': 'contentHash'},
  ],
};

/// Descriptor for `ApproveInvoiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List approveInvoiceRequestDescriptor = $convert.base64Decode(
    'ChVBcHByb3ZlSW52b2ljZVJlcXVlc3QSDgoCaWQYASABKAlSAmlkEiEKDGNvbnRlbnRfaGFzaB'
    'gCIAEoCVILY29udGVudEhhc2g=');

@$core.Deprecated('Use approveInvoiceResponseDescriptor instead')
const ApproveInvoiceResponse$json = {
  '1': 'ApproveInvoiceResponse',
  '2': [
    {
      '1': 'invoice',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Invoice',
      '10': 'invoice'
    },
  ],
};

/// Descriptor for `ApproveInvoiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List approveInvoiceResponseDescriptor =
    $convert.base64Decode(
        'ChZBcHByb3ZlSW52b2ljZVJlc3BvbnNlEjEKB2ludm9pY2UYASABKAsyFy50YmQuZmluYW5jZS'
        '52MS5JbnZvaWNlUgdpbnZvaWNl');

@$core.Deprecated('Use cancelInvoiceRequestDescriptor instead')
const CancelInvoiceRequest$json = {
  '1': 'CancelInvoiceRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'reason', '3': 2, '4': 1, '5': 9, '10': 'reason'},
  ],
};

/// Descriptor for `CancelInvoiceRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List cancelInvoiceRequestDescriptor = $convert.base64Decode(
    'ChRDYW5jZWxJbnZvaWNlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQSFgoGcmVhc29uGAIgASgJUg'
    'ZyZWFzb24=');

@$core.Deprecated('Use cancelInvoiceResponseDescriptor instead')
const CancelInvoiceResponse$json = {
  '1': 'CancelInvoiceResponse',
  '2': [
    {
      '1': 'invoice',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Invoice',
      '10': 'invoice'
    },
  ],
};

/// Descriptor for `CancelInvoiceResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List cancelInvoiceResponseDescriptor = $convert.base64Decode(
    'ChVDYW5jZWxJbnZvaWNlUmVzcG9uc2USMQoHaW52b2ljZRgBIAEoCzIXLnRiZC5maW5hbmNlLn'
    'YxLkludm9pY2VSB2ludm9pY2U=');

@$core.Deprecated('Use getInvoiceDocumentRequestDescriptor instead')
const GetInvoiceDocumentRequest$json = {
  '1': 'GetInvoiceDocumentRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetInvoiceDocumentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getInvoiceDocumentRequestDescriptor =
    $convert.base64Decode(
        'ChlHZXRJbnZvaWNlRG9jdW1lbnRSZXF1ZXN0Eg4KAmlkGAEgASgJUgJpZA==');

@$core.Deprecated('Use getInvoiceDocumentResponseDescriptor instead')
const GetInvoiceDocumentResponse$json = {
  '1': 'GetInvoiceDocumentResponse',
  '2': [
    {'1': 'content_type', '3': 1, '4': 1, '5': 9, '10': 'contentType'},
    {'1': 'pdf', '3': 2, '4': 1, '5': 12, '10': 'pdf'},
    {'1': 'number', '3': 3, '4': 1, '5': 9, '10': 'number'},
  ],
};

/// Descriptor for `GetInvoiceDocumentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getInvoiceDocumentResponseDescriptor =
    $convert.base64Decode(
        'ChpHZXRJbnZvaWNlRG9jdW1lbnRSZXNwb25zZRIhCgxjb250ZW50X3R5cGUYASABKAlSC2Nvbn'
        'RlbnRUeXBlEhAKA3BkZhgCIAEoDFIDcGRmEhYKBm51bWJlchgDIAEoCVIGbnVtYmVy');

@$core.Deprecated('Use monthlySummaryRequestDescriptor instead')
const MonthlySummaryRequest$json = {
  '1': 'MonthlySummaryRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
    {'1': 'include_internal', '3': 2, '4': 1, '5': 8, '10': 'includeInternal'},
    {'1': 'from_month', '3': 3, '4': 1, '5': 9, '10': 'fromMonth'},
  ],
};

/// Descriptor for `MonthlySummaryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List monthlySummaryRequestDescriptor = $convert.base64Decode(
    'ChVNb250aGx5U3VtbWFyeVJlcXVlc3QSGwoJcGFydHlfaWRzGAEgAygJUghwYXJ0eUlkcxIpCh'
    'BpbmNsdWRlX2ludGVybmFsGAIgASgIUg9pbmNsdWRlSW50ZXJuYWwSHQoKZnJvbV9tb250aBgD'
    'IAEoCVIJZnJvbU1vbnRo');

@$core.Deprecated('Use monthlySummaryResponseDescriptor instead')
const MonthlySummaryResponse$json = {
  '1': 'MonthlySummaryResponse',
  '2': [
    {
      '1': 'rows',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.SummaryRow',
      '10': 'rows'
    },
    {'1': 'party_ids', '3': 2, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `MonthlySummaryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List monthlySummaryResponseDescriptor =
    $convert.base64Decode(
        'ChZNb250aGx5U3VtbWFyeVJlc3BvbnNlEi4KBHJvd3MYASADKAsyGi50YmQuZmluYW5jZS52MS'
        '5TdW1tYXJ5Um93UgRyb3dzEhsKCXBhcnR5X2lkcxgCIAMoCVIIcGFydHlJZHM=');

@$core.Deprecated('Use summaryRowDescriptor instead')
const SummaryRow$json = {
  '1': 'SummaryRow',
  '2': [
    {'1': 'month', '3': 1, '4': 1, '5': 9, '10': 'month'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'category_id', '3': 3, '4': 1, '5': 9, '10': 'categoryId'},
    {'1': 'category', '3': 4, '4': 1, '5': 9, '10': 'category'},
    {'1': 'kind', '3': 5, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'currency', '3': 6, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'total_minor', '3': 7, '4': 1, '5': 3, '10': 'totalMinor'},
    {'1': 'count', '3': 8, '4': 1, '5': 13, '10': 'count'},
    {'1': 'internal', '3': 9, '4': 1, '5': 8, '10': 'internal'},
  ],
};

/// Descriptor for `SummaryRow`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List summaryRowDescriptor = $convert.base64Decode(
    'CgpTdW1tYXJ5Um93EhQKBW1vbnRoGAEgASgJUgVtb250aBIZCghwYXJ0eV9pZBgCIAEoCVIHcG'
    'FydHlJZBIfCgtjYXRlZ29yeV9pZBgDIAEoCVIKY2F0ZWdvcnlJZBIaCghjYXRlZ29yeRgEIAEo'
    'CVIIY2F0ZWdvcnkSEgoEa2luZBgFIAEoCVIEa2luZBIaCghjdXJyZW5jeRgGIAEoCVIIY3Vycm'
    'VuY3kSHwoLdG90YWxfbWlub3IYByABKANSCnRvdGFsTWlub3ISFAoFY291bnQYCCABKA1SBWNv'
    'dW50EhoKCGludGVybmFsGAkgASgIUghpbnRlcm5hbA==');

@$core.Deprecated('Use listPartiesRequestDescriptor instead')
const ListPartiesRequest$json = {
  '1': 'ListPartiesRequest',
};

/// Descriptor for `ListPartiesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listPartiesRequestDescriptor =
    $convert.base64Decode('ChJMaXN0UGFydGllc1JlcXVlc3Q=');

@$core.Deprecated('Use listPartiesResponseDescriptor instead')
const ListPartiesResponse$json = {
  '1': 'ListPartiesResponse',
  '2': [
    {
      '1': 'parties',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Party',
      '10': 'parties'
    },
  ],
};

/// Descriptor for `ListPartiesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listPartiesResponseDescriptor = $convert.base64Decode(
    'ChNMaXN0UGFydGllc1Jlc3BvbnNlEi8KB3BhcnRpZXMYASADKAsyFS50YmQuZmluYW5jZS52MS'
    '5QYXJ0eVIHcGFydGllcw==');

@$core.Deprecated('Use partyDescriptor instead')
const Party$json = {
  '1': 'Party',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'kind', '3': 2, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'display_name', '3': 3, '4': 1, '5': 9, '10': 'displayName'},
    {'1': 'capability', '3': 4, '4': 1, '5': 9, '10': 'capability'},
  ],
};

/// Descriptor for `Party`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List partyDescriptor = $convert.base64Decode(
    'CgVQYXJ0eRIOCgJpZBgBIAEoCVICaWQSEgoEa2luZBgCIAEoCVIEa2luZBIhCgxkaXNwbGF5X2'
    '5hbWUYAyABKAlSC2Rpc3BsYXlOYW1lEh4KCmNhcGFiaWxpdHkYBCABKAlSCmNhcGFiaWxpdHk=');

@$core.Deprecated('Use listAccountsRequestDescriptor instead')
const ListAccountsRequest$json = {
  '1': 'ListAccountsRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `ListAccountsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listAccountsRequestDescriptor =
    $convert.base64Decode(
        'ChNMaXN0QWNjb3VudHNSZXF1ZXN0EhsKCXBhcnR5X2lkcxgBIAMoCVIIcGFydHlJZHM=');

@$core.Deprecated('Use listAccountsResponseDescriptor instead')
const ListAccountsResponse$json = {
  '1': 'ListAccountsResponse',
  '2': [
    {
      '1': 'accounts',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Account',
      '10': 'accounts'
    },
  ],
};

/// Descriptor for `ListAccountsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listAccountsResponseDescriptor = $convert.base64Decode(
    'ChRMaXN0QWNjb3VudHNSZXNwb25zZRIzCghhY2NvdW50cxgBIAMoCzIXLnRiZC5maW5hbmNlLn'
    'YxLkFjY291bnRSCGFjY291bnRz');

@$core.Deprecated('Use accountDescriptor instead')
const Account$json = {
  '1': 'Account',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'connection_id', '3': 3, '4': 1, '5': 9, '10': 'connectionId'},
    {'1': 'provider', '3': 4, '4': 1, '5': 9, '10': 'provider'},
    {'1': 'iban', '3': 5, '4': 1, '5': 9, '10': 'iban'},
    {'1': 'currency', '3': 6, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'name', '3': 7, '4': 1, '5': 9, '10': 'name'},
    {'1': 'sync_enabled', '3': 8, '4': 1, '5': 8, '10': 'syncEnabled'},
    {'1': 'last_synced_at', '3': 9, '4': 1, '5': 9, '10': 'lastSyncedAt'},
    {'1': 'last_sync_status', '3': 10, '4': 1, '5': 9, '10': 'lastSyncStatus'},
    {'1': 'last_sync_error', '3': 11, '4': 1, '5': 9, '10': 'lastSyncError'},
    {
      '1': 'last_booked_through',
      '3': 12,
      '4': 1,
      '5': 9,
      '10': 'lastBookedThrough'
    },
    {
      '1': 'sync_backoff_until',
      '3': 13,
      '4': 1,
      '5': 9,
      '10': 'syncBackoffUntil'
    },
    {'1': 'sync_budget_used', '3': 14, '4': 1, '5': 13, '10': 'syncBudgetUsed'},
    {
      '1': 'balances',
      '3': 15,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Balance',
      '10': 'balances'
    },
  ],
};

/// Descriptor for `Account`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List accountDescriptor = $convert.base64Decode(
    'CgdBY2NvdW50Eg4KAmlkGAEgASgJUgJpZBIZCghwYXJ0eV9pZBgCIAEoCVIHcGFydHlJZBIjCg'
    '1jb25uZWN0aW9uX2lkGAMgASgJUgxjb25uZWN0aW9uSWQSGgoIcHJvdmlkZXIYBCABKAlSCHBy'
    'b3ZpZGVyEhIKBGliYW4YBSABKAlSBGliYW4SGgoIY3VycmVuY3kYBiABKAlSCGN1cnJlbmN5Eh'
    'IKBG5hbWUYByABKAlSBG5hbWUSIQoMc3luY19lbmFibGVkGAggASgIUgtzeW5jRW5hYmxlZBIk'
    'Cg5sYXN0X3N5bmNlZF9hdBgJIAEoCVIMbGFzdFN5bmNlZEF0EigKEGxhc3Rfc3luY19zdGF0dX'
    'MYCiABKAlSDmxhc3RTeW5jU3RhdHVzEiYKD2xhc3Rfc3luY19lcnJvchgLIAEoCVINbGFzdFN5'
    'bmNFcnJvchIuChNsYXN0X2Jvb2tlZF90aHJvdWdoGAwgASgJUhFsYXN0Qm9va2VkVGhyb3VnaB'
    'IsChJzeW5jX2JhY2tvZmZfdW50aWwYDSABKAlSEHN5bmNCYWNrb2ZmVW50aWwSKAoQc3luY19i'
    'dWRnZXRfdXNlZBgOIAEoDVIOc3luY0J1ZGdldFVzZWQSMwoIYmFsYW5jZXMYDyADKAsyFy50Ym'
    'QuZmluYW5jZS52MS5CYWxhbmNlUghiYWxhbmNlcw==');

@$core.Deprecated('Use balanceDescriptor instead')
const Balance$json = {
  '1': 'Balance',
  '2': [
    {'1': 'balance_type', '3': 1, '4': 1, '5': 9, '10': 'balanceType'},
    {'1': 'amount_minor', '3': 2, '4': 1, '5': 3, '10': 'amountMinor'},
    {'1': 'currency', '3': 3, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'observed_at', '3': 4, '4': 1, '5': 9, '10': 'observedAt'},
  ],
};

/// Descriptor for `Balance`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List balanceDescriptor = $convert.base64Decode(
    'CgdCYWxhbmNlEiEKDGJhbGFuY2VfdHlwZRgBIAEoCVILYmFsYW5jZVR5cGUSIQoMYW1vdW50X2'
    '1pbm9yGAIgASgDUgthbW91bnRNaW5vchIaCghjdXJyZW5jeRgDIAEoCVIIY3VycmVuY3kSHwoL'
    'b2JzZXJ2ZWRfYXQYBCABKAlSCm9ic2VydmVkQXQ=');

@$core.Deprecated('Use refreshAccountRequestDescriptor instead')
const RefreshAccountRequest$json = {
  '1': 'RefreshAccountRequest',
  '2': [
    {'1': 'account_id', '3': 1, '4': 1, '5': 9, '10': 'accountId'},
  ],
};

/// Descriptor for `RefreshAccountRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List refreshAccountRequestDescriptor = $convert.base64Decode(
    'ChVSZWZyZXNoQWNjb3VudFJlcXVlc3QSHQoKYWNjb3VudF9pZBgBIAEoCVIJYWNjb3VudElk');

@$core.Deprecated('Use refreshAccountResponseDescriptor instead')
const RefreshAccountResponse$json = {
  '1': 'RefreshAccountResponse',
  '2': [
    {'1': 'outcome', '3': 1, '4': 1, '5': 9, '10': 'outcome'},
    {'1': 'skipped', '3': 2, '4': 1, '5': 9, '10': 'skipped'},
    {'1': 'inserted', '3': 3, '4': 1, '5': 13, '10': 'inserted'},
    {'1': 'booked', '3': 4, '4': 1, '5': 13, '10': 'booked'},
    {'1': 'duplicates', '3': 5, '4': 1, '5': 13, '10': 'duplicates'},
    {'1': 'attended', '3': 6, '4': 1, '5': 8, '10': 'attended'},
  ],
};

/// Descriptor for `RefreshAccountResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List refreshAccountResponseDescriptor = $convert.base64Decode(
    'ChZSZWZyZXNoQWNjb3VudFJlc3BvbnNlEhgKB291dGNvbWUYASABKAlSB291dGNvbWUSGAoHc2'
    'tpcHBlZBgCIAEoCVIHc2tpcHBlZBIaCghpbnNlcnRlZBgDIAEoDVIIaW5zZXJ0ZWQSFgoGYm9v'
    'a2VkGAQgASgNUgZib29rZWQSHgoKZHVwbGljYXRlcxgFIAEoDVIKZHVwbGljYXRlcxIaCghhdH'
    'RlbmRlZBgGIAEoCFIIYXR0ZW5kZWQ=');

@$core.Deprecated('Use setAccountSyncRequestDescriptor instead')
const SetAccountSyncRequest$json = {
  '1': 'SetAccountSyncRequest',
  '2': [
    {'1': 'account_id', '3': 1, '4': 1, '5': 9, '10': 'accountId'},
    {'1': 'enabled', '3': 2, '4': 1, '5': 8, '10': 'enabled'},
  ],
};

/// Descriptor for `SetAccountSyncRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setAccountSyncRequestDescriptor = $convert.base64Decode(
    'ChVTZXRBY2NvdW50U3luY1JlcXVlc3QSHQoKYWNjb3VudF9pZBgBIAEoCVIJYWNjb3VudElkEh'
    'gKB2VuYWJsZWQYAiABKAhSB2VuYWJsZWQ=');

@$core.Deprecated('Use setAccountSyncResponseDescriptor instead')
const SetAccountSyncResponse$json = {
  '1': 'SetAccountSyncResponse',
  '2': [
    {
      '1': 'account',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Account',
      '10': 'account'
    },
  ],
};

/// Descriptor for `SetAccountSyncResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List setAccountSyncResponseDescriptor =
    $convert.base64Decode(
        'ChZTZXRBY2NvdW50U3luY1Jlc3BvbnNlEjEKB2FjY291bnQYASABKAsyFy50YmQuZmluYW5jZS'
        '52MS5BY2NvdW50UgdhY2NvdW50');

@$core.Deprecated('Use listCategoriesRequestDescriptor instead')
const ListCategoriesRequest$json = {
  '1': 'ListCategoriesRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `ListCategoriesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listCategoriesRequestDescriptor = $convert.base64Decode(
    'ChVMaXN0Q2F0ZWdvcmllc1JlcXVlc3QSGwoJcGFydHlfaWRzGAEgAygJUghwYXJ0eUlkcw==');

@$core.Deprecated('Use listCategoriesResponseDescriptor instead')
const ListCategoriesResponse$json = {
  '1': 'ListCategoriesResponse',
  '2': [
    {
      '1': 'categories',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Category',
      '10': 'categories'
    },
  ],
};

/// Descriptor for `ListCategoriesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listCategoriesResponseDescriptor =
    $convert.base64Decode(
        'ChZMaXN0Q2F0ZWdvcmllc1Jlc3BvbnNlEjgKCmNhdGVnb3JpZXMYASADKAsyGC50YmQuZmluYW'
        '5jZS52MS5DYXRlZ29yeVIKY2F0ZWdvcmllcw==');

@$core.Deprecated('Use categoryDescriptor instead')
const Category$json = {
  '1': 'Category',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'slug', '3': 3, '4': 1, '5': 9, '10': 'slug'},
    {'1': 'name', '3': 4, '4': 1, '5': 9, '10': 'name'},
    {'1': 'kind', '3': 5, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'deductible', '3': 6, '4': 1, '5': 8, '10': 'deductible'},
    {'1': 'archived', '3': 7, '4': 1, '5': 8, '10': 'archived'},
  ],
};

/// Descriptor for `Category`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List categoryDescriptor = $convert.base64Decode(
    'CghDYXRlZ29yeRIOCgJpZBgBIAEoCVICaWQSGQoIcGFydHlfaWQYAiABKAlSB3BhcnR5SWQSEg'
    'oEc2x1ZxgDIAEoCVIEc2x1ZxISCgRuYW1lGAQgASgJUgRuYW1lEhIKBGtpbmQYBSABKAlSBGtp'
    'bmQSHgoKZGVkdWN0aWJsZRgGIAEoCFIKZGVkdWN0aWJsZRIaCghhcmNoaXZlZBgHIAEoCFIIYX'
    'JjaGl2ZWQ=');

@$core.Deprecated('Use declareCategoryRequestDescriptor instead')
const DeclareCategoryRequest$json = {
  '1': 'DeclareCategoryRequest',
  '2': [
    {'1': 'transaction_id', '3': 1, '4': 1, '5': 9, '10': 'transactionId'},
    {'1': 'category_id', '3': 2, '4': 1, '5': 9, '10': 'categoryId'},
  ],
};

/// Descriptor for `DeclareCategoryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List declareCategoryRequestDescriptor =
    $convert.base64Decode(
        'ChZEZWNsYXJlQ2F0ZWdvcnlSZXF1ZXN0EiUKDnRyYW5zYWN0aW9uX2lkGAEgASgJUg10cmFuc2'
        'FjdGlvbklkEh8KC2NhdGVnb3J5X2lkGAIgASgJUgpjYXRlZ29yeUlk');

@$core.Deprecated('Use declareCategoryResponseDescriptor instead')
const DeclareCategoryResponse$json = {
  '1': 'DeclareCategoryResponse',
  '2': [
    {
      '1': 'transaction',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Transaction',
      '10': 'transaction'
    },
  ],
};

/// Descriptor for `DeclareCategoryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List declareCategoryResponseDescriptor =
    $convert.base64Decode(
        'ChdEZWNsYXJlQ2F0ZWdvcnlSZXNwb25zZRI9Cgt0cmFuc2FjdGlvbhgBIAEoCzIbLnRiZC5maW'
        '5hbmNlLnYxLlRyYW5zYWN0aW9uUgt0cmFuc2FjdGlvbg==');

@$core.Deprecated('Use upsertCategoryRequestDescriptor instead')
const UpsertCategoryRequest$json = {
  '1': 'UpsertCategoryRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'name', '3': 3, '4': 1, '5': 9, '10': 'name'},
    {'1': 'kind', '3': 4, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'deductible', '3': 5, '4': 1, '5': 8, '10': 'deductible'},
    {'1': 'archived', '3': 6, '4': 1, '5': 8, '10': 'archived'},
  ],
};

/// Descriptor for `UpsertCategoryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertCategoryRequestDescriptor = $convert.base64Decode(
    'ChVVcHNlcnRDYXRlZ29yeVJlcXVlc3QSDgoCaWQYASABKAlSAmlkEhkKCHBhcnR5X2lkGAIgAS'
    'gJUgdwYXJ0eUlkEhIKBG5hbWUYAyABKAlSBG5hbWUSEgoEa2luZBgEIAEoCVIEa2luZBIeCgpk'
    'ZWR1Y3RpYmxlGAUgASgIUgpkZWR1Y3RpYmxlEhoKCGFyY2hpdmVkGAYgASgIUghhcmNoaXZlZA'
    '==');

@$core.Deprecated('Use upsertCategoryResponseDescriptor instead')
const UpsertCategoryResponse$json = {
  '1': 'UpsertCategoryResponse',
  '2': [
    {
      '1': 'category',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Category',
      '10': 'category'
    },
    {'1': 'categorised', '3': 2, '4': 1, '5': 13, '10': 'categorised'},
    {'1': 'unmatched', '3': 3, '4': 1, '5': 13, '10': 'unmatched'},
  ],
};

/// Descriptor for `UpsertCategoryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertCategoryResponseDescriptor = $convert.base64Decode(
    'ChZVcHNlcnRDYXRlZ29yeVJlc3BvbnNlEjQKCGNhdGVnb3J5GAEgASgLMhgudGJkLmZpbmFuY2'
    'UudjEuQ2F0ZWdvcnlSCGNhdGVnb3J5EiAKC2NhdGVnb3Jpc2VkGAIgASgNUgtjYXRlZ29yaXNl'
    'ZBIcCgl1bm1hdGNoZWQYAyABKA1SCXVubWF0Y2hlZA==');

@$core.Deprecated('Use listRulesRequestDescriptor instead')
const ListRulesRequest$json = {
  '1': 'ListRulesRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `ListRulesRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRulesRequestDescriptor = $convert.base64Decode(
    'ChBMaXN0UnVsZXNSZXF1ZXN0EhsKCXBhcnR5X2lkcxgBIAMoCVIIcGFydHlJZHM=');

@$core.Deprecated('Use listRulesResponseDescriptor instead')
const ListRulesResponse$json = {
  '1': 'ListRulesResponse',
  '2': [
    {
      '1': 'rules',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Rule',
      '10': 'rules'
    },
  ],
};

/// Descriptor for `ListRulesResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listRulesResponseDescriptor = $convert.base64Decode(
    'ChFMaXN0UnVsZXNSZXNwb25zZRIqCgVydWxlcxgBIAMoCzIULnRiZC5maW5hbmNlLnYxLlJ1bG'
    'VSBXJ1bGVz');

@$core.Deprecated('Use ruleDescriptor instead')
const Rule$json = {
  '1': 'Rule',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'priority', '3': 3, '4': 1, '5': 5, '10': 'priority'},
    {'1': 'name', '3': 4, '4': 1, '5': 9, '10': 'name'},
    {'1': 'category_id', '3': 5, '4': 1, '5': 9, '10': 'categoryId'},
    {
      '1': 'match_counterparty_like',
      '3': 6,
      '4': 1,
      '5': 9,
      '10': 'matchCounterpartyLike'
    },
    {
      '1': 'match_counterparty_iban',
      '3': 7,
      '4': 1,
      '5': 9,
      '10': 'matchCounterpartyIban'
    },
    {
      '1': 'match_remittance_like',
      '3': 8,
      '4': 1,
      '5': 9,
      '10': 'matchRemittanceLike'
    },
    {'1': 'match_currency', '3': 9, '4': 1, '5': 9, '10': 'matchCurrency'},
    {
      '1': 'match_credit_debit',
      '3': 10,
      '4': 1,
      '5': 9,
      '10': 'matchCreditDebit'
    },
    {'1': 'enabled', '3': 11, '4': 1, '5': 8, '10': 'enabled'},
    {'1': 'hits', '3': 12, '4': 1, '5': 3, '10': 'hits'},
  ],
};

/// Descriptor for `Rule`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List ruleDescriptor = $convert.base64Decode(
    'CgRSdWxlEg4KAmlkGAEgASgJUgJpZBIZCghwYXJ0eV9pZBgCIAEoCVIHcGFydHlJZBIaCghwcm'
    'lvcml0eRgDIAEoBVIIcHJpb3JpdHkSEgoEbmFtZRgEIAEoCVIEbmFtZRIfCgtjYXRlZ29yeV9p'
    'ZBgFIAEoCVIKY2F0ZWdvcnlJZBI2ChdtYXRjaF9jb3VudGVycGFydHlfbGlrZRgGIAEoCVIVbW'
    'F0Y2hDb3VudGVycGFydHlMaWtlEjYKF21hdGNoX2NvdW50ZXJwYXJ0eV9pYmFuGAcgASgJUhVt'
    'YXRjaENvdW50ZXJwYXJ0eUliYW4SMgoVbWF0Y2hfcmVtaXR0YW5jZV9saWtlGAggASgJUhNtYX'
    'RjaFJlbWl0dGFuY2VMaWtlEiUKDm1hdGNoX2N1cnJlbmN5GAkgASgJUg1tYXRjaEN1cnJlbmN5'
    'EiwKEm1hdGNoX2NyZWRpdF9kZWJpdBgKIAEoCVIQbWF0Y2hDcmVkaXREZWJpdBIYCgdlbmFibG'
    'VkGAsgASgIUgdlbmFibGVkEhIKBGhpdHMYDCABKANSBGhpdHM=');

@$core.Deprecated('Use upsertRuleRequestDescriptor instead')
const UpsertRuleRequest$json = {
  '1': 'UpsertRuleRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'priority', '3': 3, '4': 1, '5': 5, '10': 'priority'},
    {'1': 'name', '3': 4, '4': 1, '5': 9, '10': 'name'},
    {'1': 'category_id', '3': 5, '4': 1, '5': 9, '10': 'categoryId'},
    {
      '1': 'match_counterparty_like',
      '3': 6,
      '4': 1,
      '5': 9,
      '10': 'matchCounterpartyLike'
    },
    {
      '1': 'match_counterparty_iban',
      '3': 7,
      '4': 1,
      '5': 9,
      '10': 'matchCounterpartyIban'
    },
    {
      '1': 'match_remittance_like',
      '3': 8,
      '4': 1,
      '5': 9,
      '10': 'matchRemittanceLike'
    },
    {'1': 'match_currency', '3': 9, '4': 1, '5': 9, '10': 'matchCurrency'},
    {
      '1': 'match_credit_debit',
      '3': 10,
      '4': 1,
      '5': 9,
      '10': 'matchCreditDebit'
    },
    {'1': 'enabled', '3': 11, '4': 1, '5': 8, '10': 'enabled'},
  ],
};

/// Descriptor for `UpsertRuleRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertRuleRequestDescriptor = $convert.base64Decode(
    'ChFVcHNlcnRSdWxlUmVxdWVzdBIOCgJpZBgBIAEoCVICaWQSGQoIcGFydHlfaWQYAiABKAlSB3'
    'BhcnR5SWQSGgoIcHJpb3JpdHkYAyABKAVSCHByaW9yaXR5EhIKBG5hbWUYBCABKAlSBG5hbWUS'
    'HwoLY2F0ZWdvcnlfaWQYBSABKAlSCmNhdGVnb3J5SWQSNgoXbWF0Y2hfY291bnRlcnBhcnR5X2'
    'xpa2UYBiABKAlSFW1hdGNoQ291bnRlcnBhcnR5TGlrZRI2ChdtYXRjaF9jb3VudGVycGFydHlf'
    'aWJhbhgHIAEoCVIVbWF0Y2hDb3VudGVycGFydHlJYmFuEjIKFW1hdGNoX3JlbWl0dGFuY2VfbG'
    'lrZRgIIAEoCVITbWF0Y2hSZW1pdHRhbmNlTGlrZRIlCg5tYXRjaF9jdXJyZW5jeRgJIAEoCVIN'
    'bWF0Y2hDdXJyZW5jeRIsChJtYXRjaF9jcmVkaXRfZGViaXQYCiABKAlSEG1hdGNoQ3JlZGl0RG'
    'ViaXQSGAoHZW5hYmxlZBgLIAEoCFIHZW5hYmxlZA==');

@$core.Deprecated('Use upsertRuleResponseDescriptor instead')
const UpsertRuleResponse$json = {
  '1': 'UpsertRuleResponse',
  '2': [
    {
      '1': 'rule',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Rule',
      '10': 'rule'
    },
    {'1': 'categorised', '3': 2, '4': 1, '5': 13, '10': 'categorised'},
    {'1': 'unmatched', '3': 3, '4': 1, '5': 13, '10': 'unmatched'},
  ],
};

/// Descriptor for `UpsertRuleResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List upsertRuleResponseDescriptor = $convert.base64Decode(
    'ChJVcHNlcnRSdWxlUmVzcG9uc2USKAoEcnVsZRgBIAEoCzIULnRiZC5maW5hbmNlLnYxLlJ1bG'
    'VSBHJ1bGUSIAoLY2F0ZWdvcmlzZWQYAiABKA1SC2NhdGVnb3Jpc2VkEhwKCXVubWF0Y2hlZBgD'
    'IAEoDVIJdW5tYXRjaGVk');

@$core.Deprecated('Use startConnectionRequestDescriptor instead')
const StartConnectionRequest$json = {
  '1': 'StartConnectionRequest',
  '2': [
    {'1': 'party_id', '3': 1, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'psu_type', '3': 2, '4': 1, '5': 9, '10': 'psuType'},
    {'1': 'aspsp_name', '3': 3, '4': 1, '5': 9, '10': 'aspspName'},
    {'1': 'aspsp_country', '3': 4, '4': 1, '5': 9, '10': 'aspspCountry'},
  ],
};

/// Descriptor for `StartConnectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List startConnectionRequestDescriptor = $convert.base64Decode(
    'ChZTdGFydENvbm5lY3Rpb25SZXF1ZXN0EhkKCHBhcnR5X2lkGAEgASgJUgdwYXJ0eUlkEhkKCH'
    'BzdV90eXBlGAIgASgJUgdwc3VUeXBlEh0KCmFzcHNwX25hbWUYAyABKAlSCWFzcHNwTmFtZRIj'
    'Cg1hc3BzcF9jb3VudHJ5GAQgASgJUgxhc3BzcENvdW50cnk=');

@$core.Deprecated('Use startConnectionResponseDescriptor instead')
const StartConnectionResponse$json = {
  '1': 'StartConnectionResponse',
  '2': [
    {'1': 'connection_id', '3': 1, '4': 1, '5': 9, '10': 'connectionId'},
    {'1': 'url', '3': 2, '4': 1, '5': 9, '10': 'url'},
  ],
};

/// Descriptor for `StartConnectionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List startConnectionResponseDescriptor =
    $convert.base64Decode(
        'ChdTdGFydENvbm5lY3Rpb25SZXNwb25zZRIjCg1jb25uZWN0aW9uX2lkGAEgASgJUgxjb25uZW'
        'N0aW9uSWQSEAoDdXJsGAIgASgJUgN1cmw=');

@$core.Deprecated('Use completeConnectionRequestDescriptor instead')
const CompleteConnectionRequest$json = {
  '1': 'CompleteConnectionRequest',
  '2': [
    {'1': 'state', '3': 1, '4': 1, '5': 9, '10': 'state'},
    {'1': 'code', '3': 2, '4': 1, '5': 9, '10': 'code'},
  ],
};

/// Descriptor for `CompleteConnectionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeConnectionRequestDescriptor =
    $convert.base64Decode(
        'ChlDb21wbGV0ZUNvbm5lY3Rpb25SZXF1ZXN0EhQKBXN0YXRlGAEgASgJUgVzdGF0ZRISCgRjb2'
        'RlGAIgASgJUgRjb2Rl');

@$core.Deprecated('Use completeConnectionResponseDescriptor instead')
const CompleteConnectionResponse$json = {
  '1': 'CompleteConnectionResponse',
  '2': [
    {'1': 'connection_id', '3': 1, '4': 1, '5': 9, '10': 'connectionId'},
    {'1': 'account_ids', '3': 2, '4': 3, '5': 9, '10': 'accountIds'},
  ],
};

/// Descriptor for `CompleteConnectionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List completeConnectionResponseDescriptor =
    $convert.base64Decode(
        'ChpDb21wbGV0ZUNvbm5lY3Rpb25SZXNwb25zZRIjCg1jb25uZWN0aW9uX2lkGAEgASgJUgxjb2'
        '5uZWN0aW9uSWQSHwoLYWNjb3VudF9pZHMYAiADKAlSCmFjY291bnRJZHM=');

@$core.Deprecated('Use listConnectionsRequestDescriptor instead')
const ListConnectionsRequest$json = {
  '1': 'ListConnectionsRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `ListConnectionsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConnectionsRequestDescriptor =
    $convert.base64Decode(
        'ChZMaXN0Q29ubmVjdGlvbnNSZXF1ZXN0EhsKCXBhcnR5X2lkcxgBIAMoCVIIcGFydHlJZHM=');

@$core.Deprecated('Use listConnectionsResponseDescriptor instead')
const ListConnectionsResponse$json = {
  '1': 'ListConnectionsResponse',
  '2': [
    {
      '1': 'connections',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Connection',
      '10': 'connections'
    },
  ],
};

/// Descriptor for `ListConnectionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listConnectionsResponseDescriptor =
    $convert.base64Decode(
        'ChdMaXN0Q29ubmVjdGlvbnNSZXNwb25zZRI8Cgtjb25uZWN0aW9ucxgBIAMoCzIaLnRiZC5maW'
        '5hbmNlLnYxLkNvbm5lY3Rpb25SC2Nvbm5lY3Rpb25z');

@$core.Deprecated('Use connectionDescriptor instead')
const Connection$json = {
  '1': 'Connection',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'party_id', '3': 2, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'provider', '3': 3, '4': 1, '5': 9, '10': 'provider'},
    {'1': 'psu_type', '3': 4, '4': 1, '5': 9, '10': 'psuType'},
    {'1': 'aspsp_name', '3': 5, '4': 1, '5': 9, '10': 'aspspName'},
    {'1': 'status', '3': 6, '4': 1, '5': 9, '10': 'status'},
    {'1': 'valid_until', '3': 7, '4': 1, '5': 9, '10': 'validUntil'},
    {'1': 'authorized_at', '3': 8, '4': 1, '5': 9, '10': 'authorizedAt'},
    {'1': 'accounts', '3': 9, '4': 1, '5': 13, '10': 'accounts'},
  ],
};

/// Descriptor for `Connection`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List connectionDescriptor = $convert.base64Decode(
    'CgpDb25uZWN0aW9uEg4KAmlkGAEgASgJUgJpZBIZCghwYXJ0eV9pZBgCIAEoCVIHcGFydHlJZB'
    'IaCghwcm92aWRlchgDIAEoCVIIcHJvdmlkZXISGQoIcHN1X3R5cGUYBCABKAlSB3BzdVR5cGUS'
    'HQoKYXNwc3BfbmFtZRgFIAEoCVIJYXNwc3BOYW1lEhYKBnN0YXR1cxgGIAEoCVIGc3RhdHVzEh'
    '8KC3ZhbGlkX3VudGlsGAcgASgJUgp2YWxpZFVudGlsEiMKDWF1dGhvcml6ZWRfYXQYCCABKAlS'
    'DGF1dGhvcml6ZWRBdBIaCghhY2NvdW50cxgJIAEoDVIIYWNjb3VudHM=');

@$core.Deprecated('Use pingRequestDescriptor instead')
const PingRequest$json = {
  '1': 'PingRequest',
  '2': [
    {'1': 'message', '3': 1, '4': 1, '5': 9, '10': 'message'},
  ],
};

/// Descriptor for `PingRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pingRequestDescriptor = $convert
    .base64Decode('CgtQaW5nUmVxdWVzdBIYCgdtZXNzYWdlGAEgASgJUgdtZXNzYWdl');

@$core.Deprecated('Use pingResponseDescriptor instead')
const PingResponse$json = {
  '1': 'PingResponse',
  '2': [
    {'1': 'message', '3': 1, '4': 1, '5': 9, '10': 'message'},
    {'1': 'version', '3': 2, '4': 1, '5': 9, '10': 'version'},
    {'1': 'stub', '3': 3, '4': 1, '5': 8, '10': 'stub'},
  ],
};

/// Descriptor for `PingResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pingResponseDescriptor = $convert.base64Decode(
    'CgxQaW5nUmVzcG9uc2USGAoHbWVzc2FnZRgBIAEoCVIHbWVzc2FnZRIYCgd2ZXJzaW9uGAIgAS'
    'gJUgd2ZXJzaW9uEhIKBHN0dWIYAyABKAhSBHN0dWI=');

@$core.Deprecated('Use listTransactionsRequestDescriptor instead')
const ListTransactionsRequest$json = {
  '1': 'ListTransactionsRequest',
  '2': [
    {'1': 'party_ids', '3': 1, '4': 3, '5': 9, '10': 'partyIds'},
    {'1': 'limit', '3': 2, '4': 1, '5': 13, '10': 'limit'},
    {'1': 'month', '3': 3, '4': 1, '5': 9, '10': 'month'},
    {'1': 'category_id', '3': 4, '4': 1, '5': 9, '10': 'categoryId'},
    {'1': 'account_id', '3': 5, '4': 1, '5': 9, '10': 'accountId'},
    {'1': 'search', '3': 6, '4': 1, '5': 9, '10': 'search'},
    {'1': 'offset', '3': 7, '4': 1, '5': 13, '10': 'offset'},
  ],
};

/// Descriptor for `ListTransactionsRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTransactionsRequestDescriptor = $convert.base64Decode(
    'ChdMaXN0VHJhbnNhY3Rpb25zUmVxdWVzdBIbCglwYXJ0eV9pZHMYASADKAlSCHBhcnR5SWRzEh'
    'QKBWxpbWl0GAIgASgNUgVsaW1pdBIUCgVtb250aBgDIAEoCVIFbW9udGgSHwoLY2F0ZWdvcnlf'
    'aWQYBCABKAlSCmNhdGVnb3J5SWQSHQoKYWNjb3VudF9pZBgFIAEoCVIJYWNjb3VudElkEhYKBn'
    'NlYXJjaBgGIAEoCVIGc2VhcmNoEhYKBm9mZnNldBgHIAEoDVIGb2Zmc2V0');

@$core.Deprecated('Use listTransactionsResponseDescriptor instead')
const ListTransactionsResponse$json = {
  '1': 'ListTransactionsResponse',
  '2': [
    {
      '1': 'transactions',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.finance.v1.Transaction',
      '10': 'transactions'
    },
    {'1': 'party_ids', '3': 2, '4': 3, '5': 9, '10': 'partyIds'},
  ],
};

/// Descriptor for `ListTransactionsResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List listTransactionsResponseDescriptor = $convert.base64Decode(
    'ChhMaXN0VHJhbnNhY3Rpb25zUmVzcG9uc2USPwoMdHJhbnNhY3Rpb25zGAEgAygLMhsudGJkLm'
    'ZpbmFuY2UudjEuVHJhbnNhY3Rpb25SDHRyYW5zYWN0aW9ucxIbCglwYXJ0eV9pZHMYAiADKAlS'
    'CHBhcnR5SWRz');

@$core.Deprecated('Use transactionDescriptor instead')
const Transaction$json = {
  '1': 'Transaction',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'account_id', '3': 2, '4': 1, '5': 9, '10': 'accountId'},
    {'1': 'party_id', '3': 3, '4': 1, '5': 9, '10': 'partyId'},
    {'1': 'status', '3': 4, '4': 1, '5': 9, '10': 'status'},
    {'1': 'amount_minor', '3': 5, '4': 1, '5': 3, '10': 'amountMinor'},
    {'1': 'currency', '3': 6, '4': 1, '5': 9, '10': 'currency'},
    {'1': 'scale', '3': 7, '4': 1, '5': 13, '10': 'scale'},
    {'1': 'booking_date', '3': 8, '4': 1, '5': 9, '10': 'bookingDate'},
    {
      '1': 'counterparty_name',
      '3': 9,
      '4': 1,
      '5': 9,
      '10': 'counterpartyName'
    },
    {'1': 'remittance', '3': 10, '4': 1, '5': 9, '10': 'remittance'},
    {'1': 'value_date', '3': 11, '4': 1, '5': 9, '10': 'valueDate'},
    {
      '1': 'counterparty_iban',
      '3': 12,
      '4': 1,
      '5': 9,
      '10': 'counterpartyIban'
    },
    {'1': 'category_id', '3': 13, '4': 1, '5': 9, '10': 'categoryId'},
    {'1': 'category', '3': 14, '4': 1, '5': 9, '10': 'category'},
    {'1': 'category_source', '3': 15, '4': 1, '5': 9, '10': 'categorySource'},
    {'1': 'internal', '3': 16, '4': 1, '5': 8, '10': 'internal'},
    {'1': 'reference_number', '3': 17, '4': 1, '5': 9, '10': 'referenceNumber'},
    {'1': 'entry_reference', '3': 18, '4': 1, '5': 9, '10': 'entryReference'},
    {'1': 'category_rule_id', '3': 19, '4': 1, '5': 9, '10': 'categoryRuleId'},
    {'1': 'categorised_at', '3': 20, '4': 1, '5': 9, '10': 'categorisedAt'},
    {'1': 'account_name', '3': 21, '4': 1, '5': 9, '10': 'accountName'},
    {'1': 'raw', '3': 22, '4': 1, '5': 9, '10': 'raw'},
  ],
};

/// Descriptor for `Transaction`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List transactionDescriptor = $convert.base64Decode(
    'CgtUcmFuc2FjdGlvbhIOCgJpZBgBIAEoCVICaWQSHQoKYWNjb3VudF9pZBgCIAEoCVIJYWNjb3'
    'VudElkEhkKCHBhcnR5X2lkGAMgASgJUgdwYXJ0eUlkEhYKBnN0YXR1cxgEIAEoCVIGc3RhdHVz'
    'EiEKDGFtb3VudF9taW5vchgFIAEoA1ILYW1vdW50TWlub3ISGgoIY3VycmVuY3kYBiABKAlSCG'
    'N1cnJlbmN5EhQKBXNjYWxlGAcgASgNUgVzY2FsZRIhCgxib29raW5nX2RhdGUYCCABKAlSC2Jv'
    'b2tpbmdEYXRlEisKEWNvdW50ZXJwYXJ0eV9uYW1lGAkgASgJUhBjb3VudGVycGFydHlOYW1lEh'
    '4KCnJlbWl0dGFuY2UYCiABKAlSCnJlbWl0dGFuY2USHQoKdmFsdWVfZGF0ZRgLIAEoCVIJdmFs'
    'dWVEYXRlEisKEWNvdW50ZXJwYXJ0eV9pYmFuGAwgASgJUhBjb3VudGVycGFydHlJYmFuEh8KC2'
    'NhdGVnb3J5X2lkGA0gASgJUgpjYXRlZ29yeUlkEhoKCGNhdGVnb3J5GA4gASgJUghjYXRlZ29y'
    'eRInCg9jYXRlZ29yeV9zb3VyY2UYDyABKAlSDmNhdGVnb3J5U291cmNlEhoKCGludGVybmFsGB'
    'AgASgIUghpbnRlcm5hbBIpChByZWZlcmVuY2VfbnVtYmVyGBEgASgJUg9yZWZlcmVuY2VOdW1i'
    'ZXISJwoPZW50cnlfcmVmZXJlbmNlGBIgASgJUg5lbnRyeVJlZmVyZW5jZRIoChBjYXRlZ29yeV'
    '9ydWxlX2lkGBMgASgJUg5jYXRlZ29yeVJ1bGVJZBIlCg5jYXRlZ29yaXNlZF9hdBgUIAEoCVIN'
    'Y2F0ZWdvcmlzZWRBdBIhCgxhY2NvdW50X25hbWUYFSABKAlSC2FjY291bnROYW1lEhAKA3Jhdx'
    'gWIAEoCVIDcmF3');

@$core.Deprecated('Use getTransactionRequestDescriptor instead')
const GetTransactionRequest$json = {
  '1': 'GetTransactionRequest',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
  ],
};

/// Descriptor for `GetTransactionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTransactionRequestDescriptor = $convert
    .base64Decode('ChVHZXRUcmFuc2FjdGlvblJlcXVlc3QSDgoCaWQYASABKAlSAmlk');

@$core.Deprecated('Use getTransactionResponseDescriptor instead')
const GetTransactionResponse$json = {
  '1': 'GetTransactionResponse',
  '2': [
    {
      '1': 'transaction',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.finance.v1.Transaction',
      '10': 'transaction'
    },
  ],
};

/// Descriptor for `GetTransactionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getTransactionResponseDescriptor =
    $convert.base64Decode(
        'ChZHZXRUcmFuc2FjdGlvblJlc3BvbnNlEj0KC3RyYW5zYWN0aW9uGAEgASgLMhsudGJkLmZpbm'
        'FuY2UudjEuVHJhbnNhY3Rpb25SC3RyYW5zYWN0aW9u');
