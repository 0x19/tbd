// This is a generated file - do not edit.
//
// Generated from tbd/ledger/v1/ledger.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:protobuf/protobuf.dart' as $pb;

/// Who or what produced a fact (docs/design/humans/001-sources.md).
class Source extends $pb.ProtobufEnum {
  static const Source SOURCE_UNSPECIFIED =
      Source._(0, _omitEnumNames ? '' : 'SOURCE_UNSPECIFIED');
  static const Source SOURCE_VERIFIED =
      Source._(1, _omitEnumNames ? '' : 'SOURCE_VERIFIED');
  static const Source SOURCE_DECLARED =
      Source._(2, _omitEnumNames ? '' : 'SOURCE_DECLARED');
  static const Source SOURCE_INFERRED =
      Source._(3, _omitEnumNames ? '' : 'SOURCE_INFERRED');
  static const Source SOURCE_SYMBOLIC =
      Source._(4, _omitEnumNames ? '' : 'SOURCE_SYMBOLIC');
  static const Source SOURCE_OBSERVED =
      Source._(5, _omitEnumNames ? '' : 'SOURCE_OBSERVED');

  static const $core.List<Source> values = <Source>[
    SOURCE_UNSPECIFIED,
    SOURCE_VERIFIED,
    SOURCE_DECLARED,
    SOURCE_INFERRED,
    SOURCE_SYMBOLIC,
    SOURCE_OBSERVED,
  ];

  static final $core.List<Source?> _byValue =
      $pb.ProtobufEnum.$_initByValueList(values, 5);
  static Source? valueOf($core.int value) =>
      value < 0 || value >= _byValue.length ? null : _byValue[value];

  const Source._(super.value, super.name);
}

const $core.bool _omitEnumNames =
    $core.bool.fromEnvironment('protobuf.omit_enum_names');
