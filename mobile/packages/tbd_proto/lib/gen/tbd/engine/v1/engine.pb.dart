// This is a generated file - do not edit.
//
// Generated from tbd/engine/v1/engine.proto.

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

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class EvaluateRequest extends $pb.GeneratedMessage {
  factory EvaluateRequest({
    $core.String? subjectId,
    $core.List<$core.int>? payload,
  }) {
    final result = EvaluateRequest._();
    if (subjectId != null) result.subjectId = subjectId;
    if (payload != null) result.payload = payload;
    return result;
  }

  EvaluateRequest._();

  factory EvaluateRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EvaluateRequest()..mergeFromBuffer(data, registry);
  factory EvaluateRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EvaluateRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluateRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.engine.v1'),
      createEmptyInstance: EvaluateRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'payload', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateRequest copyWith(void Function(EvaluateRequest) updates) =>
      super.copyWith((message) => updates(message as EvaluateRequest))
          as EvaluateRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use EvaluateRequest() / EvaluateRequest.new instead')
  static EvaluateRequest create() => EvaluateRequest._();
  static $pb.GeneratedMessage $_createMessage() => EvaluateRequest._();
  @$core.override
  EvaluateRequest createEmptyInstance() => EvaluateRequest._();
  @$core.pragma('dart2js:noInline')
  static EvaluateRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<EvaluateRequest>(
          EvaluateRequest.$_createMessage);
  static EvaluateRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get payload => $_getN(1);
  @$pb.TagNumber(2)
  set payload($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPayload() => $_has(1);
  @$pb.TagNumber(2)
  void clearPayload() => $_clearField(2);
}

class EvaluateResponse extends $pb.GeneratedMessage {
  factory EvaluateResponse({
    $core.String? subjectId,
    $core.double? score,
    $core.bool? stub,
    $core.String? modelVersion,
  }) {
    final result = EvaluateResponse._();
    if (subjectId != null) result.subjectId = subjectId;
    if (score != null) result.score = score;
    if (stub != null) result.stub = stub;
    if (modelVersion != null) result.modelVersion = modelVersion;
    return result;
  }

  EvaluateResponse._();

  factory EvaluateResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EvaluateResponse()..mergeFromBuffer(data, registry);
  factory EvaluateResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      EvaluateResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'EvaluateResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.engine.v1'),
      createEmptyInstance: EvaluateResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..aD(2, _omitFieldNames ? '' : 'score')
    ..aOB(3, _omitFieldNames ? '' : 'stub')
    ..aOS(4, _omitFieldNames ? '' : 'modelVersion')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  EvaluateResponse copyWith(void Function(EvaluateResponse) updates) =>
      super.copyWith((message) => updates(message as EvaluateResponse))
          as EvaluateResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use EvaluateResponse() / EvaluateResponse.new instead')
  static EvaluateResponse create() => EvaluateResponse._();
  static $pb.GeneratedMessage $_createMessage() => EvaluateResponse._();
  @$core.override
  EvaluateResponse createEmptyInstance() => EvaluateResponse._();
  @$core.pragma('dart2js:noInline')
  static EvaluateResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<EvaluateResponse>(
          EvaluateResponse.$_createMessage);
  static EvaluateResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.double get score => $_getN(1);
  @$pb.TagNumber(2)
  set score($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasScore() => $_has(1);
  @$pb.TagNumber(2)
  void clearScore() => $_clearField(2);

  /// True while the score comes from a placeholder, never from a model.
  /// Consumers must surface this flag; it is not optional metadata.
  @$pb.TagNumber(3)
  $core.bool get stub => $_getBF(2);
  @$pb.TagNumber(3)
  set stub($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasStub() => $_has(2);
  @$pb.TagNumber(3)
  void clearStub() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get modelVersion => $_getSZ(3);
  @$pb.TagNumber(4)
  set modelVersion($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasModelVersion() => $_has(3);
  @$pb.TagNumber(4)
  void clearModelVersion() => $_clearField(4);
}

class SubscribeRequest extends $pb.GeneratedMessage {
  factory SubscribeRequest({
    $core.String? subjectId,
  }) {
    final result = SubscribeRequest._();
    if (subjectId != null) result.subjectId = subjectId;
    return result;
  }

  SubscribeRequest._();

  factory SubscribeRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SubscribeRequest()..mergeFromBuffer(data, registry);
  factory SubscribeRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SubscribeRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SubscribeRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.engine.v1'),
      createEmptyInstance: SubscribeRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'subjectId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SubscribeRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SubscribeRequest copyWith(void Function(SubscribeRequest) updates) =>
      super.copyWith((message) => updates(message as SubscribeRequest))
          as SubscribeRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SubscribeRequest() / SubscribeRequest.new instead')
  static SubscribeRequest create() => SubscribeRequest._();
  static $pb.GeneratedMessage $_createMessage() => SubscribeRequest._();
  @$core.override
  SubscribeRequest createEmptyInstance() => SubscribeRequest._();
  @$core.pragma('dart2js:noInline')
  static SubscribeRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SubscribeRequest>(
          SubscribeRequest.$_createMessage);
  static SubscribeRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get subjectId => $_getSZ(0);
  @$pb.TagNumber(1)
  set subjectId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSubjectId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSubjectId() => $_clearField(1);
}

enum SubscribeResponse_Kind { heartbeat, scoreUpdated, notSet }

/// One event on a subscription.
class SubscribeResponse extends $pb.GeneratedMessage {
  factory SubscribeResponse({
    $core.String? id,
    $core.String? subjectId,
    $1.Timestamp? at,
    Heartbeat? heartbeat,
    ScoreUpdated? scoreUpdated,
  }) {
    final result = SubscribeResponse._();
    if (id != null) result.id = id;
    if (subjectId != null) result.subjectId = subjectId;
    if (at != null) result.at = at;
    if (heartbeat != null) result.heartbeat = heartbeat;
    if (scoreUpdated != null) result.scoreUpdated = scoreUpdated;
    return result;
  }

  SubscribeResponse._();

  factory SubscribeResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SubscribeResponse()..mergeFromBuffer(data, registry);
  factory SubscribeResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SubscribeResponse()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, SubscribeResponse_Kind>
      _SubscribeResponse_KindByTag = {
    10: SubscribeResponse_Kind.heartbeat,
    11: SubscribeResponse_Kind.scoreUpdated,
    0: SubscribeResponse_Kind.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SubscribeResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.engine.v1'),
      createEmptyInstance: SubscribeResponse.$_createMessage)
    ..oo(0, [10, 11])
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'subjectId')
    ..aOM<$1.Timestamp>(3, _omitFieldNames ? '' : 'at',
        subBuilder: $1.Timestamp.$_createMessage)
    ..aOM<Heartbeat>(10, _omitFieldNames ? '' : 'heartbeat',
        subBuilder: Heartbeat.$_createMessage)
    ..aOM<ScoreUpdated>(11, _omitFieldNames ? '' : 'scoreUpdated',
        subBuilder: ScoreUpdated.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SubscribeResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SubscribeResponse copyWith(void Function(SubscribeResponse) updates) =>
      super.copyWith((message) => updates(message as SubscribeResponse))
          as SubscribeResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SubscribeResponse() / SubscribeResponse.new instead')
  static SubscribeResponse create() => SubscribeResponse._();
  static $pb.GeneratedMessage $_createMessage() => SubscribeResponse._();
  @$core.override
  SubscribeResponse createEmptyInstance() => SubscribeResponse._();
  @$core.pragma('dart2js:noInline')
  static SubscribeResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SubscribeResponse>(
          SubscribeResponse.$_createMessage);
  static SubscribeResponse? _defaultInstance;

  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  SubscribeResponse_Kind whichKind() =>
      _SubscribeResponse_KindByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  void clearKind() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get subjectId => $_getSZ(1);
  @$pb.TagNumber(2)
  set subjectId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSubjectId() => $_has(1);
  @$pb.TagNumber(2)
  void clearSubjectId() => $_clearField(2);

  @$pb.TagNumber(3)
  $1.Timestamp get at => $_getN(2);
  @$pb.TagNumber(3)
  set at($1.Timestamp value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasAt() => $_has(2);
  @$pb.TagNumber(3)
  void clearAt() => $_clearField(3);
  @$pb.TagNumber(3)
  $1.Timestamp ensureAt() => $_ensure(2);

  @$pb.TagNumber(10)
  Heartbeat get heartbeat => $_getN(3);
  @$pb.TagNumber(10)
  set heartbeat(Heartbeat value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasHeartbeat() => $_has(3);
  @$pb.TagNumber(10)
  void clearHeartbeat() => $_clearField(10);
  @$pb.TagNumber(10)
  Heartbeat ensureHeartbeat() => $_ensure(3);

  @$pb.TagNumber(11)
  ScoreUpdated get scoreUpdated => $_getN(4);
  @$pb.TagNumber(11)
  set scoreUpdated(ScoreUpdated value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasScoreUpdated() => $_has(4);
  @$pb.TagNumber(11)
  void clearScoreUpdated() => $_clearField(11);
  @$pb.TagNumber(11)
  ScoreUpdated ensureScoreUpdated() => $_ensure(4);
}

class Heartbeat extends $pb.GeneratedMessage {
  factory Heartbeat({
    $fixnum.Int64? seq,
  }) {
    final result = Heartbeat._();
    if (seq != null) result.seq = seq;
    return result;
  }

  Heartbeat._();

  factory Heartbeat.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Heartbeat()..mergeFromBuffer(data, registry);
  factory Heartbeat.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Heartbeat()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Heartbeat',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.engine.v1'),
      createEmptyInstance: Heartbeat.$_createMessage)
    ..a<$fixnum.Int64>(1, _omitFieldNames ? '' : 'seq', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Heartbeat clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Heartbeat copyWith(void Function(Heartbeat) updates) =>
      super.copyWith((message) => updates(message as Heartbeat)) as Heartbeat;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Heartbeat() / Heartbeat.new instead')
  static Heartbeat create() => Heartbeat._();
  static $pb.GeneratedMessage $_createMessage() => Heartbeat._();
  @$core.override
  Heartbeat createEmptyInstance() => Heartbeat._();
  @$core.pragma('dart2js:noInline')
  static Heartbeat getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Heartbeat>(Heartbeat.$_createMessage);
  static Heartbeat? _defaultInstance;

  @$pb.TagNumber(1)
  $fixnum.Int64 get seq => $_getI64(0);
  @$pb.TagNumber(1)
  set seq($fixnum.Int64 value) => $_setInt64(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSeq() => $_has(0);
  @$pb.TagNumber(1)
  void clearSeq() => $_clearField(1);
}

class ScoreUpdated extends $pb.GeneratedMessage {
  factory ScoreUpdated({
    $core.double? score,
    $core.bool? stub,
  }) {
    final result = ScoreUpdated._();
    if (score != null) result.score = score;
    if (stub != null) result.stub = stub;
    return result;
  }

  ScoreUpdated._();

  factory ScoreUpdated.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ScoreUpdated()..mergeFromBuffer(data, registry);
  factory ScoreUpdated.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ScoreUpdated()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ScoreUpdated',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.engine.v1'),
      createEmptyInstance: ScoreUpdated.$_createMessage)
    ..aD(1, _omitFieldNames ? '' : 'score')
    ..aOB(2, _omitFieldNames ? '' : 'stub')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScoreUpdated clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScoreUpdated copyWith(void Function(ScoreUpdated) updates) =>
      super.copyWith((message) => updates(message as ScoreUpdated))
          as ScoreUpdated;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ScoreUpdated() / ScoreUpdated.new instead')
  static ScoreUpdated create() => ScoreUpdated._();
  static $pb.GeneratedMessage $_createMessage() => ScoreUpdated._();
  @$core.override
  ScoreUpdated createEmptyInstance() => ScoreUpdated._();
  @$core.pragma('dart2js:noInline')
  static ScoreUpdated getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ScoreUpdated>(
          ScoreUpdated.$_createMessage);
  static ScoreUpdated? _defaultInstance;

  @$pb.TagNumber(1)
  $core.double get score => $_getN(0);
  @$pb.TagNumber(1)
  set score($core.double value) => $_setDouble(0, value);
  @$pb.TagNumber(1)
  $core.bool hasScore() => $_has(0);
  @$pb.TagNumber(1)
  void clearScore() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.bool get stub => $_getBF(1);
  @$pb.TagNumber(2)
  set stub($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStub() => $_has(1);
  @$pb.TagNumber(2)
  void clearStub() => $_clearField(2);
}

enum SessionRequest_Body { data, heartbeat, close, notSet }

/// A frame from the client on a session.
class SessionRequest extends $pb.GeneratedMessage {
  factory SessionRequest({
    $core.String? sessionId,
    $fixnum.Int64? seq,
    $core.List<$core.int>? data,
    Heartbeat? heartbeat,
    Close? close,
  }) {
    final result = SessionRequest._();
    if (sessionId != null) result.sessionId = sessionId;
    if (seq != null) result.seq = seq;
    if (data != null) result.data = data;
    if (heartbeat != null) result.heartbeat = heartbeat;
    if (close != null) result.close = close;
    return result;
  }

  SessionRequest._();

  factory SessionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SessionRequest()..mergeFromBuffer(data, registry);
  factory SessionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SessionRequest()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, SessionRequest_Body>
      _SessionRequest_BodyByTag = {
    10: SessionRequest_Body.data,
    11: SessionRequest_Body.heartbeat,
    12: SessionRequest_Body.close,
    0: SessionRequest_Body.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SessionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.engine.v1'),
      createEmptyInstance: SessionRequest.$_createMessage)
    ..oo(0, [10, 11, 12])
    ..aOS(1, _omitFieldNames ? '' : 'sessionId')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'seq', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$core.List<$core.int>>(
        10, _omitFieldNames ? '' : 'data', $pb.PbFieldType.OY)
    ..aOM<Heartbeat>(11, _omitFieldNames ? '' : 'heartbeat',
        subBuilder: Heartbeat.$_createMessage)
    ..aOM<Close>(12, _omitFieldNames ? '' : 'close',
        subBuilder: Close.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionRequest copyWith(void Function(SessionRequest) updates) =>
      super.copyWith((message) => updates(message as SessionRequest))
          as SessionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SessionRequest() / SessionRequest.new instead')
  static SessionRequest create() => SessionRequest._();
  static $pb.GeneratedMessage $_createMessage() => SessionRequest._();
  @$core.override
  SessionRequest createEmptyInstance() => SessionRequest._();
  @$core.pragma('dart2js:noInline')
  static SessionRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SessionRequest>(
          SessionRequest.$_createMessage);
  static SessionRequest? _defaultInstance;

  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  SessionRequest_Body whichBody() =>
      _SessionRequest_BodyByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  void clearBody() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get sessionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set sessionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSessionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSessionId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get seq => $_getI64(1);
  @$pb.TagNumber(2)
  set seq($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSeq() => $_has(1);
  @$pb.TagNumber(2)
  void clearSeq() => $_clearField(2);

  @$pb.TagNumber(10)
  $core.List<$core.int> get data => $_getN(2);
  @$pb.TagNumber(10)
  set data($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(10)
  $core.bool hasData() => $_has(2);
  @$pb.TagNumber(10)
  void clearData() => $_clearField(10);

  @$pb.TagNumber(11)
  Heartbeat get heartbeat => $_getN(3);
  @$pb.TagNumber(11)
  set heartbeat(Heartbeat value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasHeartbeat() => $_has(3);
  @$pb.TagNumber(11)
  void clearHeartbeat() => $_clearField(11);
  @$pb.TagNumber(11)
  Heartbeat ensureHeartbeat() => $_ensure(3);

  @$pb.TagNumber(12)
  Close get close => $_getN(4);
  @$pb.TagNumber(12)
  set close(Close value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasClose() => $_has(4);
  @$pb.TagNumber(12)
  void clearClose() => $_clearField(12);
  @$pb.TagNumber(12)
  Close ensureClose() => $_ensure(4);
}

enum SessionResponse_Body { data, heartbeat, close, notSet }

/// A frame from the engine on a session.
class SessionResponse extends $pb.GeneratedMessage {
  factory SessionResponse({
    $core.String? sessionId,
    $fixnum.Int64? seq,
    $core.List<$core.int>? data,
    Heartbeat? heartbeat,
    Close? close,
  }) {
    final result = SessionResponse._();
    if (sessionId != null) result.sessionId = sessionId;
    if (seq != null) result.seq = seq;
    if (data != null) result.data = data;
    if (heartbeat != null) result.heartbeat = heartbeat;
    if (close != null) result.close = close;
    return result;
  }

  SessionResponse._();

  factory SessionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SessionResponse()..mergeFromBuffer(data, registry);
  factory SessionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SessionResponse()..mergeFromJson(json, registry);

  static const $core.Map<$core.int, SessionResponse_Body>
      _SessionResponse_BodyByTag = {
    10: SessionResponse_Body.data,
    11: SessionResponse_Body.heartbeat,
    12: SessionResponse_Body.close,
    0: SessionResponse_Body.notSet
  };
  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SessionResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.engine.v1'),
      createEmptyInstance: SessionResponse.$_createMessage)
    ..oo(0, [10, 11, 12])
    ..aOS(1, _omitFieldNames ? '' : 'sessionId')
    ..a<$fixnum.Int64>(2, _omitFieldNames ? '' : 'seq', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$core.List<$core.int>>(
        10, _omitFieldNames ? '' : 'data', $pb.PbFieldType.OY)
    ..aOM<Heartbeat>(11, _omitFieldNames ? '' : 'heartbeat',
        subBuilder: Heartbeat.$_createMessage)
    ..aOM<Close>(12, _omitFieldNames ? '' : 'close',
        subBuilder: Close.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SessionResponse copyWith(void Function(SessionResponse) updates) =>
      super.copyWith((message) => updates(message as SessionResponse))
          as SessionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SessionResponse() / SessionResponse.new instead')
  static SessionResponse create() => SessionResponse._();
  static $pb.GeneratedMessage $_createMessage() => SessionResponse._();
  @$core.override
  SessionResponse createEmptyInstance() => SessionResponse._();
  @$core.pragma('dart2js:noInline')
  static SessionResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SessionResponse>(
          SessionResponse.$_createMessage);
  static SessionResponse? _defaultInstance;

  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  SessionResponse_Body whichBody() =>
      _SessionResponse_BodyByTag[$_whichOneof(0)]!;
  @$pb.TagNumber(10)
  @$pb.TagNumber(11)
  @$pb.TagNumber(12)
  void clearBody() => $_clearField($_whichOneof(0));

  @$pb.TagNumber(1)
  $core.String get sessionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set sessionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasSessionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearSessionId() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get seq => $_getI64(1);
  @$pb.TagNumber(2)
  set seq($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSeq() => $_has(1);
  @$pb.TagNumber(2)
  void clearSeq() => $_clearField(2);

  @$pb.TagNumber(10)
  $core.List<$core.int> get data => $_getN(2);
  @$pb.TagNumber(10)
  set data($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(10)
  $core.bool hasData() => $_has(2);
  @$pb.TagNumber(10)
  void clearData() => $_clearField(10);

  @$pb.TagNumber(11)
  Heartbeat get heartbeat => $_getN(3);
  @$pb.TagNumber(11)
  set heartbeat(Heartbeat value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasHeartbeat() => $_has(3);
  @$pb.TagNumber(11)
  void clearHeartbeat() => $_clearField(11);
  @$pb.TagNumber(11)
  Heartbeat ensureHeartbeat() => $_ensure(3);

  @$pb.TagNumber(12)
  Close get close => $_getN(4);
  @$pb.TagNumber(12)
  set close(Close value) => $_setField(12, value);
  @$pb.TagNumber(12)
  $core.bool hasClose() => $_has(4);
  @$pb.TagNumber(12)
  void clearClose() => $_clearField(12);
  @$pb.TagNumber(12)
  Close ensureClose() => $_ensure(4);
}

class Close extends $pb.GeneratedMessage {
  factory Close({
    $core.String? reason,
  }) {
    final result = Close._();
    if (reason != null) result.reason = reason;
    return result;
  }

  Close._();

  factory Close.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Close()..mergeFromBuffer(data, registry);
  factory Close.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Close()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Close',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.engine.v1'),
      createEmptyInstance: Close.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'reason')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Close clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Close copyWith(void Function(Close) updates) =>
      super.copyWith((message) => updates(message as Close)) as Close;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Close() / Close.new instead')
  static Close create() => Close._();
  static $pb.GeneratedMessage $_createMessage() => Close._();
  @$core.override
  Close createEmptyInstance() => Close._();
  @$core.pragma('dart2js:noInline')
  static Close getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Close>(Close.$_createMessage);
  static Close? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get reason => $_getSZ(0);
  @$pb.TagNumber(1)
  set reason($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasReason() => $_has(0);
  @$pb.TagNumber(1)
  void clearReason() => $_clearField(1);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
