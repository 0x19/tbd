// This is a generated file - do not edit.
//
// Generated from tbd/engine/v1/engine.proto.

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

@$core.Deprecated('Use evaluateRequestDescriptor instead')
const EvaluateRequest$json = {
  '1': 'EvaluateRequest',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'payload', '3': 2, '4': 1, '5': 12, '10': 'payload'},
  ],
};

/// Descriptor for `EvaluateRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluateRequestDescriptor = $convert.base64Decode(
    'Cg9FdmFsdWF0ZVJlcXVlc3QSHQoKc3ViamVjdF9pZBgBIAEoCVIJc3ViamVjdElkEhgKB3BheW'
    'xvYWQYAiABKAxSB3BheWxvYWQ=');

@$core.Deprecated('Use evaluateResponseDescriptor instead')
const EvaluateResponse$json = {
  '1': 'EvaluateResponse',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
    {'1': 'score', '3': 2, '4': 1, '5': 1, '10': 'score'},
    {'1': 'stub', '3': 3, '4': 1, '5': 8, '10': 'stub'},
    {'1': 'model_version', '3': 4, '4': 1, '5': 9, '10': 'modelVersion'},
  ],
};

/// Descriptor for `EvaluateResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List evaluateResponseDescriptor = $convert.base64Decode(
    'ChBFdmFsdWF0ZVJlc3BvbnNlEh0KCnN1YmplY3RfaWQYASABKAlSCXN1YmplY3RJZBIUCgVzY2'
    '9yZRgCIAEoAVIFc2NvcmUSEgoEc3R1YhgDIAEoCFIEc3R1YhIjCg1tb2RlbF92ZXJzaW9uGAQg'
    'ASgJUgxtb2RlbFZlcnNpb24=');

@$core.Deprecated('Use subscribeRequestDescriptor instead')
const SubscribeRequest$json = {
  '1': 'SubscribeRequest',
  '2': [
    {'1': 'subject_id', '3': 1, '4': 1, '5': 9, '10': 'subjectId'},
  ],
};

/// Descriptor for `SubscribeRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List subscribeRequestDescriptor = $convert.base64Decode(
    'ChBTdWJzY3JpYmVSZXF1ZXN0Eh0KCnN1YmplY3RfaWQYASABKAlSCXN1YmplY3RJZA==');

@$core.Deprecated('Use subscribeResponseDescriptor instead')
const SubscribeResponse$json = {
  '1': 'SubscribeResponse',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {'1': 'subject_id', '3': 2, '4': 1, '5': 9, '10': 'subjectId'},
    {
      '1': 'at',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'at'
    },
    {
      '1': 'heartbeat',
      '3': 10,
      '4': 1,
      '5': 11,
      '6': '.tbd.engine.v1.Heartbeat',
      '9': 0,
      '10': 'heartbeat'
    },
    {
      '1': 'score_updated',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.tbd.engine.v1.ScoreUpdated',
      '9': 0,
      '10': 'scoreUpdated'
    },
  ],
  '8': [
    {'1': 'kind'},
  ],
};

/// Descriptor for `SubscribeResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List subscribeResponseDescriptor = $convert.base64Decode(
    'ChFTdWJzY3JpYmVSZXNwb25zZRIOCgJpZBgBIAEoCVICaWQSHQoKc3ViamVjdF9pZBgCIAEoCV'
    'IJc3ViamVjdElkEioKAmF0GAMgASgLMhouZ29vZ2xlLnByb3RvYnVmLlRpbWVzdGFtcFICYXQS'
    'OAoJaGVhcnRiZWF0GAogASgLMhgudGJkLmVuZ2luZS52MS5IZWFydGJlYXRIAFIJaGVhcnRiZW'
    'F0EkIKDXNjb3JlX3VwZGF0ZWQYCyABKAsyGy50YmQuZW5naW5lLnYxLlNjb3JlVXBkYXRlZEgA'
    'UgxzY29yZVVwZGF0ZWRCBgoEa2luZA==');

@$core.Deprecated('Use heartbeatDescriptor instead')
const Heartbeat$json = {
  '1': 'Heartbeat',
  '2': [
    {'1': 'seq', '3': 1, '4': 1, '5': 4, '10': 'seq'},
  ],
};

/// Descriptor for `Heartbeat`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List heartbeatDescriptor =
    $convert.base64Decode('CglIZWFydGJlYXQSEAoDc2VxGAEgASgEUgNzZXE=');

@$core.Deprecated('Use scoreUpdatedDescriptor instead')
const ScoreUpdated$json = {
  '1': 'ScoreUpdated',
  '2': [
    {'1': 'score', '3': 1, '4': 1, '5': 1, '10': 'score'},
    {'1': 'stub', '3': 2, '4': 1, '5': 8, '10': 'stub'},
  ],
};

/// Descriptor for `ScoreUpdated`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List scoreUpdatedDescriptor = $convert.base64Decode(
    'CgxTY29yZVVwZGF0ZWQSFAoFc2NvcmUYASABKAFSBXNjb3JlEhIKBHN0dWIYAiABKAhSBHN0dW'
    'I=');

@$core.Deprecated('Use sessionRequestDescriptor instead')
const SessionRequest$json = {
  '1': 'SessionRequest',
  '2': [
    {'1': 'session_id', '3': 1, '4': 1, '5': 9, '10': 'sessionId'},
    {'1': 'seq', '3': 2, '4': 1, '5': 4, '10': 'seq'},
    {'1': 'data', '3': 10, '4': 1, '5': 12, '9': 0, '10': 'data'},
    {
      '1': 'heartbeat',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.tbd.engine.v1.Heartbeat',
      '9': 0,
      '10': 'heartbeat'
    },
    {
      '1': 'close',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.tbd.engine.v1.Close',
      '9': 0,
      '10': 'close'
    },
  ],
  '8': [
    {'1': 'body'},
  ],
};

/// Descriptor for `SessionRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sessionRequestDescriptor = $convert.base64Decode(
    'Cg5TZXNzaW9uUmVxdWVzdBIdCgpzZXNzaW9uX2lkGAEgASgJUglzZXNzaW9uSWQSEAoDc2VxGA'
    'IgASgEUgNzZXESFAoEZGF0YRgKIAEoDEgAUgRkYXRhEjgKCWhlYXJ0YmVhdBgLIAEoCzIYLnRi'
    'ZC5lbmdpbmUudjEuSGVhcnRiZWF0SABSCWhlYXJ0YmVhdBIsCgVjbG9zZRgMIAEoCzIULnRiZC'
    '5lbmdpbmUudjEuQ2xvc2VIAFIFY2xvc2VCBgoEYm9keQ==');

@$core.Deprecated('Use sessionResponseDescriptor instead')
const SessionResponse$json = {
  '1': 'SessionResponse',
  '2': [
    {'1': 'session_id', '3': 1, '4': 1, '5': 9, '10': 'sessionId'},
    {'1': 'seq', '3': 2, '4': 1, '5': 4, '10': 'seq'},
    {'1': 'data', '3': 10, '4': 1, '5': 12, '9': 0, '10': 'data'},
    {
      '1': 'heartbeat',
      '3': 11,
      '4': 1,
      '5': 11,
      '6': '.tbd.engine.v1.Heartbeat',
      '9': 0,
      '10': 'heartbeat'
    },
    {
      '1': 'close',
      '3': 12,
      '4': 1,
      '5': 11,
      '6': '.tbd.engine.v1.Close',
      '9': 0,
      '10': 'close'
    },
  ],
  '8': [
    {'1': 'body'},
  ],
};

/// Descriptor for `SessionResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List sessionResponseDescriptor = $convert.base64Decode(
    'Cg9TZXNzaW9uUmVzcG9uc2USHQoKc2Vzc2lvbl9pZBgBIAEoCVIJc2Vzc2lvbklkEhAKA3NlcR'
    'gCIAEoBFIDc2VxEhQKBGRhdGEYCiABKAxIAFIEZGF0YRI4CgloZWFydGJlYXQYCyABKAsyGC50'
    'YmQuZW5naW5lLnYxLkhlYXJ0YmVhdEgAUgloZWFydGJlYXQSLAoFY2xvc2UYDCABKAsyFC50Ym'
    'QuZW5naW5lLnYxLkNsb3NlSABSBWNsb3NlQgYKBGJvZHk=');

@$core.Deprecated('Use closeDescriptor instead')
const Close$json = {
  '1': 'Close',
  '2': [
    {'1': 'reason', '3': 1, '4': 1, '5': 9, '10': 'reason'},
  ],
};

/// Descriptor for `Close`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List closeDescriptor =
    $convert.base64Decode('CgVDbG9zZRIWCgZyZWFzb24YASABKAlSBnJlYXNvbg==');
