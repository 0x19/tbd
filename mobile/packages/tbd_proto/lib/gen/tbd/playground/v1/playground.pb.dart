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

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;
import 'package:protobuf/well_known_types/google/protobuf/timestamp.pb.dart'
    as $1;

import 'playground.pbenum.dart';

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

export 'playground.pbenum.dart';

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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
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
  }) {
    final result = PingResponse._();
    if (message != null) result.message = message;
    if (version != null) result.version = version;
    if (stub != null) result.stub = stub;
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
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: PingResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'message')
    ..aOS(2, _omitFieldNames ? '' : 'version')
    ..aOB(3, _omitFieldNames ? '' : 'stub')
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
}

class GetWorldRequest extends $pb.GeneratedMessage {
  factory GetWorldRequest() => GetWorldRequest._();

  GetWorldRequest._();

  factory GetWorldRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetWorldRequest()..mergeFromBuffer(data, registry);
  factory GetWorldRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetWorldRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetWorldRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: GetWorldRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetWorldRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetWorldRequest copyWith(void Function(GetWorldRequest) updates) =>
      super.copyWith((message) => updates(message as GetWorldRequest))
          as GetWorldRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetWorldRequest() / GetWorldRequest.new instead')
  static GetWorldRequest create() => GetWorldRequest._();
  static $pb.GeneratedMessage $_createMessage() => GetWorldRequest._();
  @$core.override
  GetWorldRequest createEmptyInstance() => GetWorldRequest._();
  @$core.pragma('dart2js:noInline')
  static GetWorldRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetWorldRequest>(
          GetWorldRequest.$_createMessage);
  static GetWorldRequest? _defaultInstance;
}

class GetWorldResponse extends $pb.GeneratedMessage {
  factory GetWorldResponse({
    World? world,
  }) {
    final result = GetWorldResponse._();
    if (world != null) result.world = world;
    return result;
  }

  GetWorldResponse._();

  factory GetWorldResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetWorldResponse()..mergeFromBuffer(data, registry);
  factory GetWorldResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetWorldResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetWorldResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: GetWorldResponse.$_createMessage)
    ..aOM<World>(1, _omitFieldNames ? '' : 'world',
        subBuilder: World.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetWorldResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetWorldResponse copyWith(void Function(GetWorldResponse) updates) =>
      super.copyWith((message) => updates(message as GetWorldResponse))
          as GetWorldResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetWorldResponse() / GetWorldResponse.new instead')
  static GetWorldResponse create() => GetWorldResponse._();
  static $pb.GeneratedMessage $_createMessage() => GetWorldResponse._();
  @$core.override
  GetWorldResponse createEmptyInstance() => GetWorldResponse._();
  @$core.pragma('dart2js:noInline')
  static GetWorldResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetWorldResponse>(
          GetWorldResponse.$_createMessage);
  static GetWorldResponse? _defaultInstance;

  @$pb.TagNumber(1)
  World get world => $_getN(0);
  @$pb.TagNumber(1)
  set world(World value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasWorld() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorld() => $_clearField(1);
  @$pb.TagNumber(1)
  World ensureWorld() => $_ensure(0);
}

class WatchRequest extends $pb.GeneratedMessage {
  factory WatchRequest() => WatchRequest._();

  WatchRequest._();

  factory WatchRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      WatchRequest()..mergeFromBuffer(data, registry);
  factory WatchRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      WatchRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WatchRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: WatchRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchRequest copyWith(void Function(WatchRequest) updates) =>
      super.copyWith((message) => updates(message as WatchRequest))
          as WatchRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use WatchRequest() / WatchRequest.new instead')
  static WatchRequest create() => WatchRequest._();
  static $pb.GeneratedMessage $_createMessage() => WatchRequest._();
  @$core.override
  WatchRequest createEmptyInstance() => WatchRequest._();
  @$core.pragma('dart2js:noInline')
  static WatchRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<WatchRequest>(
          WatchRequest.$_createMessage);
  static WatchRequest? _defaultInstance;
}

class WatchResponse extends $pb.GeneratedMessage {
  factory WatchResponse({
    World? world,
  }) {
    final result = WatchResponse._();
    if (world != null) result.world = world;
    return result;
  }

  WatchResponse._();

  factory WatchResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      WatchResponse()..mergeFromBuffer(data, registry);
  factory WatchResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      WatchResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WatchResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: WatchResponse.$_createMessage)
    ..aOM<World>(1, _omitFieldNames ? '' : 'world',
        subBuilder: World.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchResponse copyWith(void Function(WatchResponse) updates) =>
      super.copyWith((message) => updates(message as WatchResponse))
          as WatchResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use WatchResponse() / WatchResponse.new instead')
  static WatchResponse create() => WatchResponse._();
  static $pb.GeneratedMessage $_createMessage() => WatchResponse._();
  @$core.override
  WatchResponse createEmptyInstance() => WatchResponse._();
  @$core.pragma('dart2js:noInline')
  static WatchResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<WatchResponse>(
          WatchResponse.$_createMessage);
  static WatchResponse? _defaultInstance;

  @$pb.TagNumber(1)
  World get world => $_getN(0);
  @$pb.TagNumber(1)
  set world(World value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasWorld() => $_has(0);
  @$pb.TagNumber(1)
  void clearWorld() => $_clearField(1);
  @$pb.TagNumber(1)
  World ensureWorld() => $_ensure(0);
}

class World extends $pb.GeneratedMessage {
  factory World({
    Health? health,
    Objective? objective,
    Budget? budget,
    Traffic? traffic,
    $core.Iterable<Instance>? instances,
    $core.Iterable<ActiveFault>? faults,
    $core.int? heldForSeconds,
    $1.Timestamp? now,
    $core.int? healingSeconds,
  }) {
    final result = World._();
    if (health != null) result.health = health;
    if (objective != null) result.objective = objective;
    if (budget != null) result.budget = budget;
    if (traffic != null) result.traffic = traffic;
    if (instances != null) result.instances.addAll(instances);
    if (faults != null) result.faults.addAll(faults);
    if (heldForSeconds != null) result.heldForSeconds = heldForSeconds;
    if (now != null) result.now = now;
    if (healingSeconds != null) result.healingSeconds = healingSeconds;
    return result;
  }

  World._();

  factory World.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      World()..mergeFromBuffer(data, registry);
  factory World.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      World()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'World',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: World.$_createMessage)
    ..aE<Health>(1, _omitFieldNames ? '' : 'health', enumValues: Health.values)
    ..aOM<Objective>(2, _omitFieldNames ? '' : 'objective',
        subBuilder: Objective.$_createMessage)
    ..aOM<Budget>(3, _omitFieldNames ? '' : 'budget',
        subBuilder: Budget.$_createMessage)
    ..aOM<Traffic>(4, _omitFieldNames ? '' : 'traffic',
        subBuilder: Traffic.$_createMessage)
    ..pPM<Instance>(5, _omitFieldNames ? '' : 'instances',
        subBuilder: Instance.$_createMessage)
    ..pPM<ActiveFault>(6, _omitFieldNames ? '' : 'faults',
        subBuilder: ActiveFault.$_createMessage)
    ..aI(7, _omitFieldNames ? '' : 'heldForSeconds',
        fieldType: $pb.PbFieldType.OU3)
    ..aOM<$1.Timestamp>(8, _omitFieldNames ? '' : 'now',
        subBuilder: $1.Timestamp.$_createMessage)
    ..aI(9, _omitFieldNames ? '' : 'healingSeconds',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  World clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  World copyWith(void Function(World) updates) =>
      super.copyWith((message) => updates(message as World)) as World;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use World() / World.new instead')
  static World create() => World._();
  static $pb.GeneratedMessage $_createMessage() => World._();
  @$core.override
  World createEmptyInstance() => World._();
  @$core.pragma('dart2js:noInline')
  static World getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<World>(World.$_createMessage);
  static World? _defaultInstance;

  @$pb.TagNumber(1)
  Health get health => $_getN(0);
  @$pb.TagNumber(1)
  set health(Health value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasHealth() => $_has(0);
  @$pb.TagNumber(1)
  void clearHealth() => $_clearField(1);

  @$pb.TagNumber(2)
  Objective get objective => $_getN(1);
  @$pb.TagNumber(2)
  set objective(Objective value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasObjective() => $_has(1);
  @$pb.TagNumber(2)
  void clearObjective() => $_clearField(2);
  @$pb.TagNumber(2)
  Objective ensureObjective() => $_ensure(1);

  @$pb.TagNumber(3)
  Budget get budget => $_getN(2);
  @$pb.TagNumber(3)
  set budget(Budget value) => $_setField(3, value);
  @$pb.TagNumber(3)
  $core.bool hasBudget() => $_has(2);
  @$pb.TagNumber(3)
  void clearBudget() => $_clearField(3);
  @$pb.TagNumber(3)
  Budget ensureBudget() => $_ensure(2);

  @$pb.TagNumber(4)
  Traffic get traffic => $_getN(3);
  @$pb.TagNumber(4)
  set traffic(Traffic value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasTraffic() => $_has(3);
  @$pb.TagNumber(4)
  void clearTraffic() => $_clearField(4);
  @$pb.TagNumber(4)
  Traffic ensureTraffic() => $_ensure(3);

  @$pb.TagNumber(5)
  $pb.PbList<Instance> get instances => $_getList(4);

  @$pb.TagNumber(6)
  $pb.PbList<ActiveFault> get faults => $_getList(5);

  /// How long the world has been in its current health.
  @$pb.TagNumber(7)
  $core.int get heldForSeconds => $_getIZ(6);
  @$pb.TagNumber(7)
  set heldForSeconds($core.int value) => $_setUnsignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasHeldForSeconds() => $_has(6);
  @$pb.TagNumber(7)
  void clearHeldForSeconds() => $_clearField(7);

  /// Server time, so a client can tell a stalled stream from a quiet one.
  @$pb.TagNumber(8)
  $1.Timestamp get now => $_getN(7);
  @$pb.TagNumber(8)
  set now($1.Timestamp value) => $_setField(8, value);
  @$pb.TagNumber(8)
  $core.bool hasNow() => $_has(7);
  @$pb.TagNumber(8)
  void clearNow() => $_clearField(8);
  @$pb.TagNumber(8)
  $1.Timestamp ensureNow() => $_ensure(7);

  /// Seconds of the window that still hold failures. Nothing is injected any
  /// more and the objective is still broken because the failures have not aged
  /// out yet: this is how long until they have.
  @$pb.TagNumber(9)
  $core.int get healingSeconds => $_getIZ(8);
  @$pb.TagNumber(9)
  set healingSeconds($core.int value) => $_setUnsignedInt32(8, value);
  @$pb.TagNumber(9)
  $core.bool hasHealingSeconds() => $_has(8);
  @$pb.TagNumber(9)
  void clearHealingSeconds() => $_clearField(9);
}

class Instance extends $pb.GeneratedMessage {
  factory Instance({
    $core.String? name,
    $core.String? kind,
    $core.bool? running,
    $core.String? ailment,
    $fixnum.Int64? requestsTotal,
    $fixnum.Int64? requestsFailed,
    $core.bool? inRotation,
    $core.bool? ejected,
    $core.bool? balanced,
  }) {
    final result = Instance._();
    if (name != null) result.name = name;
    if (kind != null) result.kind = kind;
    if (running != null) result.running = running;
    if (ailment != null) result.ailment = ailment;
    if (requestsTotal != null) result.requestsTotal = requestsTotal;
    if (requestsFailed != null) result.requestsFailed = requestsFailed;
    if (inRotation != null) result.inRotation = inRotation;
    if (ejected != null) result.ejected = ejected;
    if (balanced != null) result.balanced = balanced;
    return result;
  }

  Instance._();

  factory Instance.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Instance()..mergeFromBuffer(data, registry);
  factory Instance.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Instance()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Instance',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: Instance.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'kind')
    ..aOB(3, _omitFieldNames ? '' : 'running')
    ..aOS(4, _omitFieldNames ? '' : 'ailment')
    ..a<$fixnum.Int64>(
        5, _omitFieldNames ? '' : 'requestsTotal', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..a<$fixnum.Int64>(
        6, _omitFieldNames ? '' : 'requestsFailed', $pb.PbFieldType.OU6,
        defaultOrMaker: $fixnum.Int64.ZERO)
    ..aOB(7, _omitFieldNames ? '' : 'inRotation')
    ..aOB(8, _omitFieldNames ? '' : 'ejected')
    ..aOB(9, _omitFieldNames ? '' : 'balanced')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Instance clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Instance copyWith(void Function(Instance) updates) =>
      super.copyWith((message) => updates(message as Instance)) as Instance;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Instance() / Instance.new instead')
  static Instance create() => Instance._();
  static $pb.GeneratedMessage $_createMessage() => Instance._();
  @$core.override
  Instance createEmptyInstance() => Instance._();
  @$core.pragma('dart2js:noInline')
  static Instance getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Instance>(Instance.$_createMessage);
  static Instance? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  /// The chaos kind: engine, ledger, and so on.
  @$pb.TagNumber(2)
  $core.String get kind => $_getSZ(1);
  @$pb.TagNumber(2)
  set kind($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearKind() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get running => $_getBF(2);
  @$pb.TagNumber(3)
  set running($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRunning() => $_has(2);
  @$pb.TagNumber(3)
  void clearRunning() => $_clearField(3);

  /// What is wrong with it in words, or empty when nothing is.
  @$pb.TagNumber(4)
  $core.String get ailment => $_getSZ(3);
  @$pb.TagNumber(4)
  set ailment($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasAilment() => $_has(3);
  @$pb.TagNumber(4)
  void clearAilment() => $_clearField(4);

  @$pb.TagNumber(5)
  $fixnum.Int64 get requestsTotal => $_getI64(4);
  @$pb.TagNumber(5)
  set requestsTotal($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasRequestsTotal() => $_has(4);
  @$pb.TagNumber(5)
  void clearRequestsTotal() => $_clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get requestsFailed => $_getI64(5);
  @$pb.TagNumber(6)
  set requestsFailed($fixnum.Int64 value) => $_setInt64(5, value);
  @$pb.TagNumber(6)
  $core.bool hasRequestsFailed() => $_has(5);
  @$pb.TagNumber(6)
  void clearRequestsFailed() => $_clearField(6);

  /// Whether the balancer in front of this instance's kind is currently sending
  /// it traffic. False for a kind that is not behind one.
  @$pb.TagNumber(7)
  $core.bool get inRotation => $_getBF(6);
  @$pb.TagNumber(7)
  set inRotation($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasInRotation() => $_has(6);
  @$pb.TagNumber(7)
  void clearInRotation() => $_clearField(7);

  /// Whether the balancer took it out for failing too much. An ejected instance
  /// is still up and still passing its health check — that is the point of it.
  @$pb.TagNumber(8)
  $core.bool get ejected => $_getBF(7);
  @$pb.TagNumber(8)
  set ejected($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasEjected() => $_has(7);
  @$pb.TagNumber(8)
  void clearEjected() => $_clearField(8);

  /// Whether anything sits in front of this kind at all, so a page can tell
  /// "not balanced" apart from "out of rotation".
  @$pb.TagNumber(9)
  $core.bool get balanced => $_getBF(8);
  @$pb.TagNumber(9)
  set balanced($core.bool value) => $_setBool(8, value);
  @$pb.TagNumber(9)
  $core.bool hasBalanced() => $_has(8);
  @$pb.TagNumber(9)
  void clearBalanced() => $_clearField(9);
}

class Objective extends $pb.GeneratedMessage {
  factory Objective({
    $core.double? target,
    $core.double? current,
    $core.int? windowSeconds,
    $core.double? latencyTargetMs,
    $core.double? latencyCurrentMs,
  }) {
    final result = Objective._();
    if (target != null) result.target = target;
    if (current != null) result.current = current;
    if (windowSeconds != null) result.windowSeconds = windowSeconds;
    if (latencyTargetMs != null) result.latencyTargetMs = latencyTargetMs;
    if (latencyCurrentMs != null) result.latencyCurrentMs = latencyCurrentMs;
    return result;
  }

  Objective._();

  factory Objective.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Objective()..mergeFromBuffer(data, registry);
  factory Objective.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Objective()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Objective',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: Objective.$_createMessage)
    ..aD(1, _omitFieldNames ? '' : 'target')
    ..aD(2, _omitFieldNames ? '' : 'current')
    ..aI(3, _omitFieldNames ? '' : 'windowSeconds',
        fieldType: $pb.PbFieldType.OU3)
    ..aD(4, _omitFieldNames ? '' : 'latencyTargetMs')
    ..aD(5, _omitFieldNames ? '' : 'latencyCurrentMs')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Objective clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Objective copyWith(void Function(Objective) updates) =>
      super.copyWith((message) => updates(message as Objective)) as Objective;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Objective() / Objective.new instead')
  static Objective create() => Objective._();
  static $pb.GeneratedMessage $_createMessage() => Objective._();
  @$core.override
  Objective createEmptyInstance() => Objective._();
  @$core.pragma('dart2js:noInline')
  static Objective getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Objective>(Objective.$_createMessage);
  static Objective? _defaultInstance;

  /// The success rate the sandbox is meant to hold, 0 to 1.
  @$pb.TagNumber(1)
  $core.double get target => $_getN(0);
  @$pb.TagNumber(1)
  set target($core.double value) => $_setDouble(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTarget() => $_has(0);
  @$pb.TagNumber(1)
  void clearTarget() => $_clearField(1);

  /// What it is actually holding over the window.
  @$pb.TagNumber(2)
  $core.double get current => $_getN(1);
  @$pb.TagNumber(2)
  set current($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCurrent() => $_has(1);
  @$pb.TagNumber(2)
  void clearCurrent() => $_clearField(2);

  /// How many seconds the rolling window covers.
  @$pb.TagNumber(3)
  $core.int get windowSeconds => $_getIZ(2);
  @$pb.TagNumber(3)
  set windowSeconds($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasWindowSeconds() => $_has(2);
  @$pb.TagNumber(3)
  void clearWindowSeconds() => $_clearField(3);

  /// The p99 the sandbox is meant to stay under, in milliseconds. Breaking
  /// either clause breaks the objective.
  @$pb.TagNumber(4)
  $core.double get latencyTargetMs => $_getN(3);
  @$pb.TagNumber(4)
  set latencyTargetMs($core.double value) => $_setDouble(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLatencyTargetMs() => $_has(3);
  @$pb.TagNumber(4)
  void clearLatencyTargetMs() => $_clearField(4);

  /// The mean of the per-second p99s across the window. A mean, not a maximum:
  /// one spiky second should not count as a breach.
  @$pb.TagNumber(5)
  $core.double get latencyCurrentMs => $_getN(4);
  @$pb.TagNumber(5)
  set latencyCurrentMs($core.double value) => $_setDouble(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLatencyCurrentMs() => $_has(4);
  @$pb.TagNumber(5)
  void clearLatencyCurrentMs() => $_clearField(5);
}

class Budget extends $pb.GeneratedMessage {
  factory Budget({
    $core.int? tokens,
    $core.int? maxTokens,
    $core.int? refillInSeconds,
  }) {
    final result = Budget._();
    if (tokens != null) result.tokens = tokens;
    if (maxTokens != null) result.maxTokens = maxTokens;
    if (refillInSeconds != null) result.refillInSeconds = refillInSeconds;
    return result;
  }

  Budget._();

  factory Budget.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Budget()..mergeFromBuffer(data, registry);
  factory Budget.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Budget()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Budget',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: Budget.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'tokens', fieldType: $pb.PbFieldType.OU3)
    ..aI(2, _omitFieldNames ? '' : 'maxTokens', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'refillInSeconds',
        fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Budget clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Budget copyWith(void Function(Budget) updates) =>
      super.copyWith((message) => updates(message as Budget)) as Budget;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Budget() / Budget.new instead')
  static Budget create() => Budget._();
  static $pb.GeneratedMessage $_createMessage() => Budget._();
  @$core.override
  Budget createEmptyInstance() => Budget._();
  @$core.pragma('dart2js:noInline')
  static Budget getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Budget>(Budget.$_createMessage);
  static Budget? _defaultInstance;

  /// What is left to spend.
  @$pb.TagNumber(1)
  $core.int get tokens => $_getIZ(0);
  @$pb.TagNumber(1)
  set tokens($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTokens() => $_has(0);
  @$pb.TagNumber(1)
  void clearTokens() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get maxTokens => $_getIZ(1);
  @$pb.TagNumber(2)
  set maxTokens($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMaxTokens() => $_has(1);
  @$pb.TagNumber(2)
  void clearMaxTokens() => $_clearField(2);

  /// Seconds until the next token arrives.
  @$pb.TagNumber(3)
  $core.int get refillInSeconds => $_getIZ(2);
  @$pb.TagNumber(3)
  set refillInSeconds($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasRefillInSeconds() => $_has(2);
  @$pb.TagNumber(3)
  void clearRefillInSeconds() => $_clearField(3);
}

class Traffic extends $pb.GeneratedMessage {
  factory Traffic({
    $core.double? requestsPerSecond,
    $core.double? errorRate,
    $core.double? p50Ms,
    $core.double? p99Ms,
  }) {
    final result = Traffic._();
    if (requestsPerSecond != null) result.requestsPerSecond = requestsPerSecond;
    if (errorRate != null) result.errorRate = errorRate;
    if (p50Ms != null) result.p50Ms = p50Ms;
    if (p99Ms != null) result.p99Ms = p99Ms;
    return result;
  }

  Traffic._();

  factory Traffic.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Traffic()..mergeFromBuffer(data, registry);
  factory Traffic.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Traffic()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Traffic',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: Traffic.$_createMessage)
    ..aD(1, _omitFieldNames ? '' : 'requestsPerSecond')
    ..aD(2, _omitFieldNames ? '' : 'errorRate')
    ..aD(3, _omitFieldNames ? '' : 'p50Ms')
    ..aD(4, _omitFieldNames ? '' : 'p99Ms')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Traffic clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Traffic copyWith(void Function(Traffic) updates) =>
      super.copyWith((message) => updates(message as Traffic)) as Traffic;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Traffic() / Traffic.new instead')
  static Traffic create() => Traffic._();
  static $pb.GeneratedMessage $_createMessage() => Traffic._();
  @$core.override
  Traffic createEmptyInstance() => Traffic._();
  @$core.pragma('dart2js:noInline')
  static Traffic getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Traffic>(Traffic.$_createMessage);
  static Traffic? _defaultInstance;

  @$pb.TagNumber(1)
  $core.double get requestsPerSecond => $_getN(0);
  @$pb.TagNumber(1)
  set requestsPerSecond($core.double value) => $_setDouble(0, value);
  @$pb.TagNumber(1)
  $core.bool hasRequestsPerSecond() => $_has(0);
  @$pb.TagNumber(1)
  void clearRequestsPerSecond() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.double get errorRate => $_getN(1);
  @$pb.TagNumber(2)
  set errorRate($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasErrorRate() => $_has(1);
  @$pb.TagNumber(2)
  void clearErrorRate() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.double get p50Ms => $_getN(2);
  @$pb.TagNumber(3)
  set p50Ms($core.double value) => $_setDouble(2, value);
  @$pb.TagNumber(3)
  $core.bool hasP50Ms() => $_has(2);
  @$pb.TagNumber(3)
  void clearP50Ms() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.double get p99Ms => $_getN(3);
  @$pb.TagNumber(4)
  set p99Ms($core.double value) => $_setDouble(3, value);
  @$pb.TagNumber(4)
  $core.bool hasP99Ms() => $_has(3);
  @$pb.TagNumber(4)
  void clearP99Ms() => $_clearField(4);
}

class ActiveFault extends $pb.GeneratedMessage {
  factory ActiveFault({
    $core.String? id,
    Move? move,
    $core.String? instance,
    $core.String? description,
    $core.int? expiresInSeconds,
    $core.String? actor,
  }) {
    final result = ActiveFault._();
    if (id != null) result.id = id;
    if (move != null) result.move = move;
    if (instance != null) result.instance = instance;
    if (description != null) result.description = description;
    if (expiresInSeconds != null) result.expiresInSeconds = expiresInSeconds;
    if (actor != null) result.actor = actor;
    return result;
  }

  ActiveFault._();

  factory ActiveFault.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ActiveFault()..mergeFromBuffer(data, registry);
  factory ActiveFault.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ActiveFault()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ActiveFault',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: ActiveFault.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aE<Move>(2, _omitFieldNames ? '' : 'move', enumValues: Move.values)
    ..aOS(3, _omitFieldNames ? '' : 'instance')
    ..aOS(4, _omitFieldNames ? '' : 'description')
    ..aI(5, _omitFieldNames ? '' : 'expiresInSeconds',
        fieldType: $pb.PbFieldType.OU3)
    ..aOS(6, _omitFieldNames ? '' : 'actor')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ActiveFault clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ActiveFault copyWith(void Function(ActiveFault) updates) =>
      super.copyWith((message) => updates(message as ActiveFault))
          as ActiveFault;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ActiveFault() / ActiveFault.new instead')
  static ActiveFault create() => ActiveFault._();
  static $pb.GeneratedMessage $_createMessage() => ActiveFault._();
  @$core.override
  ActiveFault createEmptyInstance() => ActiveFault._();
  @$core.pragma('dart2js:noInline')
  static ActiveFault getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ActiveFault>(
          ActiveFault.$_createMessage);
  static ActiveFault? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  Move get move => $_getN(1);
  @$pb.TagNumber(2)
  set move(Move value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasMove() => $_has(1);
  @$pb.TagNumber(2)
  void clearMove() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get instance => $_getSZ(2);
  @$pb.TagNumber(3)
  set instance($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasInstance() => $_has(2);
  @$pb.TagNumber(3)
  void clearInstance() => $_clearField(3);

  /// What it does, in words, for a client that does not know the enum.
  @$pb.TagNumber(4)
  $core.String get description => $_getSZ(3);
  @$pb.TagNumber(4)
  set description($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDescription() => $_has(3);
  @$pb.TagNumber(4)
  void clearDescription() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get expiresInSeconds => $_getIZ(4);
  @$pb.TagNumber(5)
  set expiresInSeconds($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasExpiresInSeconds() => $_has(4);
  @$pb.TagNumber(5)
  void clearExpiresInSeconds() => $_clearField(5);

  /// Who injected it, when they said.
  @$pb.TagNumber(6)
  $core.String get actor => $_getSZ(5);
  @$pb.TagNumber(6)
  set actor($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasActor() => $_has(5);
  @$pb.TagNumber(6)
  void clearActor() => $_clearField(6);
}

class InjectFaultRequest extends $pb.GeneratedMessage {
  factory InjectFaultRequest({
    Move? move,
    $core.String? instance,
    $core.String? actor,
  }) {
    final result = InjectFaultRequest._();
    if (move != null) result.move = move;
    if (instance != null) result.instance = instance;
    if (actor != null) result.actor = actor;
    return result;
  }

  InjectFaultRequest._();

  factory InjectFaultRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InjectFaultRequest()..mergeFromBuffer(data, registry);
  factory InjectFaultRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InjectFaultRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InjectFaultRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: InjectFaultRequest.$_createMessage)
    ..aE<Move>(1, _omitFieldNames ? '' : 'move', enumValues: Move.values)
    ..aOS(2, _omitFieldNames ? '' : 'instance')
    ..aOS(3, _omitFieldNames ? '' : 'actor')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InjectFaultRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InjectFaultRequest copyWith(void Function(InjectFaultRequest) updates) =>
      super.copyWith((message) => updates(message as InjectFaultRequest))
          as InjectFaultRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use InjectFaultRequest() / InjectFaultRequest.new instead')
  static InjectFaultRequest create() => InjectFaultRequest._();
  static $pb.GeneratedMessage $_createMessage() => InjectFaultRequest._();
  @$core.override
  InjectFaultRequest createEmptyInstance() => InjectFaultRequest._();
  @$core.pragma('dart2js:noInline')
  static InjectFaultRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InjectFaultRequest>(
          InjectFaultRequest.$_createMessage);
  static InjectFaultRequest? _defaultInstance;

  @$pb.TagNumber(1)
  Move get move => $_getN(0);
  @$pb.TagNumber(1)
  set move(Move value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMove() => $_has(0);
  @$pb.TagNumber(1)
  void clearMove() => $_clearField(1);

  /// Which instance to hit. Empty lets the service choose one that suits the
  /// move; a name it does not recognise is refused rather than guessed at.
  @$pb.TagNumber(2)
  $core.String get instance => $_getSZ(1);
  @$pb.TagNumber(2)
  set instance($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasInstance() => $_has(1);
  @$pb.TagNumber(2)
  void clearInstance() => $_clearField(2);

  /// An optional name for the leaderboard. Trimmed, capped and sanitised.
  @$pb.TagNumber(3)
  $core.String get actor => $_getSZ(2);
  @$pb.TagNumber(3)
  set actor($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasActor() => $_has(2);
  @$pb.TagNumber(3)
  void clearActor() => $_clearField(3);
}

class InjectFaultResponse extends $pb.GeneratedMessage {
  factory InjectFaultResponse({
    $core.bool? accepted,
    $core.String? reason,
    $core.int? cost,
    World? world,
  }) {
    final result = InjectFaultResponse._();
    if (accepted != null) result.accepted = accepted;
    if (reason != null) result.reason = reason;
    if (cost != null) result.cost = cost;
    if (world != null) result.world = world;
    return result;
  }

  InjectFaultResponse._();

  factory InjectFaultResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InjectFaultResponse()..mergeFromBuffer(data, registry);
  factory InjectFaultResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InjectFaultResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InjectFaultResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: InjectFaultResponse.$_createMessage)
    ..aOB(1, _omitFieldNames ? '' : 'accepted')
    ..aOS(2, _omitFieldNames ? '' : 'reason')
    ..aI(3, _omitFieldNames ? '' : 'cost', fieldType: $pb.PbFieldType.OU3)
    ..aOM<World>(4, _omitFieldNames ? '' : 'world',
        subBuilder: World.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InjectFaultResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InjectFaultResponse copyWith(void Function(InjectFaultResponse) updates) =>
      super.copyWith((message) => updates(message as InjectFaultResponse))
          as InjectFaultResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use InjectFaultResponse() / InjectFaultResponse.new instead')
  static InjectFaultResponse create() => InjectFaultResponse._();
  static $pb.GeneratedMessage $_createMessage() => InjectFaultResponse._();
  @$core.override
  InjectFaultResponse createEmptyInstance() => InjectFaultResponse._();
  @$core.pragma('dart2js:noInline')
  static InjectFaultResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<InjectFaultResponse>(
          InjectFaultResponse.$_createMessage);
  static InjectFaultResponse? _defaultInstance;

  /// False when the move was refused; `reason` says why and the world is
  /// returned unchanged.
  @$pb.TagNumber(1)
  $core.bool get accepted => $_getBF(0);
  @$pb.TagNumber(1)
  set accepted($core.bool value) => $_setBool(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAccepted() => $_has(0);
  @$pb.TagNumber(1)
  void clearAccepted() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get reason => $_getSZ(1);
  @$pb.TagNumber(2)
  set reason($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasReason() => $_has(1);
  @$pb.TagNumber(2)
  void clearReason() => $_clearField(2);

  /// What the move cost, in budget tokens.
  @$pb.TagNumber(3)
  $core.int get cost => $_getIZ(2);
  @$pb.TagNumber(3)
  set cost($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCost() => $_has(2);
  @$pb.TagNumber(3)
  void clearCost() => $_clearField(3);

  @$pb.TagNumber(4)
  World get world => $_getN(3);
  @$pb.TagNumber(4)
  set world(World value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasWorld() => $_has(3);
  @$pb.TagNumber(4)
  void clearWorld() => $_clearField(4);
  @$pb.TagNumber(4)
  World ensureWorld() => $_ensure(3);
}

class ScoresRequest extends $pb.GeneratedMessage {
  factory ScoresRequest() => ScoresRequest._();

  ScoresRequest._();

  factory ScoresRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ScoresRequest()..mergeFromBuffer(data, registry);
  factory ScoresRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ScoresRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ScoresRequest',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: ScoresRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScoresRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScoresRequest copyWith(void Function(ScoresRequest) updates) =>
      super.copyWith((message) => updates(message as ScoresRequest))
          as ScoresRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ScoresRequest() / ScoresRequest.new instead')
  static ScoresRequest create() => ScoresRequest._();
  static $pb.GeneratedMessage $_createMessage() => ScoresRequest._();
  @$core.override
  ScoresRequest createEmptyInstance() => ScoresRequest._();
  @$core.pragma('dart2js:noInline')
  static ScoresRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ScoresRequest>(
          ScoresRequest.$_createMessage);
  static ScoresRequest? _defaultInstance;
}

class ScoresResponse extends $pb.GeneratedMessage {
  factory ScoresResponse({
    $core.Iterable<Score>? scores,
  }) {
    final result = ScoresResponse._();
    if (scores != null) result.scores.addAll(scores);
    return result;
  }

  ScoresResponse._();

  factory ScoresResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ScoresResponse()..mergeFromBuffer(data, registry);
  factory ScoresResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ScoresResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ScoresResponse',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: ScoresResponse.$_createMessage)
    ..pPM<Score>(1, _omitFieldNames ? '' : 'scores',
        subBuilder: Score.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScoresResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ScoresResponse copyWith(void Function(ScoresResponse) updates) =>
      super.copyWith((message) => updates(message as ScoresResponse))
          as ScoresResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ScoresResponse() / ScoresResponse.new instead')
  static ScoresResponse create() => ScoresResponse._();
  static $pb.GeneratedMessage $_createMessage() => ScoresResponse._();
  @$core.override
  ScoresResponse createEmptyInstance() => ScoresResponse._();
  @$core.pragma('dart2js:noInline')
  static ScoresResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ScoresResponse>(
          ScoresResponse.$_createMessage);
  static ScoresResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Score> get scores => $_getList(0);
}

class Score extends $pb.GeneratedMessage {
  factory Score({
    $core.String? actor,
    $core.double? secondsToBreach,
    $core.int? faultsUsed,
    $1.Timestamp? at,
  }) {
    final result = Score._();
    if (actor != null) result.actor = actor;
    if (secondsToBreach != null) result.secondsToBreach = secondsToBreach;
    if (faultsUsed != null) result.faultsUsed = faultsUsed;
    if (at != null) result.at = at;
    return result;
  }

  Score._();

  factory Score.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Score()..mergeFromBuffer(data, registry);
  factory Score.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Score()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Score',
      package:
          const $pb.PackageName(_omitMessageNames ? '' : 'tbd.playground.v1'),
      createEmptyInstance: Score.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'actor')
    ..aD(2, _omitFieldNames ? '' : 'secondsToBreach')
    ..aI(3, _omitFieldNames ? '' : 'faultsUsed', fieldType: $pb.PbFieldType.OU3)
    ..aOM<$1.Timestamp>(4, _omitFieldNames ? '' : 'at',
        subBuilder: $1.Timestamp.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Score clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Score copyWith(void Function(Score) updates) =>
      super.copyWith((message) => updates(message as Score)) as Score;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Score() / Score.new instead')
  static Score create() => Score._();
  static $pb.GeneratedMessage $_createMessage() => Score._();
  @$core.override
  Score createEmptyInstance() => Score._();
  @$core.pragma('dart2js:noInline')
  static Score getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Score>(Score.$_createMessage);
  static Score? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get actor => $_getSZ(0);
  @$pb.TagNumber(1)
  set actor($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasActor() => $_has(0);
  @$pb.TagNumber(1)
  void clearActor() => $_clearField(1);

  /// How long the breach took from the last healed moment.
  @$pb.TagNumber(2)
  $core.double get secondsToBreach => $_getN(1);
  @$pb.TagNumber(2)
  set secondsToBreach($core.double value) => $_setDouble(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSecondsToBreach() => $_has(1);
  @$pb.TagNumber(2)
  void clearSecondsToBreach() => $_clearField(2);

  /// How many faults were live when it broke.
  @$pb.TagNumber(3)
  $core.int get faultsUsed => $_getIZ(2);
  @$pb.TagNumber(3)
  set faultsUsed($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFaultsUsed() => $_has(2);
  @$pb.TagNumber(3)
  void clearFaultsUsed() => $_clearField(3);

  @$pb.TagNumber(4)
  $1.Timestamp get at => $_getN(3);
  @$pb.TagNumber(4)
  set at($1.Timestamp value) => $_setField(4, value);
  @$pb.TagNumber(4)
  $core.bool hasAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearAt() => $_clearField(4);
  @$pb.TagNumber(4)
  $1.Timestamp ensureAt() => $_ensure(3);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
