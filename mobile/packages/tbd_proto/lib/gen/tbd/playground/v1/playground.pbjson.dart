// This is a generated file - do not edit.
//
// Generated from tbd/playground/v1/playground.proto.

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

@$core.Deprecated('Use healthDescriptor instead')
const Health$json = {
  '1': 'Health',
  '2': [
    {'1': 'HEALTH_UNSPECIFIED', '2': 0},
    {'1': 'HEALTH_HEALED', '2': 1},
    {'1': 'HEALTH_DEGRADED', '2': 2},
    {'1': 'HEALTH_BREACHED', '2': 3},
  ],
};

/// Descriptor for `Health`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List healthDescriptor = $convert.base64Decode(
    'CgZIZWFsdGgSFgoSSEVBTFRIX1VOU1BFQ0lGSUVEEAASEQoNSEVBTFRIX0hFQUxFRBABEhMKD0'
    'hFQUxUSF9ERUdSQURFRBACEhMKD0hFQUxUSF9CUkVBQ0hFRBAD');

@$core.Deprecated('Use moveDescriptor instead')
const Move$json = {
  '1': 'Move',
  '2': [
    {'1': 'MOVE_UNSPECIFIED', '2': 0},
    {'1': 'MOVE_LATENCY', '2': 1},
    {'1': 'MOVE_ERRORS', '2': 2},
    {'1': 'MOVE_KILL', '2': 3},
    {'1': 'MOVE_STALL_STORE', '2': 4},
  ],
};

/// Descriptor for `Move`. Decode as a `google.protobuf.EnumDescriptorProto`.
final $typed_data.Uint8List moveDescriptor = $convert.base64Decode(
    'CgRNb3ZlEhQKEE1PVkVfVU5TUEVDSUZJRUQQABIQCgxNT1ZFX0xBVEVOQ1kQARIPCgtNT1ZFX0'
    'VSUk9SUxACEg0KCU1PVkVfS0lMTBADEhQKEE1PVkVfU1RBTExfU1RPUkUQBA==');

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

@$core.Deprecated('Use getWorldRequestDescriptor instead')
const GetWorldRequest$json = {
  '1': 'GetWorldRequest',
};

/// Descriptor for `GetWorldRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getWorldRequestDescriptor =
    $convert.base64Decode('Cg9HZXRXb3JsZFJlcXVlc3Q=');

@$core.Deprecated('Use getWorldResponseDescriptor instead')
const GetWorldResponse$json = {
  '1': 'GetWorldResponse',
  '2': [
    {
      '1': 'world',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.playground.v1.World',
      '10': 'world'
    },
  ],
};

/// Descriptor for `GetWorldResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List getWorldResponseDescriptor = $convert.base64Decode(
    'ChBHZXRXb3JsZFJlc3BvbnNlEi4KBXdvcmxkGAEgASgLMhgudGJkLnBsYXlncm91bmQudjEuV2'
    '9ybGRSBXdvcmxk');

@$core.Deprecated('Use watchRequestDescriptor instead')
const WatchRequest$json = {
  '1': 'WatchRequest',
};

/// Descriptor for `WatchRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchRequestDescriptor =
    $convert.base64Decode('CgxXYXRjaFJlcXVlc3Q=');

@$core.Deprecated('Use watchResponseDescriptor instead')
const WatchResponse$json = {
  '1': 'WatchResponse',
  '2': [
    {
      '1': 'world',
      '3': 1,
      '4': 1,
      '5': 11,
      '6': '.tbd.playground.v1.World',
      '10': 'world'
    },
  ],
};

/// Descriptor for `WatchResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List watchResponseDescriptor = $convert.base64Decode(
    'Cg1XYXRjaFJlc3BvbnNlEi4KBXdvcmxkGAEgASgLMhgudGJkLnBsYXlncm91bmQudjEuV29ybG'
    'RSBXdvcmxk');

@$core.Deprecated('Use worldDescriptor instead')
const World$json = {
  '1': 'World',
  '2': [
    {
      '1': 'health',
      '3': 1,
      '4': 1,
      '5': 14,
      '6': '.tbd.playground.v1.Health',
      '10': 'health'
    },
    {
      '1': 'objective',
      '3': 2,
      '4': 1,
      '5': 11,
      '6': '.tbd.playground.v1.Objective',
      '10': 'objective'
    },
    {
      '1': 'budget',
      '3': 3,
      '4': 1,
      '5': 11,
      '6': '.tbd.playground.v1.Budget',
      '10': 'budget'
    },
    {
      '1': 'traffic',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.tbd.playground.v1.Traffic',
      '10': 'traffic'
    },
    {
      '1': 'instances',
      '3': 5,
      '4': 3,
      '5': 11,
      '6': '.tbd.playground.v1.Instance',
      '10': 'instances'
    },
    {
      '1': 'faults',
      '3': 6,
      '4': 3,
      '5': 11,
      '6': '.tbd.playground.v1.ActiveFault',
      '10': 'faults'
    },
    {'1': 'held_for_seconds', '3': 7, '4': 1, '5': 13, '10': 'heldForSeconds'},
    {
      '1': 'now',
      '3': 8,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'now'
    },
    {'1': 'healing_seconds', '3': 9, '4': 1, '5': 13, '10': 'healingSeconds'},
  ],
};

/// Descriptor for `World`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List worldDescriptor = $convert.base64Decode(
    'CgVXb3JsZBIxCgZoZWFsdGgYASABKA4yGS50YmQucGxheWdyb3VuZC52MS5IZWFsdGhSBmhlYW'
    'x0aBI6CglvYmplY3RpdmUYAiABKAsyHC50YmQucGxheWdyb3VuZC52MS5PYmplY3RpdmVSCW9i'
    'amVjdGl2ZRIxCgZidWRnZXQYAyABKAsyGS50YmQucGxheWdyb3VuZC52MS5CdWRnZXRSBmJ1ZG'
    'dldBI0Cgd0cmFmZmljGAQgASgLMhoudGJkLnBsYXlncm91bmQudjEuVHJhZmZpY1IHdHJhZmZp'
    'YxI5CglpbnN0YW5jZXMYBSADKAsyGy50YmQucGxheWdyb3VuZC52MS5JbnN0YW5jZVIJaW5zdG'
    'FuY2VzEjYKBmZhdWx0cxgGIAMoCzIeLnRiZC5wbGF5Z3JvdW5kLnYxLkFjdGl2ZUZhdWx0UgZm'
    'YXVsdHMSKAoQaGVsZF9mb3Jfc2Vjb25kcxgHIAEoDVIOaGVsZEZvclNlY29uZHMSLAoDbm93GA'
    'ggASgLMhouZ29vZ2xlLnByb3RvYnVmLlRpbWVzdGFtcFIDbm93EicKD2hlYWxpbmdfc2Vjb25k'
    'cxgJIAEoDVIOaGVhbGluZ1NlY29uZHM=');

@$core.Deprecated('Use instanceDescriptor instead')
const Instance$json = {
  '1': 'Instance',
  '2': [
    {'1': 'name', '3': 1, '4': 1, '5': 9, '10': 'name'},
    {'1': 'kind', '3': 2, '4': 1, '5': 9, '10': 'kind'},
    {'1': 'running', '3': 3, '4': 1, '5': 8, '10': 'running'},
    {'1': 'ailment', '3': 4, '4': 1, '5': 9, '10': 'ailment'},
    {'1': 'requests_total', '3': 5, '4': 1, '5': 4, '10': 'requestsTotal'},
    {'1': 'requests_failed', '3': 6, '4': 1, '5': 4, '10': 'requestsFailed'},
    {'1': 'in_rotation', '3': 7, '4': 1, '5': 8, '10': 'inRotation'},
    {'1': 'ejected', '3': 8, '4': 1, '5': 8, '10': 'ejected'},
    {'1': 'balanced', '3': 9, '4': 1, '5': 8, '10': 'balanced'},
  ],
};

/// Descriptor for `Instance`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List instanceDescriptor = $convert.base64Decode(
    'CghJbnN0YW5jZRISCgRuYW1lGAEgASgJUgRuYW1lEhIKBGtpbmQYAiABKAlSBGtpbmQSGAoHcn'
    'VubmluZxgDIAEoCFIHcnVubmluZxIYCgdhaWxtZW50GAQgASgJUgdhaWxtZW50EiUKDnJlcXVl'
    'c3RzX3RvdGFsGAUgASgEUg1yZXF1ZXN0c1RvdGFsEicKD3JlcXVlc3RzX2ZhaWxlZBgGIAEoBF'
    'IOcmVxdWVzdHNGYWlsZWQSHwoLaW5fcm90YXRpb24YByABKAhSCmluUm90YXRpb24SGAoHZWpl'
    'Y3RlZBgIIAEoCFIHZWplY3RlZBIaCghiYWxhbmNlZBgJIAEoCFIIYmFsYW5jZWQ=');

@$core.Deprecated('Use objectiveDescriptor instead')
const Objective$json = {
  '1': 'Objective',
  '2': [
    {'1': 'target', '3': 1, '4': 1, '5': 1, '10': 'target'},
    {'1': 'current', '3': 2, '4': 1, '5': 1, '10': 'current'},
    {'1': 'window_seconds', '3': 3, '4': 1, '5': 13, '10': 'windowSeconds'},
    {'1': 'latency_target_ms', '3': 4, '4': 1, '5': 1, '10': 'latencyTargetMs'},
    {
      '1': 'latency_current_ms',
      '3': 5,
      '4': 1,
      '5': 1,
      '10': 'latencyCurrentMs'
    },
  ],
};

/// Descriptor for `Objective`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List objectiveDescriptor = $convert.base64Decode(
    'CglPYmplY3RpdmUSFgoGdGFyZ2V0GAEgASgBUgZ0YXJnZXQSGAoHY3VycmVudBgCIAEoAVIHY3'
    'VycmVudBIlCg53aW5kb3dfc2Vjb25kcxgDIAEoDVINd2luZG93U2Vjb25kcxIqChFsYXRlbmN5'
    'X3RhcmdldF9tcxgEIAEoAVIPbGF0ZW5jeVRhcmdldE1zEiwKEmxhdGVuY3lfY3VycmVudF9tcx'
    'gFIAEoAVIQbGF0ZW5jeUN1cnJlbnRNcw==');

@$core.Deprecated('Use budgetDescriptor instead')
const Budget$json = {
  '1': 'Budget',
  '2': [
    {'1': 'tokens', '3': 1, '4': 1, '5': 13, '10': 'tokens'},
    {'1': 'max_tokens', '3': 2, '4': 1, '5': 13, '10': 'maxTokens'},
    {
      '1': 'refill_in_seconds',
      '3': 3,
      '4': 1,
      '5': 13,
      '10': 'refillInSeconds'
    },
  ],
};

/// Descriptor for `Budget`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List budgetDescriptor = $convert.base64Decode(
    'CgZCdWRnZXQSFgoGdG9rZW5zGAEgASgNUgZ0b2tlbnMSHQoKbWF4X3Rva2VucxgCIAEoDVIJbW'
    'F4VG9rZW5zEioKEXJlZmlsbF9pbl9zZWNvbmRzGAMgASgNUg9yZWZpbGxJblNlY29uZHM=');

@$core.Deprecated('Use trafficDescriptor instead')
const Traffic$json = {
  '1': 'Traffic',
  '2': [
    {
      '1': 'requests_per_second',
      '3': 1,
      '4': 1,
      '5': 1,
      '10': 'requestsPerSecond'
    },
    {'1': 'error_rate', '3': 2, '4': 1, '5': 1, '10': 'errorRate'},
    {'1': 'p50_ms', '3': 3, '4': 1, '5': 1, '10': 'p50Ms'},
    {'1': 'p99_ms', '3': 4, '4': 1, '5': 1, '10': 'p99Ms'},
  ],
};

/// Descriptor for `Traffic`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List trafficDescriptor = $convert.base64Decode(
    'CgdUcmFmZmljEi4KE3JlcXVlc3RzX3Blcl9zZWNvbmQYASABKAFSEXJlcXVlc3RzUGVyU2Vjb2'
    '5kEh0KCmVycm9yX3JhdGUYAiABKAFSCWVycm9yUmF0ZRIVCgZwNTBfbXMYAyABKAFSBXA1ME1z'
    'EhUKBnA5OV9tcxgEIAEoAVIFcDk5TXM=');

@$core.Deprecated('Use activeFaultDescriptor instead')
const ActiveFault$json = {
  '1': 'ActiveFault',
  '2': [
    {'1': 'id', '3': 1, '4': 1, '5': 9, '10': 'id'},
    {
      '1': 'move',
      '3': 2,
      '4': 1,
      '5': 14,
      '6': '.tbd.playground.v1.Move',
      '10': 'move'
    },
    {'1': 'instance', '3': 3, '4': 1, '5': 9, '10': 'instance'},
    {'1': 'description', '3': 4, '4': 1, '5': 9, '10': 'description'},
    {
      '1': 'expires_in_seconds',
      '3': 5,
      '4': 1,
      '5': 13,
      '10': 'expiresInSeconds'
    },
    {'1': 'actor', '3': 6, '4': 1, '5': 9, '10': 'actor'},
  ],
};

/// Descriptor for `ActiveFault`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List activeFaultDescriptor = $convert.base64Decode(
    'CgtBY3RpdmVGYXVsdBIOCgJpZBgBIAEoCVICaWQSKwoEbW92ZRgCIAEoDjIXLnRiZC5wbGF5Z3'
    'JvdW5kLnYxLk1vdmVSBG1vdmUSGgoIaW5zdGFuY2UYAyABKAlSCGluc3RhbmNlEiAKC2Rlc2Ny'
    'aXB0aW9uGAQgASgJUgtkZXNjcmlwdGlvbhIsChJleHBpcmVzX2luX3NlY29uZHMYBSABKA1SEG'
    'V4cGlyZXNJblNlY29uZHMSFAoFYWN0b3IYBiABKAlSBWFjdG9y');

@$core.Deprecated('Use injectFaultRequestDescriptor instead')
const InjectFaultRequest$json = {
  '1': 'InjectFaultRequest',
  '2': [
    {
      '1': 'move',
      '3': 1,
      '4': 1,
      '5': 14,
      '6': '.tbd.playground.v1.Move',
      '10': 'move'
    },
    {'1': 'instance', '3': 2, '4': 1, '5': 9, '10': 'instance'},
    {'1': 'actor', '3': 3, '4': 1, '5': 9, '10': 'actor'},
  ],
};

/// Descriptor for `InjectFaultRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List injectFaultRequestDescriptor = $convert.base64Decode(
    'ChJJbmplY3RGYXVsdFJlcXVlc3QSKwoEbW92ZRgBIAEoDjIXLnRiZC5wbGF5Z3JvdW5kLnYxLk'
    '1vdmVSBG1vdmUSGgoIaW5zdGFuY2UYAiABKAlSCGluc3RhbmNlEhQKBWFjdG9yGAMgASgJUgVh'
    'Y3Rvcg==');

@$core.Deprecated('Use injectFaultResponseDescriptor instead')
const InjectFaultResponse$json = {
  '1': 'InjectFaultResponse',
  '2': [
    {'1': 'accepted', '3': 1, '4': 1, '5': 8, '10': 'accepted'},
    {'1': 'reason', '3': 2, '4': 1, '5': 9, '10': 'reason'},
    {'1': 'cost', '3': 3, '4': 1, '5': 13, '10': 'cost'},
    {
      '1': 'world',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.tbd.playground.v1.World',
      '10': 'world'
    },
  ],
};

/// Descriptor for `InjectFaultResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List injectFaultResponseDescriptor = $convert.base64Decode(
    'ChNJbmplY3RGYXVsdFJlc3BvbnNlEhoKCGFjY2VwdGVkGAEgASgIUghhY2NlcHRlZBIWCgZyZW'
    'Fzb24YAiABKAlSBnJlYXNvbhISCgRjb3N0GAMgASgNUgRjb3N0Ei4KBXdvcmxkGAQgASgLMhgu'
    'dGJkLnBsYXlncm91bmQudjEuV29ybGRSBXdvcmxk');

@$core.Deprecated('Use scoresRequestDescriptor instead')
const ScoresRequest$json = {
  '1': 'ScoresRequest',
};

/// Descriptor for `ScoresRequest`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List scoresRequestDescriptor =
    $convert.base64Decode('Cg1TY29yZXNSZXF1ZXN0');

@$core.Deprecated('Use scoresResponseDescriptor instead')
const ScoresResponse$json = {
  '1': 'ScoresResponse',
  '2': [
    {
      '1': 'scores',
      '3': 1,
      '4': 3,
      '5': 11,
      '6': '.tbd.playground.v1.Score',
      '10': 'scores'
    },
  ],
};

/// Descriptor for `ScoresResponse`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List scoresResponseDescriptor = $convert.base64Decode(
    'Cg5TY29yZXNSZXNwb25zZRIwCgZzY29yZXMYASADKAsyGC50YmQucGxheWdyb3VuZC52MS5TY2'
    '9yZVIGc2NvcmVz');

@$core.Deprecated('Use scoreDescriptor instead')
const Score$json = {
  '1': 'Score',
  '2': [
    {'1': 'actor', '3': 1, '4': 1, '5': 9, '10': 'actor'},
    {'1': 'seconds_to_breach', '3': 2, '4': 1, '5': 1, '10': 'secondsToBreach'},
    {'1': 'faults_used', '3': 3, '4': 1, '5': 13, '10': 'faultsUsed'},
    {
      '1': 'at',
      '3': 4,
      '4': 1,
      '5': 11,
      '6': '.google.protobuf.Timestamp',
      '10': 'at'
    },
  ],
};

/// Descriptor for `Score`. Decode as a `google.protobuf.DescriptorProto`.
final $typed_data.Uint8List scoreDescriptor = $convert.base64Decode(
    'CgVTY29yZRIUCgVhY3RvchgBIAEoCVIFYWN0b3ISKgoRc2Vjb25kc190b19icmVhY2gYAiABKA'
    'FSD3NlY29uZHNUb0JyZWFjaBIfCgtmYXVsdHNfdXNlZBgDIAEoDVIKZmF1bHRzVXNlZBIqCgJh'
    'dBgEIAEoCzIaLmdvb2dsZS5wcm90b2J1Zi5UaW1lc3RhbXBSAmF0');
