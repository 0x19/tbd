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

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart'
    as $1;

import 'ledger.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'ledger.pbenum.dart';

class PingRequest extends $pb.GeneratedMessage {
  factory PingRequest({
    $core.String? message,
  }) {
    final result = PingRequest._();
    if (message != null) result.message = message;
    return result;
  }

  PingRequest._();

  factory PingRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PingRequest()..mergeFromBuffer(data, registry);
  factory PingRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PingRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PingRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: PingRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'message')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PingRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PingRequest copyWith(void Function(PingRequest) updates) =>
      super.copyWith((message) => updates(message as PingRequest))
          as PingRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use PingRequest() / PingRequest.new instead')
  static PingRequest create() => PingRequest._();
  static $pb.GeneratedMessage $_createMessage() => PingRequest._();
  @$core.override
  PingRequest createEmptyInstance() => PingRequest._();
  @$core.pragma('dart2js:noInline')
  static PingRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PingRequest>(
          PingRequest.$_createMessage);
  static PingRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get message => $_getSZ(0);
  @$pb.TagNumber(1)
  set message($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMessage() => $_has(0);
  @$pb.TagNumber(1)
  void clearMessage() => $_clearField(1);
}

class PingResponse extends $pb.GeneratedMessage {
  factory PingResponse({
    $core.String? message,
    $core.String? version,
    $core.bool? stub,
    $core.String? store,
  }) {
    final result = PingResponse._();
    if (message != null) result.message = message;
    if (version != null) result.version = version;
    if (stub != null) result.stub = stub;
    if (store != null) result.store = store;
    return result;
  }

  PingResponse._();

  factory PingResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PingResponse()..mergeFromBuffer(data, registry);
  factory PingResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PingResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PingResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: PingResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'message')
    ..aOS(2, _omitFieldNames ? '' : 'version')
    ..aOB(3, _omitFieldNames ? '' : 'stub')
    ..aOS(4, _omitFieldNames ? '' : 'store')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PingResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PingResponse copyWith(void Function(PingResponse) updates) =>
      super.copyWith((message) => updates(message as PingResponse))
          as PingResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use PingResponse() / PingResponse.new instead')
  static PingResponse create() => PingResponse._();
  static $pb.GeneratedMessage $_createMessage() => PingResponse._();
  @$core.override
  PingResponse createEmptyInstance() => PingResponse._();
  @$core.pragma('dart2js:noInline')
  static PingResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<PingResponse>(
          PingResponse.$_createMessage);
  static PingResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get message => $_getSZ(0);
  @$pb.TagNumber(1)
  set message($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMessage() => $_has(0);
  @$pb.TagNumber(1)
  void clearMessage() => $_clearField(1);

  /// Version of the binary that answered.
  @$pb.TagNumber(2)
  $core.String get version => $_getSZ(1);
  @$pb.TagNumber(2)
  set version($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasVersion() => $_has(1);
  @$pb.TagNumber(2)
  void clearVersion() => $_clearField(2);

  /// True while the service is a placeholder with nothing real behind it.
  @$pb.TagNumber(3)
  $core.bool get stub => $_getBF(2);
  @$pb.TagNumber(3)
  set stub($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStub() => $_has(2);
  @$pb.TagNumber(3)
  void clearStub() => $_clearField(3);

  /// Which store answers: `postgres` or `memory`.
  @$pb.TagNumber(4)
  $core.String get store => $_getSZ(3);
  @$pb.TagNumber(4)
  set store($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasStore() => $_has(3);
  @$pb.TagNumber(4)
  void clearStore() => $_clearField(4);
}

/// Bytes in an envelope format. Version 0 is labelled plaintext JSON; the
/// encryption phase adds versions, not fields. At most 64 KiB: larger is
/// INVALID_ARGUMENT, and a request over 256 KiB is OUT_OF_RANGE before it is read.
class Envelope extends $pb.GeneratedMessage {
  factory Envelope({
    $core.int? version,
    $core.List<$core.int>? bytes,
  }) {
    final result = Envelope._();
    if (version != null) result.version = version;
    if (bytes != null) result.bytes = bytes;
    return result;
  }

  Envelope._();

  factory Envelope.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Envelope()..mergeFromBuffer(data, registry);
  factory Envelope.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Envelope()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Envelope',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: Envelope.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'version', fieldType: $pb.PbFieldType.OU3)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'bytes', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Envelope clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Envelope copyWith(void Function(Envelope) updates) =>
      super.copyWith((message) => updates(message as Envelope)) as Envelope;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Envelope() / Envelope.new instead')
  static Envelope create() => Envelope._();
  static $pb.GeneratedMessage $_createMessage() => Envelope._();
  @$core.override
  Envelope createEmptyInstance() => Envelope._();
  @$core.pragma('dart2js:noInline')
  static Envelope getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Envelope>(Envelope.$_createMessage);
  static Envelope? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get version => $_getIZ(0);
  @$pb.TagNumber(1)
  set version($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasVersion() => $_has(0);
  @$pb.TagNumber(1)
  void clearVersion() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get bytes => $_getN(1);
  @$pb.TagNumber(2)
  set bytes($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasBytes() => $_has(1);
  @$pb.TagNumber(2)
  void clearBytes() => $_clearField(2);
}

/// A fact as the ledger holds it.
class Fact extends $pb.GeneratedMessage {
  factory Fact({
    $core.String? subjectId,
    $fixnum.Int64? id,
    $core.String? path,
    Source? source,
    Envelope? value,
    Envelope? origin,
    $core.double? confidence,
    $core.String? counterpartyId,
    $1.Timestamp? observedAt,
    $1.Timestamp? recordedAt,
    $1.Timestamp? expiresAt,
    $core.Iterable<$core.String>? consent,
    $core.bool? stub,
  }) {
    final result = Fact._();
    if (subjectId != null) result.subjectId = subjectId;
    if (id != null) result.id = id;
    if (path != null) result.path = path;
    if (source != null) result.source = source;
    if (value != null) result.value = value;
    if (origin != null) result.origin = origin;
    if (confidence != null) result.confidence = confidence;
    if (counterpartyId != null) result.counterpartyId = counterpartyId;
    if (observedAt != null) result.observedAt = observedAt;
    if (recordedAt != null) result.recordedAt = recordedAt;
    if (expiresAt != null) result.expiresAt = expiresAt;
    if (consent != null) result.consent.addAll(consent);
    if (stub != null) result.stub = stub;
    return result;
  }

  Fact._();

  factory Fact.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Fact()..mergeFromBuffer(data, registry);
  factory Fact.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Fact()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Fact',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: Fact.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..aInt64(2, _omitFieldNames ? '' : 'id')
    ..aOS(3, _omitFieldNames ? '' : 'path')
    ..aE<Source>(4, _omitFieldNames ? '' : 'source', enumValues: Source.values)
    ..aOM<Envelope>(5, _omitFieldNames ? '' : 'value',
        subBuilder: Envelope.$_createMessage)
    ..aOM<Envelope>(6, _omitFieldNames ? '' : 'origin',
        subBuilder: Envelope.$_createMessage)
    ..aD(7, _omitFieldNames ? '' : 'confidence', fieldType: $pb.PbFieldType.OF)
    ..aOS(8, _omitFieldNames ? '' : 'counterpartyId')
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'observedAt',
        subBuilder: $1.Timestamp.$_createMessage)
    ..aOM<$1.Timestamp>(10, _omitFieldNames ? '' : 'recordedAt',
        subBuilder: $1.Timestamp.$_createMessage)
    ..aOM<$1.Timestamp>(11, _omitFieldNames ? '' : 'expiresAt',
        subBuilder: $1.Timestamp.$_createMessage)
    ..pPS(12, _omitFieldNames ? '' : 'consent')
    ..aOB(13, _omitFieldNames ? '' : 'stub')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Fact clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Fact copyWith(void Function(Fact) updates) =>
      super.copyWith((message) => updates(message as Fact)) as Fact;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Fact() / Fact.new instead')
  static Fact create() => Fact._();
  static $pb.GeneratedMessage $_createMessage() => Fact._();
  @$core.override
  Fact createEmptyInstance() => Fact._();
  @$core.pragma('dart2js:noInline')
  static Fact getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Fact>(Fact.$_createMessage);
  static Fact? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get id => $_getI64(1);
  @$pb.TagNumber(2)
  set id($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get path => $_getSZ(2);
  @$pb.TagNumber(3)
  set path($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPath() => $_has(2);
  @$pb.TagNumber(3)
  void clearPath() => $_clearField(3);

  @$pb.TagNumber(4)
  Source get source => $_getN(3);
  @$pb.TagNumber(4)
  set source(Source value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasSource() => $_has(3);
  @$pb.TagNumber(4)
  void clearSource() => $_clearField(4);

  /// Absent on a tombstone.
  @$pb.TagNumber(5)
  Envelope get value => $_getN(4);
  @$pb.TagNumber(5)
  set value(Envelope value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasValue() => $_has(4);
  @$pb.TagNumber(5)
  void clearValue() => $_clearField(5);
  @$pb.TagNumber(5)
  Envelope ensureValue() => $_ensure(4);

  @$pb.TagNumber(6)
  Envelope get origin => $_getN(5);
  @$pb.TagNumber(6)
  set origin(Envelope value) => $_setField(6, value);
  @$pb.TagNumber(6)
  $core.bool hasOrigin() => $_has(5);
  @$pb.TagNumber(6)
  void clearOrigin() => $_clearField(6);
  @$pb.TagNumber(6)
  Envelope ensureOrigin() => $_ensure(5);

  @$pb.TagNumber(7)
  $core.double get confidence => $_getN(6);
  @$pb.TagNumber(7)
  set confidence($core.double value) => $_setFloat(6, value);
  @$pb.TagNumber(7)
  $core.bool hasConfidence() => $_has(6);
  @$pb.TagNumber(7)
  void clearConfidence() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get counterpartyId => $_getSZ(7);
  @$pb.TagNumber(8)
  set counterpartyId($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCounterpartyId() => $_has(7);
  @$pb.TagNumber(8)
  void clearCounterpartyId() => $_clearField(8);

  @$pb.TagNumber(9)
  $1.Timestamp get observedAt => $_getN(8);
  @$pb.TagNumber(9)
  set observedAt($1.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasObservedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearObservedAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.Timestamp ensureObservedAt() => $_ensure(8);

  @$pb.TagNumber(10)
  $1.Timestamp get recordedAt => $_getN(9);
  @$pb.TagNumber(10)
  set recordedAt($1.Timestamp value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasRecordedAt() => $_has(9);
  @$pb.TagNumber(10)
  void clearRecordedAt() => $_clearField(10);
  @$pb.TagNumber(10)
  $1.Timestamp ensureRecordedAt() => $_ensure(9);

  @$pb.TagNumber(11)
  $1.Timestamp get expiresAt => $_getN(10);
  @$pb.TagNumber(11)
  set expiresAt($1.Timestamp value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasExpiresAt() => $_has(10);
  @$pb.TagNumber(11)
  void clearExpiresAt() => $_clearField(11);
  @$pb.TagNumber(11)
  $1.Timestamp ensureExpiresAt() => $_ensure(10);

  @$pb.TagNumber(12)
  $pb.PbList<$core.String> get consent => $_getList(11);

  @$pb.TagNumber(13)
  $core.bool get stub => $_getBF(12);
  @$pb.TagNumber(13)
  set stub($core.bool value) => $_setBool(12, value);
  @$pb.TagNumber(13)
  $core.bool hasStub() => $_has(12);
  @$pb.TagNumber(13)
  void clearStub() => $_clearField(13);
}

class AppendRequest extends $pb.GeneratedMessage {
  factory AppendRequest({
    $core.String? subjectId,
    $core.String? path,
    Source? source,
    Envelope? value,
    Envelope? origin,
    $core.double? confidence,
    $core.String? counterpartyId,
    $1.Timestamp? observedAt,
    $1.Timestamp? expiresAt,
    $core.Iterable<$core.String>? consent,
    $core.bool? stub,
    $core.String? idempotencyKey,
  }) {
    final result = AppendRequest._();
    if (subjectId != null) result.subjectId = subjectId;
    if (path != null) result.path = path;
    if (source != null) result.source = source;
    if (value != null) result.value = value;
    if (origin != null) result.origin = origin;
    if (confidence != null) result.confidence = confidence;
    if (counterpartyId != null) result.counterpartyId = counterpartyId;
    if (observedAt != null) result.observedAt = observedAt;
    if (expiresAt != null) result.expiresAt = expiresAt;
    if (consent != null) result.consent.addAll(consent);
    if (stub != null) result.stub = stub;
    if (idempotencyKey != null) result.idempotencyKey = idempotencyKey;
    return result;
  }

  AppendRequest._();

  factory AppendRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AppendRequest()..mergeFromBuffer(data, registry);
  factory AppendRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AppendRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AppendRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: AppendRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..aOS(2, _omitFieldNames ? '' : 'path')
    ..aE<Source>(3, _omitFieldNames ? '' : 'source', enumValues: Source.values)
    ..aOM<Envelope>(4, _omitFieldNames ? '' : 'value',
        subBuilder: Envelope.$_createMessage)
    ..aOM<Envelope>(5, _omitFieldNames ? '' : 'origin',
        subBuilder: Envelope.$_createMessage)
    ..aD(6, _omitFieldNames ? '' : 'confidence', fieldType: $pb.PbFieldType.OF)
    ..aOS(7, _omitFieldNames ? '' : 'counterpartyId')
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'observedAt',
        subBuilder: $1.Timestamp.$_createMessage)
    ..aOM<$1.Timestamp>(9, _omitFieldNames ? '' : 'expiresAt',
        subBuilder: $1.Timestamp.$_createMessage)
    ..pPS(10, _omitFieldNames ? '' : 'consent')
    ..aOB(11, _omitFieldNames ? '' : 'stub')
    ..aOS(12, _omitFieldNames ? '' : 'idempotencyKey')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AppendRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AppendRequest copyWith(void Function(AppendRequest) updates) =>
      super.copyWith((message) => updates(message as AppendRequest))
          as AppendRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use AppendRequest() / AppendRequest.new instead')
  static AppendRequest create() => AppendRequest._();
  static $pb.GeneratedMessage $_createMessage() => AppendRequest._();
  @$core.override
  AppendRequest createEmptyInstance() => AppendRequest._();
  @$core.pragma('dart2js:noInline')
  static AppendRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<AppendRequest>(
          AppendRequest.$_createMessage);
  static AppendRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get path => $_getSZ(1);
  @$pb.TagNumber(2)
  set path($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPath() => $_has(1);
  @$pb.TagNumber(2)
  void clearPath() => $_clearField(2);

  @$pb.TagNumber(3)
  Source get source => $_getN(2);
  @$pb.TagNumber(3)
  set source(Source value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasSource() => $_has(2);
  @$pb.TagNumber(3)
  void clearSource() => $_clearField(3);

  @$pb.TagNumber(4)
  Envelope get value => $_getN(3);
  @$pb.TagNumber(4)
  set value(Envelope value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasValue() => $_has(3);
  @$pb.TagNumber(4)
  void clearValue() => $_clearField(4);
  @$pb.TagNumber(4)
  Envelope ensureValue() => $_ensure(3);

  @$pb.TagNumber(5)
  Envelope get origin => $_getN(4);
  @$pb.TagNumber(5)
  set origin(Envelope value) => $_setField(5, value);
  @$pb.TagNumber(5)
  $core.bool hasOrigin() => $_has(4);
  @$pb.TagNumber(5)
  void clearOrigin() => $_clearField(5);
  @$pb.TagNumber(5)
  Envelope ensureOrigin() => $_ensure(4);

  @$pb.TagNumber(6)
  $core.double get confidence => $_getN(5);
  @$pb.TagNumber(6)
  set confidence($core.double value) => $_setFloat(5, value);
  @$pb.TagNumber(6)
  $core.bool hasConfidence() => $_has(5);
  @$pb.TagNumber(6)
  void clearConfidence() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get counterpartyId => $_getSZ(6);
  @$pb.TagNumber(7)
  set counterpartyId($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasCounterpartyId() => $_has(6);
  @$pb.TagNumber(7)
  void clearCounterpartyId() => $_clearField(7);

  @$pb.TagNumber(8)
  $1.Timestamp get observedAt => $_getN(7);
  @$pb.TagNumber(8)
  set observedAt($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasObservedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearObservedAt() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureObservedAt() => $_ensure(7);

  @$pb.TagNumber(9)
  $1.Timestamp get expiresAt => $_getN(8);
  @$pb.TagNumber(9)
  set expiresAt($1.Timestamp value) => $_setField(9, value);
  @$pb.TagNumber(9)
  $core.bool hasExpiresAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearExpiresAt() => $_clearField(9);
  @$pb.TagNumber(9)
  $1.Timestamp ensureExpiresAt() => $_ensure(8);

  @$pb.TagNumber(10)
  $pb.PbList<$core.String> get consent => $_getList(9);

  @$pb.TagNumber(11)
  $core.bool get stub => $_getBF(10);
  @$pb.TagNumber(11)
  set stub($core.bool value) => $_setBool(10, value);
  @$pb.TagNumber(11)
  $core.bool hasStub() => $_has(10);
  @$pb.TagNumber(11)
  void clearStub() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get idempotencyKey => $_getSZ(11);
  @$pb.TagNumber(12)
  set idempotencyKey($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasIdempotencyKey() => $_has(11);
  @$pb.TagNumber(12)
  void clearIdempotencyKey() => $_clearField(12);
}

class AppendResponse extends $pb.GeneratedMessage {
  factory AppendResponse({
    Fact? fact,
    $core.bool? replayed,
  }) {
    final result = AppendResponse._();
    if (fact != null) result.fact = fact;
    if (replayed != null) result.replayed = replayed;
    return result;
  }

  AppendResponse._();

  factory AppendResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AppendResponse()..mergeFromBuffer(data, registry);
  factory AppendResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      AppendResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'AppendResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: AppendResponse.$_createMessage)
    ..aOM<Fact>(1, _omitFieldNames ? '' : 'fact',
        subBuilder: Fact.$_createMessage)
    ..aOB(2, _omitFieldNames ? '' : 'replayed')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AppendResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  AppendResponse copyWith(void Function(AppendResponse) updates) =>
      super.copyWith((message) => updates(message as AppendResponse))
          as AppendResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use AppendResponse() / AppendResponse.new instead')
  static AppendResponse create() => AppendResponse._();
  static $pb.GeneratedMessage $_createMessage() => AppendResponse._();
  @$core.override
  AppendResponse createEmptyInstance() => AppendResponse._();
  @$core.pragma('dart2js:noInline')
  static AppendResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<AppendResponse>(
          AppendResponse.$_createMessage);
  static AppendResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Fact get fact => $_getN(0);
  @$pb.TagNumber(1)
  set fact(Fact value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasFact() => $_has(0);
  @$pb.TagNumber(1)
  void clearFact() => $_clearField(1);
  @$pb.TagNumber(1)
  Fact ensureFact() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.bool get replayed => $_getBF(1);
  @$pb.TagNumber(2)
  set replayed($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasReplayed() => $_has(1);
  @$pb.TagNumber(2)
  void clearReplayed() => $_clearField(2);
}

class CurrentRequest extends $pb.GeneratedMessage {
  factory CurrentRequest({
    $core.String? subjectId,
    $core.Iterable<$core.String>? paths,
    $core.Iterable<Source>? sources,
    $core.Iterable<$core.String>? scopes,
    $core.String? cursor,
    $core.int? limit,
  }) {
    final result = CurrentRequest._();
    if (subjectId != null) result.subjectId = subjectId;
    if (paths != null) result.paths.addAll(paths);
    if (sources != null) result.sources.addAll(sources);
    if (scopes != null) result.scopes.addAll(scopes);
    if (cursor != null) result.cursor = cursor;
    if (limit != null) result.limit = limit;
    return result;
  }

  CurrentRequest._();

  factory CurrentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CurrentRequest()..mergeFromBuffer(data, registry);
  factory CurrentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CurrentRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CurrentRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: CurrentRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..pPS(2, _omitFieldNames ? '' : 'paths')
    ..pc<Source>(3, _omitFieldNames ? '' : 'sources', $pb.PbFieldType.KE,
        valueOf: Source.valueOf,
        enumValues: Source.values,
        defaultEnumValue: Source.SOURCE_UNSPECIFIED)
    ..pPS(4, _omitFieldNames ? '' : 'scopes')
    ..aOS(5, _omitFieldNames ? '' : 'cursor')
    ..aI(6, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CurrentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CurrentRequest copyWith(void Function(CurrentRequest) updates) =>
      super.copyWith((message) => updates(message as CurrentRequest))
          as CurrentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CurrentRequest() / CurrentRequest.new instead')
  static CurrentRequest create() => CurrentRequest._();
  static $pb.GeneratedMessage $_createMessage() => CurrentRequest._();
  @$core.override
  CurrentRequest createEmptyInstance() => CurrentRequest._();
  @$core.pragma('dart2js:noInline')
  static CurrentRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CurrentRequest>(
          CurrentRequest.$_createMessage);
  static CurrentRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);

  /// `traits.warmth` or `traits.*`; empty means every path.
  @$pb.TagNumber(2)
  $pb.PbList<$core.String> get paths => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbList<Source> get sources => $_getList(2);

  /// Consent scopes the caller reads under. Required and non-empty.
  @$pb.TagNumber(4)
  $pb.PbList<$core.String> get scopes => $_getList(3);

  @$pb.TagNumber(5)
  $core.String get cursor => $_getSZ(4);
  @$pb.TagNumber(5)
  set cursor($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCursor() => $_has(4);
  @$pb.TagNumber(5)
  void clearCursor() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get limit => $_getIZ(5);
  @$pb.TagNumber(6)
  set limit($core.int value) => $_setUnsignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasLimit() => $_has(5);
  @$pb.TagNumber(6)
  void clearLimit() => $_clearField(6);
}

class CurrentResponse extends $pb.GeneratedMessage {
  factory CurrentResponse({
    $core.Iterable<Fact>? facts,
    $core.String? next,
  }) {
    final result = CurrentResponse._();
    if (facts != null) result.facts.addAll(facts);
    if (next != null) result.next = next;
    return result;
  }

  CurrentResponse._();

  factory CurrentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CurrentResponse()..mergeFromBuffer(data, registry);
  factory CurrentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CurrentResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CurrentResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: CurrentResponse.$_createMessage)
    ..pPM<Fact>(1, _omitFieldNames ? '' : 'facts',
        subBuilder: Fact.$_createMessage)
    ..aOS(2, _omitFieldNames ? '' : 'next')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CurrentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CurrentResponse copyWith(void Function(CurrentResponse) updates) =>
      super.copyWith((message) => updates(message as CurrentResponse))
          as CurrentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CurrentResponse() / CurrentResponse.new instead')
  static CurrentResponse create() => CurrentResponse._();
  static $pb.GeneratedMessage $_createMessage() => CurrentResponse._();
  @$core.override
  CurrentResponse createEmptyInstance() => CurrentResponse._();
  @$core.pragma('dart2js:noInline')
  static CurrentResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<CurrentResponse>(
          CurrentResponse.$_createMessage);
  static CurrentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Fact> get facts => $_getList(0);

  /// Cursor for the next page; empty when this was the last.
  @$pb.TagNumber(2)
  $core.String get next => $_getSZ(1);
  @$pb.TagNumber(2)
  set next($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNext() => $_has(1);
  @$pb.TagNumber(2)
  void clearNext() => $_clearField(2);
}

class HistoryRequest extends $pb.GeneratedMessage {
  factory HistoryRequest({
    $core.String? subjectId,
    $core.Iterable<$core.String>? paths,
    $core.Iterable<Source>? sources,
    $core.Iterable<$core.String>? scopes,
    $core.String? cursor,
    $core.int? limit,
    $1.Timestamp? at,
  }) {
    final result = HistoryRequest._();
    if (subjectId != null) result.subjectId = subjectId;
    if (paths != null) result.paths.addAll(paths);
    if (sources != null) result.sources.addAll(sources);
    if (scopes != null) result.scopes.addAll(scopes);
    if (cursor != null) result.cursor = cursor;
    if (limit != null) result.limit = limit;
    if (at != null) result.at = at;
    return result;
  }

  HistoryRequest._();

  factory HistoryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HistoryRequest()..mergeFromBuffer(data, registry);
  factory HistoryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HistoryRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'HistoryRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: HistoryRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..pPS(2, _omitFieldNames ? '' : 'paths')
    ..pc<Source>(3, _omitFieldNames ? '' : 'sources', $pb.PbFieldType.KE,
        valueOf: Source.valueOf,
        enumValues: Source.values,
        defaultEnumValue: Source.SOURCE_UNSPECIFIED)
    ..pPS(4, _omitFieldNames ? '' : 'scopes')
    ..aOS(5, _omitFieldNames ? '' : 'cursor')
    ..aI(6, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..aOM<$1.Timestamp>(7, _omitFieldNames ? '' : 'at',
        subBuilder: $1.Timestamp.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HistoryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HistoryRequest copyWith(void Function(HistoryRequest) updates) =>
      super.copyWith((message) => updates(message as HistoryRequest))
          as HistoryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use HistoryRequest() / HistoryRequest.new instead')
  static HistoryRequest create() => HistoryRequest._();
  static $pb.GeneratedMessage $_createMessage() => HistoryRequest._();
  @$core.override
  HistoryRequest createEmptyInstance() => HistoryRequest._();
  @$core.pragma('dart2js:noInline')
  static HistoryRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<HistoryRequest>(
          HistoryRequest.$_createMessage);
  static HistoryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.String> get paths => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbList<Source> get sources => $_getList(2);

  @$pb.TagNumber(4)
  $pb.PbList<$core.String> get scopes => $_getList(3);

  @$pb.TagNumber(5)
  $core.String get cursor => $_getSZ(4);
  @$pb.TagNumber(5)
  set cursor($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCursor() => $_has(4);
  @$pb.TagNumber(5)
  void clearCursor() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get limit => $_getIZ(5);
  @$pb.TagNumber(6)
  set limit($core.int value) => $_setUnsignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasLimit() => $_has(5);
  @$pb.TagNumber(6)
  void clearLimit() => $_clearField(6);

  /// Facts recorded at or before this instant.
  @$pb.TagNumber(7)
  $1.Timestamp get at => $_getN(6);
  @$pb.TagNumber(7)
  set at($1.Timestamp value) => $_setField(7, value);
  @$pb.TagNumber(7)
  $core.bool hasAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearAt() => $_clearField(7);
  @$pb.TagNumber(7)
  $1.Timestamp ensureAt() => $_ensure(6);
}

class HistoryResponse extends $pb.GeneratedMessage {
  factory HistoryResponse({
    $core.Iterable<Fact>? facts,
    $core.String? next,
  }) {
    final result = HistoryResponse._();
    if (facts != null) result.facts.addAll(facts);
    if (next != null) result.next = next;
    return result;
  }

  HistoryResponse._();

  factory HistoryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HistoryResponse()..mergeFromBuffer(data, registry);
  factory HistoryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      HistoryResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'HistoryResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: HistoryResponse.$_createMessage)
    ..pPM<Fact>(1, _omitFieldNames ? '' : 'facts',
        subBuilder: Fact.$_createMessage)
    ..aOS(2, _omitFieldNames ? '' : 'next')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HistoryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  HistoryResponse copyWith(void Function(HistoryResponse) updates) =>
      super.copyWith((message) => updates(message as HistoryResponse))
          as HistoryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use HistoryResponse() / HistoryResponse.new instead')
  static HistoryResponse create() => HistoryResponse._();
  static $pb.GeneratedMessage $_createMessage() => HistoryResponse._();
  @$core.override
  HistoryResponse createEmptyInstance() => HistoryResponse._();
  @$core.pragma('dart2js:noInline')
  static HistoryResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<HistoryResponse>(
          HistoryResponse.$_createMessage);
  static HistoryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Fact> get facts => $_getList(0);

  @$pb.TagNumber(2)
  $core.String get next => $_getSZ(1);
  @$pb.TagNumber(2)
  set next($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNext() => $_has(1);
  @$pb.TagNumber(2)
  void clearNext() => $_clearField(2);
}

class RetractRequest extends $pb.GeneratedMessage {
  factory RetractRequest({
    $core.String? subjectId,
    $core.String? path,
    Source? source,
    Envelope? origin,
  }) {
    final result = RetractRequest._();
    if (subjectId != null) result.subjectId = subjectId;
    if (path != null) result.path = path;
    if (source != null) result.source = source;
    if (origin != null) result.origin = origin;
    return result;
  }

  RetractRequest._();

  factory RetractRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RetractRequest()..mergeFromBuffer(data, registry);
  factory RetractRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RetractRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RetractRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: RetractRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..aOS(2, _omitFieldNames ? '' : 'path')
    ..aE<Source>(3, _omitFieldNames ? '' : 'source', enumValues: Source.values)
    ..aOM<Envelope>(4, _omitFieldNames ? '' : 'origin',
        subBuilder: Envelope.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RetractRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RetractRequest copyWith(void Function(RetractRequest) updates) =>
      super.copyWith((message) => updates(message as RetractRequest))
          as RetractRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RetractRequest() / RetractRequest.new instead')
  static RetractRequest create() => RetractRequest._();
  static $pb.GeneratedMessage $_createMessage() => RetractRequest._();
  @$core.override
  RetractRequest createEmptyInstance() => RetractRequest._();
  @$core.pragma('dart2js:noInline')
  static RetractRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RetractRequest>(
          RetractRequest.$_createMessage);
  static RetractRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get path => $_getSZ(1);
  @$pb.TagNumber(2)
  set path($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPath() => $_has(1);
  @$pb.TagNumber(2)
  void clearPath() => $_clearField(2);

  @$pb.TagNumber(3)
  Source get source => $_getN(2);
  @$pb.TagNumber(3)
  set source(Source value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasSource() => $_has(2);
  @$pb.TagNumber(3)
  void clearSource() => $_clearField(3);

  @$pb.TagNumber(4)
  Envelope get origin => $_getN(3);
  @$pb.TagNumber(4)
  set origin(Envelope value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasOrigin() => $_has(3);
  @$pb.TagNumber(4)
  void clearOrigin() => $_clearField(4);
  @$pb.TagNumber(4)
  Envelope ensureOrigin() => $_ensure(3);
}

class RetractResponse extends $pb.GeneratedMessage {
  factory RetractResponse({
    Fact? tombstone,
  }) {
    final result = RetractResponse._();
    if (tombstone != null) result.tombstone = tombstone;
    return result;
  }

  RetractResponse._();

  factory RetractResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RetractResponse()..mergeFromBuffer(data, registry);
  factory RetractResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RetractResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RetractResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: RetractResponse.$_createMessage)
    ..aOM<Fact>(1, _omitFieldNames ? '' : 'tombstone',
        subBuilder: Fact.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RetractResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RetractResponse copyWith(void Function(RetractResponse) updates) =>
      super.copyWith((message) => updates(message as RetractResponse))
          as RetractResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RetractResponse() / RetractResponse.new instead')
  static RetractResponse create() => RetractResponse._();
  static $pb.GeneratedMessage $_createMessage() => RetractResponse._();
  @$core.override
  RetractResponse createEmptyInstance() => RetractResponse._();
  @$core.pragma('dart2js:noInline')
  static RetractResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RetractResponse>(
          RetractResponse.$_createMessage);
  static RetractResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Fact get tombstone => $_getN(0);
  @$pb.TagNumber(1)
  set tombstone(Fact value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTombstone() => $_has(0);
  @$pb.TagNumber(1)
  void clearTombstone() => $_clearField(1);
  @$pb.TagNumber(1)
  Fact ensureTombstone() => $_ensure(0);
}

class EraseRequest extends $pb.GeneratedMessage {
  factory EraseRequest({
    $core.String? subjectId,
  }) {
    final result = EraseRequest._();
    if (subjectId != null) result.subjectId = subjectId;
    return result;
  }

  EraseRequest._();

  factory EraseRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EraseRequest()..mergeFromBuffer(data, registry);
  factory EraseRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EraseRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EraseRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: EraseRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EraseRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EraseRequest copyWith(void Function(EraseRequest) updates) =>
      super.copyWith((message) => updates(message as EraseRequest))
          as EraseRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use EraseRequest() / EraseRequest.new instead')
  static EraseRequest create() => EraseRequest._();
  static $pb.GeneratedMessage $_createMessage() => EraseRequest._();
  @$core.override
  EraseRequest createEmptyInstance() => EraseRequest._();
  @$core.pragma('dart2js:noInline')
  static EraseRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<EraseRequest>(
          EraseRequest.$_createMessage);
  static EraseRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);
}

class EraseResponse extends $pb.GeneratedMessage {
  factory EraseResponse({
    $1.Timestamp? requestedAt,
    $1.Timestamp? executesAfter,
  }) {
    final result = EraseResponse._();
    if (requestedAt != null) result.requestedAt = requestedAt;
    if (executesAfter != null) result.executesAfter = executesAfter;
    return result;
  }

  EraseResponse._();

  factory EraseResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EraseResponse()..mergeFromBuffer(data, registry);
  factory EraseResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EraseResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EraseResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: EraseResponse.$_createMessage)
    ..aOM<$1.Timestamp>(1, _omitFieldNames ? '' : 'requestedAt',
        subBuilder: $1.Timestamp.$_createMessage)
    ..aOM<$1.Timestamp>(2, _omitFieldNames ? '' : 'executesAfter',
        subBuilder: $1.Timestamp.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EraseResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EraseResponse copyWith(void Function(EraseResponse) updates) =>
      super.copyWith((message) => updates(message as EraseResponse))
          as EraseResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use EraseResponse() / EraseResponse.new instead')
  static EraseResponse create() => EraseResponse._();
  static $pb.GeneratedMessage $_createMessage() => EraseResponse._();
  @$core.override
  EraseResponse createEmptyInstance() => EraseResponse._();
  @$core.pragma('dart2js:noInline')
  static EraseResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<EraseResponse>(
          EraseResponse.$_createMessage);
  static EraseResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $1.Timestamp get requestedAt => $_getN(0);
  @$pb.TagNumber(1)
  set requestedAt($1.Timestamp value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRequestedAt() => $_has(0);
  @$pb.TagNumber(1)
  void clearRequestedAt() => $_clearField(1);
  @$pb.TagNumber(1)
  $1.Timestamp ensureRequestedAt() => $_ensure(0);

  /// When the cascade may run: requested_at plus the grace window.
  @$pb.TagNumber(2)
  $1.Timestamp get executesAfter => $_getN(1);
  @$pb.TagNumber(2)
  set executesAfter($1.Timestamp value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasExecutesAfter() => $_has(1);
  @$pb.TagNumber(2)
  void clearExecutesAfter() => $_clearField(2);
  @$pb.TagNumber(2)
  $1.Timestamp ensureExecutesAfter() => $_ensure(1);
}

class RestoreRequest extends $pb.GeneratedMessage {
  factory RestoreRequest({
    $core.String? subjectId,
  }) {
    final result = RestoreRequest._();
    if (subjectId != null) result.subjectId = subjectId;
    return result;
  }

  RestoreRequest._();

  factory RestoreRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RestoreRequest()..mergeFromBuffer(data, registry);
  factory RestoreRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RestoreRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RestoreRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: RestoreRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RestoreRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RestoreRequest copyWith(void Function(RestoreRequest) updates) =>
      super.copyWith((message) => updates(message as RestoreRequest))
          as RestoreRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RestoreRequest() / RestoreRequest.new instead')
  static RestoreRequest create() => RestoreRequest._();
  static $pb.GeneratedMessage $_createMessage() => RestoreRequest._();
  @$core.override
  RestoreRequest createEmptyInstance() => RestoreRequest._();
  @$core.pragma('dart2js:noInline')
  static RestoreRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RestoreRequest>(
          RestoreRequest.$_createMessage);
  static RestoreRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);
}

class RestoreResponse extends $pb.GeneratedMessage {
  factory RestoreResponse() => RestoreResponse._();

  RestoreResponse._();

  factory RestoreResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RestoreResponse()..mergeFromBuffer(data, registry);
  factory RestoreResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RestoreResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RestoreResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.ledger.v1'),
      createEmptyInstance: RestoreResponse.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RestoreResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RestoreResponse copyWith(void Function(RestoreResponse) updates) =>
      super.copyWith((message) => updates(message as RestoreResponse))
          as RestoreResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use RestoreResponse() / RestoreResponse.new instead')
  static RestoreResponse create() => RestoreResponse._();
  static $pb.GeneratedMessage $_createMessage() => RestoreResponse._();
  @$core.override
  RestoreResponse createEmptyInstance() => RestoreResponse._();
  @$core.pragma('dart2js:noInline')
  static RestoreResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<RestoreResponse>(
          RestoreResponse.$_createMessage);
  static RestoreResponse? _defaultInstance;
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
