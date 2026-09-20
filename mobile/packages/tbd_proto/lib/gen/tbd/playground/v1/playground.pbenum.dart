// This is a generated file - do not edit.
//
// Generated from tbd/playground/v1/playground.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// How the sandbox is doing overall.
class Health extends $pb.ProtobufEnum {
  static const Health HEALTH_UNSPECIFIED =
      Health._(0, _omitEnumNames ? '' : 'HEALTH_UNSPECIFIED');

  /// Nothing is wrong and nothing is injected.
  static const Health HEALTH_HEALED =
      Health._(1, _omitEnumNames ? '' : 'HEALTH_HEALED');

  /// Something is injected but the objective still holds.
  static const Health HEALTH_DEGRADED =
      Health._(2, _omitEnumNames ? '' : 'HEALTH_DEGRADED');

  /// The objective is broken. Somebody won.
  static const Health HEALTH_BREACHED =
      Health._(3, _omitEnumNames ? '' : 'HEALTH_BREACHED');

  static const $core.List<Health> values = <Health>[
    HEALTH_UNSPECIFIED,
    HEALTH_HEALED,
    HEALTH_DEGRADED,
    HEALTH_BREACHED,
  ];

  static final $core.List<Health?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 3);
  static Health? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const Health._(super.value, super.name);
}

/// One move a caller can make. Anything not listed here cannot be injected at
/// all: `hang` never returns and `delayed_failure` nests without bound, so
/// neither is reachable from this surface.
class Move extends $pb.ProtobufEnum {
  static const Move MOVE_UNSPECIFIED =
      Move._(0, _omitEnumNames ? '' : 'MOVE_UNSPECIFIED');

  /// Add latency to one instance.
  static const Move MOVE_LATENCY =
      Move._(1, _omitEnumNames ? '' : 'MOVE_LATENCY');

  /// Fail a share of one instance's requests.
  static const Move MOVE_ERRORS =
      Move._(2, _omitEnumNames ? '' : 'MOVE_ERRORS');

  /// Stop one instance; it comes back by itself.
  static const Move MOVE_KILL = Move._(3, _omitEnumNames ? '' : 'MOVE_KILL');

  /// Fail the ledger's store underneath it.
  static const Move MOVE_STALL_STORE =
      Move._(4, _omitEnumNames ? '' : 'MOVE_STALL_STORE');

  static const $core.List<Move> values = <Move>[
    MOVE_UNSPECIFIED,
    MOVE_LATENCY,
    MOVE_ERRORS,
    MOVE_KILL,
    MOVE_STALL_STORE,
  ];

  static final $core.List<Move?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 4);
  static Move? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const Move._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
