// This is a generated file - do not edit.
//
// Generated from tbd/ledger/v1/ledger.proto.

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

@$core.Deprecated('Use sourceDescriptor instead')
const Source$json = {
  '1': 'Source',
  '2': [
    {'1': 'SOURCE_UNSPECIFIED', '2': 0},
    {'1': 'SOURCE_VERIFIED', '2': 1},
    {'1': 'SOURCE_DECLARED', '2': 2},
    {'1': 'SOURCE_INFERRED', '2': 3},
    {'1': 'SOURCE_SYMBOLIC', '2': 4},
    {'1': 'SOURCE_OBSERVED', '2': 5},
  ],
};

/// Descriptor for `Source`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List sourceDescriptor = $convert.base64Decode(
    'CgZTb3VyY2USFgoSU09VUkNFX1VOU1BFQ0lGSUVEEAASEwoPU09VUkNFX1ZFUklGSUVEEAESEw'
    'oPU09VUkNFX0RFQ0xBUkVEEAISEwoPU09VUkNFX0lORkVSUkVEEAMSEwoPU09VUkNFX1NZTUJP'
    'TElDEAQSEwoPU09VUkNFX09CU0VSVkVEEAU=');

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
    {'1': 'store', '3': 4, '4': 1, '5': 9, '10': 'store'},
  ],
};

/// Descriptor for `PingResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List pingResponseDescriptor = $convert.base64Decode(
    'CgxQaW5nUmVzcG9uc2USGAoHbWVzc2FnZRgBIAEoCVIHbWVzc2FnZRIYCgd2ZXJzaW9uGAIgAS'
    'gJUgd2ZXJzaW9uEhIKBHN0dWIYAyABKAhSBHN0dWISFAoFc3RvcmUYBCABKAlSBXN0b3Jl');

@$core.Deprecated('Use envelopeDescriptor instead')
const Envelope$json = {
  '1': 'Envelope',
  '2': [
    {'1': 'version', '3': 1, '4': 1, '5': 13, '10': 'version'},
    {'1': 'bytes', '3': 2, '4': 1, '5': 12, '10': 'bytes'},
  ],
};

/// Descriptor for `Envelope`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List envelopeDescriptor = $convert.base64Decode(
    'CghFbnZlbG9wZRIYCgd2ZXJzaW9uGAEgASgNUgd2ZXJzaW9uEhQKBWJ5dGVzGAIgASgMUgVieX'
    'Rlcw==');

@$core.Deprecated('Use factDescriptor instead')
const Fact$json = {
  '1': 'Fact',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'id', '3': 2, '4': 1, '5': 3, '10': 'id'},
    {'1': 'path', '3': 3, '4': 1, '5': 9, '10': 'path'},
    {
      '1': 'source',
      '3': 4,
      '4': 1,
      '5': 14,
      '6': '.tbd.ledger.v1.Source',
      '10': 'source'
    },
    {
      '1': 'value',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.tbd.ledger.v1.Envelope',
      '10': 'value'
    },
    {
      '1': 'origin',
      '3': 6,
      '4': 1,
      '5': 11,
      '6': '.tbd.ledger.v1.Envelope',
      '10': 'origin'
    },
    {
      '1': 'confidence',
      '3': 7,
      '4': 1,
      '5': 2,
      '9': 0,
      '10': 'confidence',
      '17': true
    },
    {
      '1': 'counterparty_id',
      '3': 8,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'counterpartyId',
      '17': true
    },
    {
      '1': 'observed_at',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'observedAt'
    },
    {
      '1': 'recorded_at',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'recordedAt'
    },
    {
      '1': 'expires_at',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'expiresAt'
    },
    {'1': 'consent', '3': 12, '4': 3, '5': 9, '10': 'consent'},
    {'1': 'stub', '3': 13, '4': 1, '5': 8, '10': 'stub'},
  ],
  '8': [
    {'1': '_confidence'},
    {'1': '_counterparty_id'},
  ],
};

/// Descriptor for `Fact`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List factDescriptor = $convert.base64Decode(
    'CgRGYWN0Eh0KCnN1YmplY3RfaWQYASABKAlSCXN1YmplY3RJZBIOCgJpZBgCIAEoA1ICaWQSEg'
    'oEcGF0aBgDIAEoCVIEcGF0aBItCgZzb3VyY2UYBCABKA4yFS50YmQubGVkZ2VyLnYxLlNvdXJj'
    'ZVIGc291cmNlEi0KBXZhbHVlGAUgASgLMhcudGJkLmxlZGdlci52MS5FbnZlbG9wZVIFdmFsdW'
    'USLwoGb3JpZ2luGAYgASgLMhcudGJkLmxlZGdlci52MS5FbnZlbG9wZVIGb3JpZ2luEiMKCmNv'
    'bmZpZGVuY2UYByABKAJIAFIKY29uZmlkZW5jZYgBARIsCg9jb3VudGVycGFydHlfaWQYCCABKA'
    'lIAVIOY291bnRlcnBhcnR5SWSIAQESOwoLb2JzZXJ2ZWRfYXQYCSABKAsyGi5nb29nbGUucHJv'
    'dG9idWYuVGltZXN0YW1wUgpvYnNlcnZlZEF0EjsKC3JlY29yZGVkX2F0GAogASgLMhouZ29vZ2'
    'xlLnByb3RvYnVmLlRpbWVzdGFtcFIKcmVjb3JkZWRBdBI5CgpleHBpcmVzX2F0GAsgASgLMhou'
    'Z29vZ2xlLnByb3RvYnVmLlRpbWVzdGFtcFIJZXhwaXJlc0F0EhgKB2NvbnNlbnQYDCADKAlSB2'
    'NvbnNlbnQSEgoEc3R1YhgNIAEoCFIEc3R1YkINCgtfY29uZmlkZW5jZUISChBfY291bnRlcnBh'
    'cnR5X2lk');

@$core.Deprecated('Use appendRequestDescriptor instead')
const AppendRequest$json = {
  '1': 'AppendRequest',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'path', '3': 2, '4': 1, '5': 9, '10': 'path'},
    {
      '1': 'source',
      '3': 3,
      '4': 1,
      '5': 14,
      '6': '.tbd.ledger.v1.Source',
      '10': 'source'
    },
    {
      '1': 'value',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.tbd.ledger.v1.Envelope',
      '10': 'value'
    },
    {
      '1': 'origin',
      '3': 5,
      '4': 1,
      '5': 11,
      '6': '.tbd.ledger.v1.Envelope',
      '10': 'origin'
    },
    {
      '1': 'confidence',
      '3': 6,
      '4': 1,
      '5': 2,
      '9': 0,
      '10': 'confidence',
      '17': true
    },
    {
      '1': 'counterparty_id',
      '3': 7,
      '4': 1,
      '5': 9,
      '9': 1,
      '10': 'counterpartyId',
      '17': true
    },
    {
      '1': 'observed_at',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'observedAt'
    },
    {
      '1': 'expires_at',
      '3': 9,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'expiresAt'
    },
    {'1': 'consent', '3': 10, '4': 3, '5': 9, '10': 'consent'},
    {'1': 'stub', '3': 11, '4': 1, '5': 8, '10': 'stub'},
    {'1': 'idempotency_key', '3': 12, '4': 1, '5': 9, '10': 'idempotencyKey'},
  ],
  '8': [
    {'1': '_confidence'},
    {'1': '_counterparty_id'},
  ],
};

/// Descriptor for `AppendRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List appendRequestDescriptor = $convert.base64Decode(
    'Cg1BcHBlbmRSZXF1ZXN0Eh0KCnN1YmplY3RfaWQYASABKAlSCXN1YmplY3RJZBISCgRwYXRoGA'
    'IgASgJUgRwYXRoEi0KBnNvdXJjZRgDIAEoDjIVLnRiZC5sZWRnZXIudjEuU291cmNlUgZzb3Vy'
    'Y2USLQoFdmFsdWUYBCABKAsyFy50YmQubGVkZ2VyLnYxLkVudmVsb3BlUgV2YWx1ZRIvCgZvcm'
    'lnaW4YBSABKAsyFy50YmQubGVkZ2VyLnYxLkVudmVsb3BlUgZvcmlnaW4SIwoKY29uZmlkZW5j'
    'ZRgGIAEoAkgAUgpjb25maWRlbmNliAEBEiwKD2NvdW50ZXJwYXJ0eV9pZBgHIAEoCUgBUg5jb3'
    'VudGVycGFydHlJZIgBARI7CgtvYnNlcnZlZF9hdBgIIAEoCzIaLmdvb2dsZS5wcm90b2J1Zi5U'
    'aW1lc3RhbXBSCm9ic2VydmVkQXQSOQoKZXhwaXJlc19hdBgJIAEoCzIaLmdvb2dsZS5wcm90b2'
    'J1Zi5UaW1lc3RhbXBSCWV4cGlyZXNBdBIYCgdjb25zZW50GAogAygJUgdjb25zZW50EhIKBHN0'
    'dWIYCyABKAhSBHN0dWISJwoPaWRlbXBvdGVuY3lfa2V5GAwgASgJUg5pZGVtcG90ZW5jeUtleU'
    'INCgtfY29uZmlkZW5jZUISChBfY291bnRlcnBhcnR5X2lk');

@$core.Deprecated('Use appendResponseDescriptor instead')
const AppendResponse$json = {
  '1': 'AppendResponse',
  '2': [
    {
      '1': 'fact',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.ledger.v1.Fact',
      '10': 'fact'
    },
    {'1': 'replayed', '3': 2, '4': 1, '5': 8, '10': 'replayed'},
  ],
};

/// Descriptor for `AppendResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List appendResponseDescriptor = $convert.base64Decode(
    'Cg5BcHBlbmRSZXNwb25zZRInCgRmYWN0GAEgASgLMhMudGJkLmxlZGdlci52MS5GYWN0UgRmYW'
    'N0EhoKCHJlcGxheWVkGAIgASgIUghyZXBsYXllZA==');

@$core.Deprecated('Use currentRequestDescriptor instead')
const CurrentRequest$json = {
  '1': 'CurrentRequest',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'paths', '3': 2, '4': 3, '5': 9, '10': 'paths'},
    {
      '1': 'sources',
      '3': 3,
      '4': 3,
      '5': 14,
      '6': '.tbd.ledger.v1.Source',
      '10': 'sources'
    },
    {'1': 'scopes', '3': 4, '4': 3, '5': 9, '10': 'scopes'},
    {'1': 'cursor', '3': 5, '4': 1, '5': 9, '10': 'cursor'},
    {'1': 'limit', '3': 6, '4': 1, '5': 13, '10': 'limit'},
  ],
};

/// Descriptor for `CurrentRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List currentRequestDescriptor = $convert.base64Decode(
    'Cg5DdXJyZW50UmVxdWVzdBIdCgpzdWJqZWN0X2lkGAEgASgJUglzdWJqZWN0SWQSFAoFcGF0aH'
    'MYAiADKAlSBXBhdGhzEi8KB3NvdXJjZXMYAyADKA4yFS50YmQubGVkZ2VyLnYxLlNvdXJjZVIH'
    'c291cmNlcxIWCgZzY29wZXMYBCADKAlSBnNjb3BlcxIWCgZjdXJzb3IYBSABKAlSBmN1cnNvch'
    'IUCgVsaW1pdBgGIAEoDVIFbGltaXQ=');

@$core.Deprecated('Use currentResponseDescriptor instead')
const CurrentResponse$json = {
  '1': 'CurrentResponse',
  '2': [
    {
      '1': 'facts',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.ledger.v1.Fact',
      '10': 'facts'
    },
    {'1': 'next', '3': 2, '4': 1, '5': 9, '10': 'next'},
  ],
};

/// Descriptor for `CurrentResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List currentResponseDescriptor = $convert.base64Decode(
    'Cg9DdXJyZW50UmVzcG9uc2USKQoFZmFjdHMYASADKAsyEy50YmQubGVkZ2VyLnYxLkZhY3RSBW'
    'ZhY3RzEhIKBG5leHQYAiABKAlSBG5leHQ=');

@$core.Deprecated('Use historyRequestDescriptor instead')
const HistoryRequest$json = {
  '1': 'HistoryRequest',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'paths', '3': 2, '4': 3, '5': 9, '10': 'paths'},
    {
      '1': 'sources',
      '3': 3,
      '4': 3,
      '5': 14,
      '6': '.tbd.ledger.v1.Source',
      '10': 'sources'
    },
    {'1': 'scopes', '3': 4, '4': 3, '5': 9, '10': 'scopes'},
    {'1': 'cursor', '3': 5, '4': 1, '5': 9, '10': 'cursor'},
    {'1': 'limit', '3': 6, '4': 1, '5': 13, '10': 'limit'},
    {
      '1': 'at',
      '3': 7,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'at'
    },
  ],
};

/// Descriptor for `HistoryRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List historyRequestDescriptor = $convert.base64Decode(
    'Cg5IaXN0b3J5UmVxdWVzdBIdCgpzdWJqZWN0X2lkGAEgASgJUglzdWJqZWN0SWQSFAoFcGF0aH'
    'MYAiADKAlSBXBhdGhzEi8KB3NvdXJjZXMYAyADKA4yFS50YmQubGVkZ2VyLnYxLlNvdXJjZVIH'
    'c291cmNlcxIWCgZzY29wZXMYBCADKAlSBnNjb3BlcxIWCgZjdXJzb3IYBSABKAlSBmN1cnNvch'
    'IUCgVsaW1pdBgGIAEoDVIFbGltaXQSKgoCYXQYByABKAsyGi5nb29nbGUucHJvdG9idWYuVGlt'
    'ZXN0YW1wUgJhdA==');

@$core.Deprecated('Use historyResponseDescriptor instead')
const HistoryResponse$json = {
  '1': 'HistoryResponse',
  '2': [
    {
      '1': 'facts',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.ledger.v1.Fact',
      '10': 'facts'
    },
    {'1': 'next', '3': 2, '4': 1, '5': 9, '10': 'next'},
  ],
};

/// Descriptor for `HistoryResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List historyResponseDescriptor = $convert.base64Decode(
    'Cg9IaXN0b3J5UmVzcG9uc2USKQoFZmFjdHMYASADKAsyEy50YmQubGVkZ2VyLnYxLkZhY3RSBW'
    'ZhY3RzEhIKBG5leHQYAiABKAlSBG5leHQ=');

@$core.Deprecated('Use retractRequestDescriptor instead')
const RetractRequest$json = {
  '1': 'RetractRequest',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'path', '3': 2, '4': 1, '5': 9, '10': 'path'},
    {
      '1': 'source',
      '3': 3,
      '4': 1,
      '5': 14,
      '6': '.tbd.ledger.v1.Source',
      '10': 'source'
    },
    {
      '1': 'origin',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.tbd.ledger.v1.Envelope',
      '10': 'origin'
    },
  ],
};

/// Descriptor for `RetractRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retractRequestDescriptor = $convert.base64Decode(
    'Cg5SZXRyYWN0UmVxdWVzdBIdCgpzdWJqZWN0X2lkGAEgASgJUglzdWJqZWN0SWQSEgoEcGF0aB'
    'gCIAEoCVIEcGF0aBItCgZzb3VyY2UYAyABKA4yFS50YmQubGVkZ2VyLnYxLlNvdXJjZVIGc291'
    'cmNlEi8KBm9yaWdpbhgEIAEoCzIXLnRiZC5sZWRnZXIudjEuRW52ZWxvcGVSBm9yaWdpbg==');

@$core.Deprecated('Use retractResponseDescriptor instead')
const RetractResponse$json = {
  '1': 'RetractResponse',
  '2': [
    {
      '1': 'tombstone',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.ledger.v1.Fact',
      '10': 'tombstone'
    },
  ],
};

/// Descriptor for `RetractResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List retractResponseDescriptor = $convert.base64Decode(
    'Cg9SZXRyYWN0UmVzcG9uc2USMQoJdG9tYnN0b25lGAEgASgLMhMudGJkLmxlZGdlci52MS5GYW'
    'N0Ugl0b21ic3RvbmU=');

@$core.Deprecated('Use eraseRequestDescriptor instead')
const EraseRequest$json = {
  '1': 'EraseRequest',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
  ],
};

/// Descriptor for `EraseRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eraseRequestDescriptor = $convert.base64Decode(
    'CgxFcmFzZVJlcXVlc3QSHQoKc3ViamVjdF9pZBgBIAEoCVIJc3ViamVjdElk');

@$core.Deprecated('Use eraseResponseDescriptor instead')
const EraseResponse$json = {
  '1': 'EraseResponse',
  '2': [
    {
      '1': 'requested_at',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'requestedAt'
    },
    {
      '1': 'executes_after',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'executesAfter'
    },
  ],
};

/// Descriptor for `EraseResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List eraseResponseDescriptor = $convert.base64Decode(
    'Cg1FcmFzZVJlc3BvbnNlEj0KDHJlcXVlc3RlZF9hdBgBIAEoCzIaLmdvb2dsZS5wcm90b2J1Zi'
    '5UaW1lc3RhbXBSC3JlcXVlc3RlZEF0EkEKDmV4ZWN1dGVzX2FmdGVyGAIgASgLMhouZ29vZ2xl'
    'LnByb3RvYnVmLlRpbWVzdGFtcFINZXhlY3V0ZXNBZnRlcg==');

@$core.Deprecated('Use restoreRequestDescriptor instead')
const RestoreRequest$json = {
  '1': 'RestoreRequest',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
  ],
};

/// Descriptor for `RestoreRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List restoreRequestDescriptor = $convert.base64Decode(
    'Cg5SZXN0b3JlUmVxdWVzdBIdCgpzdWJqZWN0X2lkGAEgASgJUglzdWJqZWN0SWQ=');

@$core.Deprecated('Use restoreResponseDescriptor instead')
const RestoreResponse$json = {
  '1': 'RestoreResponse',
};

/// Descriptor for `RestoreResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List restoreResponseDescriptor =
    $convert.base64Decode('Cg9SZXN0b3JlUmVzcG9uc2U=');
