// This is a generated file - do not edit.
//
// Generated from tbd/finance/v1/finance.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:core' as $core;

import 'package:fixnum/fixnum.dart' as $fixnum;
import 'package:protobuf/protobuf.dart' as $pb;

export 'package:protobuf/protobuf.dart' show GeneratedMessageGenericExtensions;

class ConnectorKind extends $pb.GeneratedMessage {
  factory ConnectorKind({
    $core.String? name,
    $core.String? label,
    $core.String? description,
    $core.String? auth,
    $core.String? consentNote,
    $core.bool? configured,
  }) {
    final result = ConnectorKind._();
    if (name != null) result.name = name;
    if (label != null) result.label = label;
    if (description != null) result.description = description;
    if (auth != null) result.auth = auth;
    if (consentNote != null) result.consentNote = consentNote;
    if (configured != null) result.configured = configured;
    return result;
  }

  ConnectorKind._();

  factory ConnectorKind.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConnectorKind()..mergeFromBuffer(data, registry);
  factory ConnectorKind.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConnectorKind()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConnectorKind',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ConnectorKind.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..aOS(2, _omitFieldNames ? '' : 'label')
    ..aOS(3, _omitFieldNames ? '' : 'description')
    ..aOS(4, _omitFieldNames ? '' : 'auth')
    ..aOS(5, _omitFieldNames ? '' : 'consentNote')
    ..aOB(6, _omitFieldNames ? '' : 'configured')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectorKind clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectorKind copyWith(void Function(ConnectorKind) updates) =>
      super.copyWith((message) => updates(message as ConnectorKind))
          as ConnectorKind;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ConnectorKind() / ConnectorKind.new instead')
  static ConnectorKind create() => ConnectorKind._();
  static $pb.GeneratedMessage $_createMessage() => ConnectorKind._();
  @$core.override
  ConnectorKind createEmptyInstance() => ConnectorKind._();
  @$core.pragma('dart2js:noInline')
  static ConnectorKind getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ConnectorKind>(
          ConnectorKind.$_createMessage);
  static ConnectorKind? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get label => $_getSZ(1);
  @$pb.TagNumber(2)
  set label($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLabel() => $_has(1);
  @$pb.TagNumber(2)
  void clearLabel() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get description => $_getSZ(2);
  @$pb.TagNumber(3)
  set description($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDescription() => $_has(2);
  @$pb.TagNumber(3)
  void clearDescription() => $_clearField(3);

  /// oauth or token.
  @$pb.TagNumber(4)
  $core.String get auth => $_getSZ(3);
  @$pb.TagNumber(4)
  set auth($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasAuth() => $_has(3);
  @$pb.TagNumber(4)
  void clearAuth() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get consentNote => $_getSZ(4);
  @$pb.TagNumber(5)
  set consentNote($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasConsentNote() => $_has(4);
  @$pb.TagNumber(5)
  void clearConsentNote() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get configured => $_getBF(5);
  @$pb.TagNumber(6)
  set configured($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasConfigured() => $_has(5);
  @$pb.TagNumber(6)
  void clearConfigured() => $_clearField(6);
}

class ListConnectorKindsRequest extends $pb.GeneratedMessage {
  factory ListConnectorKindsRequest() => ListConnectorKindsRequest._();

  ListConnectorKindsRequest._();

  factory ListConnectorKindsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorKindsRequest()..mergeFromBuffer(data, registry);
  factory ListConnectorKindsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorKindsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConnectorKindsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListConnectorKindsRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorKindsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorKindsRequest copyWith(
          void Function(ListConnectorKindsRequest) updates) =>
      super.copyWith((message) => updates(message as ListConnectorKindsRequest))
          as ListConnectorKindsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListConnectorKindsRequest() / ListConnectorKindsRequest.new instead')
  static ListConnectorKindsRequest create() => ListConnectorKindsRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      ListConnectorKindsRequest._();
  @$core.override
  ListConnectorKindsRequest createEmptyInstance() =>
      ListConnectorKindsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListConnectorKindsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConnectorKindsRequest>(
          ListConnectorKindsRequest.$_createMessage);
  static ListConnectorKindsRequest? _defaultInstance;
}

class ListConnectorKindsResponse extends $pb.GeneratedMessage {
  factory ListConnectorKindsResponse({
    $core.Iterable<ConnectorKind>? kinds,
  }) {
    final result = ListConnectorKindsResponse._();
    if (kinds != null) result.kinds.addAll(kinds);
    return result;
  }

  ListConnectorKindsResponse._();

  factory ListConnectorKindsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorKindsResponse()..mergeFromBuffer(data, registry);
  factory ListConnectorKindsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorKindsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConnectorKindsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListConnectorKindsResponse.$_createMessage)
    ..pPM<ConnectorKind>(1, _omitFieldNames ? '' : 'kinds',
        subBuilder: ConnectorKind.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorKindsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorKindsResponse copyWith(
          void Function(ListConnectorKindsResponse) updates) =>
      super.copyWith(
              (message) => updates(message as ListConnectorKindsResponse))
          as ListConnectorKindsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListConnectorKindsResponse() / ListConnectorKindsResponse.new instead')
  static ListConnectorKindsResponse create() => ListConnectorKindsResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      ListConnectorKindsResponse._();
  @$core.override
  ListConnectorKindsResponse createEmptyInstance() =>
      ListConnectorKindsResponse._();
  @$core.pragma('dart2js:noInline')
  static ListConnectorKindsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConnectorKindsResponse>(
          ListConnectorKindsResponse.$_createMessage);
  static ListConnectorKindsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ConnectorKind> get kinds => $_getList(0);
}

class Connector extends $pb.GeneratedMessage {
  factory Connector({
    $core.String? id,
    $core.String? partyId,
    $core.String? kind,
    $core.String? label,
    $core.String? status,
    $core.String? config,
    $core.String? externalId,
    $core.String? linkedAt,
    $core.String? lastSyncAt,
    $core.String? lastSyncStatus,
    $core.String? lastSyncError,
    $core.String? failure,
    $core.String? createdAt,
    $core.bool? canSend,
  }) {
    final result = Connector._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (kind != null) result.kind = kind;
    if (label != null) result.label = label;
    if (status != null) result.status = status;
    if (config != null) result.config = config;
    if (externalId != null) result.externalId = externalId;
    if (linkedAt != null) result.linkedAt = linkedAt;
    if (lastSyncAt != null) result.lastSyncAt = lastSyncAt;
    if (lastSyncStatus != null) result.lastSyncStatus = lastSyncStatus;
    if (lastSyncError != null) result.lastSyncError = lastSyncError;
    if (failure != null) result.failure = failure;
    if (createdAt != null) result.createdAt = createdAt;
    if (canSend != null) result.canSend = canSend;
    return result;
  }

  Connector._();

  factory Connector.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Connector()..mergeFromBuffer(data, registry);
  factory Connector.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Connector()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Connector',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Connector.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'kind')
    ..aOS(4, _omitFieldNames ? '' : 'label')
    ..aOS(5, _omitFieldNames ? '' : 'status')
    ..aOS(6, _omitFieldNames ? '' : 'config')
    ..aOS(7, _omitFieldNames ? '' : 'externalId')
    ..aOS(8, _omitFieldNames ? '' : 'linkedAt')
    ..aOS(9, _omitFieldNames ? '' : 'lastSyncAt')
    ..aOS(10, _omitFieldNames ? '' : 'lastSyncStatus')
    ..aOS(11, _omitFieldNames ? '' : 'lastSyncError')
    ..aOS(12, _omitFieldNames ? '' : 'failure')
    ..aOS(13, _omitFieldNames ? '' : 'createdAt')
    ..aOB(14, _omitFieldNames ? '' : 'canSend')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Connector clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Connector copyWith(void Function(Connector) updates) =>
      super.copyWith((message) => updates(message as Connector)) as Connector;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Connector() / Connector.new instead')
  static Connector create() => Connector._();
  static $pb.GeneratedMessage $_createMessage() => Connector._();
  @$core.override
  Connector createEmptyInstance() => Connector._();
  @$core.pragma('dart2js:noInline')
  static Connector getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Connector>(Connector.$_createMessage);
  static Connector? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get kind => $_getSZ(2);
  @$pb.TagNumber(3)
  set kind($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasKind() => $_has(2);
  @$pb.TagNumber(3)
  void clearKind() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get label => $_getSZ(3);
  @$pb.TagNumber(4)
  set label($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLabel() => $_has(3);
  @$pb.TagNumber(4)
  void clearLabel() => $_clearField(4);

  /// pending, linked, expired, failed, disabled.
  @$pb.TagNumber(5)
  $core.String get status => $_getSZ(4);
  @$pb.TagNumber(5)
  set status($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasStatus() => $_has(4);
  @$pb.TagNumber(5)
  void clearStatus() => $_clearField(5);

  /// The kind's non-secret settings, as JSON text.
  @$pb.TagNumber(6)
  $core.String get config => $_getSZ(5);
  @$pb.TagNumber(6)
  set config($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasConfig() => $_has(5);
  @$pb.TagNumber(6)
  void clearConfig() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get externalId => $_getSZ(6);
  @$pb.TagNumber(7)
  set externalId($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasExternalId() => $_has(6);
  @$pb.TagNumber(7)
  void clearExternalId() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get linkedAt => $_getSZ(7);
  @$pb.TagNumber(8)
  set linkedAt($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasLinkedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearLinkedAt() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get lastSyncAt => $_getSZ(8);
  @$pb.TagNumber(9)
  set lastSyncAt($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasLastSyncAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearLastSyncAt() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get lastSyncStatus => $_getSZ(9);
  @$pb.TagNumber(10)
  set lastSyncStatus($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasLastSyncStatus() => $_has(9);
  @$pb.TagNumber(10)
  void clearLastSyncStatus() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.String get lastSyncError => $_getSZ(10);
  @$pb.TagNumber(11)
  set lastSyncError($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasLastSyncError() => $_has(10);
  @$pb.TagNumber(11)
  void clearLastSyncError() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get failure => $_getSZ(11);
  @$pb.TagNumber(12)
  set failure($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasFailure() => $_has(11);
  @$pb.TagNumber(12)
  void clearFailure() => $_clearField(12);

  @$pb.TagNumber(13)
  $core.String get createdAt => $_getSZ(12);
  @$pb.TagNumber(13)
  set createdAt($core.String value) => $_setString(12, value);
  @$pb.TagNumber(13)
  $core.bool hasCreatedAt() => $_has(12);
  @$pb.TagNumber(13)
  void clearCreatedAt() => $_clearField(13);

  /// The consent included sending: the mail page may use it.
  @$pb.TagNumber(14)
  $core.bool get canSend => $_getBF(13);
  @$pb.TagNumber(14)
  set canSend($core.bool value) => $_setBool(13, value);
  @$pb.TagNumber(14)
  $core.bool hasCanSend() => $_has(13);
  @$pb.TagNumber(14)
  void clearCanSend() => $_clearField(14);
}

class ListConnectorsRequest extends $pb.GeneratedMessage {
  factory ListConnectorsRequest({
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = ListConnectorsRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  ListConnectorsRequest._();

  factory ListConnectorsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorsRequest()..mergeFromBuffer(data, registry);
  factory ListConnectorsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConnectorsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListConnectorsRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorsRequest copyWith(
          void Function(ListConnectorsRequest) updates) =>
      super.copyWith((message) => updates(message as ListConnectorsRequest))
          as ListConnectorsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListConnectorsRequest() / ListConnectorsRequest.new instead')
  static ListConnectorsRequest create() => ListConnectorsRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListConnectorsRequest._();
  @$core.override
  ListConnectorsRequest createEmptyInstance() => ListConnectorsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListConnectorsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConnectorsRequest>(
          ListConnectorsRequest.$_createMessage);
  static ListConnectorsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);
}

class ListConnectorsResponse extends $pb.GeneratedMessage {
  factory ListConnectorsResponse({
    $core.Iterable<Connector>? connectors,
  }) {
    final result = ListConnectorsResponse._();
    if (connectors != null) result.connectors.addAll(connectors);
    return result;
  }

  ListConnectorsResponse._();

  factory ListConnectorsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorsResponse()..mergeFromBuffer(data, registry);
  factory ListConnectorsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConnectorsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListConnectorsResponse.$_createMessage)
    ..pPM<Connector>(1, _omitFieldNames ? '' : 'connectors',
        subBuilder: Connector.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorsResponse copyWith(
          void Function(ListConnectorsResponse) updates) =>
      super.copyWith((message) => updates(message as ListConnectorsResponse))
          as ListConnectorsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListConnectorsResponse() / ListConnectorsResponse.new instead')
  static ListConnectorsResponse create() => ListConnectorsResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListConnectorsResponse._();
  @$core.override
  ListConnectorsResponse createEmptyInstance() => ListConnectorsResponse._();
  @$core.pragma('dart2js:noInline')
  static ListConnectorsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConnectorsResponse>(
          ListConnectorsResponse.$_createMessage);
  static ListConnectorsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Connector> get connectors => $_getList(0);
}

class StartConnectorRequest extends $pb.GeneratedMessage {
  factory StartConnectorRequest({
    $core.String? partyId,
    $core.String? kind,
  }) {
    final result = StartConnectorRequest._();
    if (partyId != null) result.partyId = partyId;
    if (kind != null) result.kind = kind;
    return result;
  }

  StartConnectorRequest._();

  factory StartConnectorRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartConnectorRequest()..mergeFromBuffer(data, registry);
  factory StartConnectorRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartConnectorRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StartConnectorRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: StartConnectorRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'partyId')
    ..aOS(2, _omitFieldNames ? '' : 'kind')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartConnectorRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartConnectorRequest copyWith(
          void Function(StartConnectorRequest) updates) =>
      super.copyWith((message) => updates(message as StartConnectorRequest))
          as StartConnectorRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use StartConnectorRequest() / StartConnectorRequest.new instead')
  static StartConnectorRequest create() => StartConnectorRequest._();
  static $pb.GeneratedMessage $_createMessage() => StartConnectorRequest._();
  @$core.override
  StartConnectorRequest createEmptyInstance() => StartConnectorRequest._();
  @$core.pragma('dart2js:noInline')
  static StartConnectorRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StartConnectorRequest>(
          StartConnectorRequest.$_createMessage);
  static StartConnectorRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get partyId => $_getSZ(0);
  @$pb.TagNumber(1)
  set partyId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPartyId() => $_has(0);
  @$pb.TagNumber(1)
  void clearPartyId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get kind => $_getSZ(1);
  @$pb.TagNumber(2)
  set kind($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearKind() => $_clearField(2);
}

class StartConnectorResponse extends $pb.GeneratedMessage {
  factory StartConnectorResponse({
    $core.String? connectorId,
    $core.String? url,
  }) {
    final result = StartConnectorResponse._();
    if (connectorId != null) result.connectorId = connectorId;
    if (url != null) result.url = url;
    return result;
  }

  StartConnectorResponse._();

  factory StartConnectorResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartConnectorResponse()..mergeFromBuffer(data, registry);
  factory StartConnectorResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartConnectorResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StartConnectorResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: StartConnectorResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'connectorId')
    ..aOS(2, _omitFieldNames ? '' : 'url')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartConnectorResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartConnectorResponse copyWith(
          void Function(StartConnectorResponse) updates) =>
      super.copyWith((message) => updates(message as StartConnectorResponse))
          as StartConnectorResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use StartConnectorResponse() / StartConnectorResponse.new instead')
  static StartConnectorResponse create() => StartConnectorResponse._();
  static $pb.GeneratedMessage $_createMessage() => StartConnectorResponse._();
  @$core.override
  StartConnectorResponse createEmptyInstance() => StartConnectorResponse._();
  @$core.pragma('dart2js:noInline')
  static StartConnectorResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StartConnectorResponse>(
          StartConnectorResponse.$_createMessage);
  static StartConnectorResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get connectorId => $_getSZ(0);
  @$pb.TagNumber(1)
  set connectorId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasConnectorId() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnectorId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get url => $_getSZ(1);
  @$pb.TagNumber(2)
  set url($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUrl() => $_has(1);
  @$pb.TagNumber(2)
  void clearUrl() => $_clearField(2);
}

class CompleteConnectorRequest extends $pb.GeneratedMessage {
  factory CompleteConnectorRequest({
    $core.String? state,
    $core.String? code,
  }) {
    final result = CompleteConnectorRequest._();
    if (state != null) result.state = state;
    if (code != null) result.code = code;
    return result;
  }

  CompleteConnectorRequest._();

  factory CompleteConnectorRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteConnectorRequest()..mergeFromBuffer(data, registry);
  factory CompleteConnectorRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteConnectorRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteConnectorRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: CompleteConnectorRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'state')
    ..aOS(2, _omitFieldNames ? '' : 'code')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteConnectorRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteConnectorRequest copyWith(
          void Function(CompleteConnectorRequest) updates) =>
      super.copyWith((message) => updates(message as CompleteConnectorRequest))
          as CompleteConnectorRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use CompleteConnectorRequest() / CompleteConnectorRequest.new instead')
  static CompleteConnectorRequest create() => CompleteConnectorRequest._();
  static $pb.GeneratedMessage $_createMessage() => CompleteConnectorRequest._();
  @$core.override
  CompleteConnectorRequest createEmptyInstance() =>
      CompleteConnectorRequest._();
  @$core.pragma('dart2js:noInline')
  static CompleteConnectorRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteConnectorRequest>(
          CompleteConnectorRequest.$_createMessage);
  static CompleteConnectorRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get state => $_getSZ(0);
  @$pb.TagNumber(1)
  set state($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasState() => $_has(0);
  @$pb.TagNumber(1)
  void clearState() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get code => $_getSZ(1);
  @$pb.TagNumber(2)
  set code($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCode() => $_has(1);
  @$pb.TagNumber(2)
  void clearCode() => $_clearField(2);
}

class CompleteConnectorResponse extends $pb.GeneratedMessage {
  factory CompleteConnectorResponse({
    Connector? connector,
  }) {
    final result = CompleteConnectorResponse._();
    if (connector != null) result.connector = connector;
    return result;
  }

  CompleteConnectorResponse._();

  factory CompleteConnectorResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteConnectorResponse()..mergeFromBuffer(data, registry);
  factory CompleteConnectorResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteConnectorResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteConnectorResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: CompleteConnectorResponse.$_createMessage)
    ..aOM<Connector>(1, _omitFieldNames ? '' : 'connector',
        subBuilder: Connector.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteConnectorResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteConnectorResponse copyWith(
          void Function(CompleteConnectorResponse) updates) =>
      super.copyWith((message) => updates(message as CompleteConnectorResponse))
          as CompleteConnectorResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use CompleteConnectorResponse() / CompleteConnectorResponse.new instead')
  static CompleteConnectorResponse create() => CompleteConnectorResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      CompleteConnectorResponse._();
  @$core.override
  CompleteConnectorResponse createEmptyInstance() =>
      CompleteConnectorResponse._();
  @$core.pragma('dart2js:noInline')
  static CompleteConnectorResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteConnectorResponse>(
          CompleteConnectorResponse.$_createMessage);
  static CompleteConnectorResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Connector get connector => $_getN(0);
  @$pb.TagNumber(1)
  set connector(Connector value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasConnector() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnector() => $_clearField(1);
  @$pb.TagNumber(1)
  Connector ensureConnector() => $_ensure(0);
}

class TestConnectorRequest extends $pb.GeneratedMessage {
  factory TestConnectorRequest({
    $core.String? id,
  }) {
    final result = TestConnectorRequest._();
    if (id != null) result.id = id;
    return result;
  }

  TestConnectorRequest._();

  factory TestConnectorRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TestConnectorRequest()..mergeFromBuffer(data, registry);
  factory TestConnectorRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TestConnectorRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TestConnectorRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: TestConnectorRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TestConnectorRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TestConnectorRequest copyWith(void Function(TestConnectorRequest) updates) =>
      super.copyWith((message) => updates(message as TestConnectorRequest))
          as TestConnectorRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use TestConnectorRequest() / TestConnectorRequest.new instead')
  static TestConnectorRequest create() => TestConnectorRequest._();
  static $pb.GeneratedMessage $_createMessage() => TestConnectorRequest._();
  @$core.override
  TestConnectorRequest createEmptyInstance() => TestConnectorRequest._();
  @$core.pragma('dart2js:noInline')
  static TestConnectorRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TestConnectorRequest>(
          TestConnectorRequest.$_createMessage);
  static TestConnectorRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class TestConnectorResponse extends $pb.GeneratedMessage {
  factory TestConnectorResponse({
    $core.String? status,
  }) {
    final result = TestConnectorResponse._();
    if (status != null) result.status = status;
    return result;
  }

  TestConnectorResponse._();

  factory TestConnectorResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TestConnectorResponse()..mergeFromBuffer(data, registry);
  factory TestConnectorResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      TestConnectorResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'TestConnectorResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: TestConnectorResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'status')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TestConnectorResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  TestConnectorResponse copyWith(
          void Function(TestConnectorResponse) updates) =>
      super.copyWith((message) => updates(message as TestConnectorResponse))
          as TestConnectorResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use TestConnectorResponse() / TestConnectorResponse.new instead')
  static TestConnectorResponse create() => TestConnectorResponse._();
  static $pb.GeneratedMessage $_createMessage() => TestConnectorResponse._();
  @$core.override
  TestConnectorResponse createEmptyInstance() => TestConnectorResponse._();
  @$core.pragma('dart2js:noInline')
  static TestConnectorResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<TestConnectorResponse>(
          TestConnectorResponse.$_createMessage);
  static TestConnectorResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get status => $_getSZ(0);
  @$pb.TagNumber(1)
  set status($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasStatus() => $_has(0);
  @$pb.TagNumber(1)
  void clearStatus() => $_clearField(1);
}

class SyncConnectorRequest extends $pb.GeneratedMessage {
  factory SyncConnectorRequest({
    $core.String? id,
  }) {
    final result = SyncConnectorRequest._();
    if (id != null) result.id = id;
    return result;
  }

  SyncConnectorRequest._();

  factory SyncConnectorRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SyncConnectorRequest()..mergeFromBuffer(data, registry);
  factory SyncConnectorRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SyncConnectorRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SyncConnectorRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SyncConnectorRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncConnectorRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncConnectorRequest copyWith(void Function(SyncConnectorRequest) updates) =>
      super.copyWith((message) => updates(message as SyncConnectorRequest))
          as SyncConnectorRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use SyncConnectorRequest() / SyncConnectorRequest.new instead')
  static SyncConnectorRequest create() => SyncConnectorRequest._();
  static $pb.GeneratedMessage $_createMessage() => SyncConnectorRequest._();
  @$core.override
  SyncConnectorRequest createEmptyInstance() => SyncConnectorRequest._();
  @$core.pragma('dart2js:noInline')
  static SyncConnectorRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SyncConnectorRequest>(
          SyncConnectorRequest.$_createMessage);
  static SyncConnectorRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

/// The run just opened, `outcome` empty while it pulls. A mailbox takes
/// minutes and a call has seconds, so the pull runs detached; poll
/// ListConnectorRuns for the outcome.
class SyncConnectorResponse extends $pb.GeneratedMessage {
  factory SyncConnectorResponse({
    ConnectorRun? run,
  }) {
    final result = SyncConnectorResponse._();
    if (run != null) result.run = run;
    return result;
  }

  SyncConnectorResponse._();

  factory SyncConnectorResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SyncConnectorResponse()..mergeFromBuffer(data, registry);
  factory SyncConnectorResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SyncConnectorResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SyncConnectorResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SyncConnectorResponse.$_createMessage)
    ..aOM<ConnectorRun>(1, _omitFieldNames ? '' : 'run',
        subBuilder: ConnectorRun.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncConnectorResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SyncConnectorResponse copyWith(
          void Function(SyncConnectorResponse) updates) =>
      super.copyWith((message) => updates(message as SyncConnectorResponse))
          as SyncConnectorResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use SyncConnectorResponse() / SyncConnectorResponse.new instead')
  static SyncConnectorResponse create() => SyncConnectorResponse._();
  static $pb.GeneratedMessage $_createMessage() => SyncConnectorResponse._();
  @$core.override
  SyncConnectorResponse createEmptyInstance() => SyncConnectorResponse._();
  @$core.pragma('dart2js:noInline')
  static SyncConnectorResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SyncConnectorResponse>(
          SyncConnectorResponse.$_createMessage);
  static SyncConnectorResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ConnectorRun get run => $_getN(0);
  @$pb.TagNumber(1)
  set run(ConnectorRun value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRun() => $_has(0);
  @$pb.TagNumber(1)
  void clearRun() => $_clearField(1);
  @$pb.TagNumber(1)
  ConnectorRun ensureRun() => $_ensure(0);
}

class ConfigureConnectorRequest extends $pb.GeneratedMessage {
  factory ConfigureConnectorRequest({
    $core.String? id,
    $core.String? config,
    $core.String? partyId,
    $core.String? label,
  }) {
    final result = ConfigureConnectorRequest._();
    if (id != null) result.id = id;
    if (config != null) result.config = config;
    if (partyId != null) result.partyId = partyId;
    if (label != null) result.label = label;
    return result;
  }

  ConfigureConnectorRequest._();

  factory ConfigureConnectorRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConfigureConnectorRequest()..mergeFromBuffer(data, registry);
  factory ConfigureConnectorRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConfigureConnectorRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConfigureConnectorRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ConfigureConnectorRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'config')
    ..aOS(3, _omitFieldNames ? '' : 'partyId')
    ..aOS(4, _omitFieldNames ? '' : 'label')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigureConnectorRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigureConnectorRequest copyWith(
          void Function(ConfigureConnectorRequest) updates) =>
      super.copyWith((message) => updates(message as ConfigureConnectorRequest))
          as ConfigureConnectorRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ConfigureConnectorRequest() / ConfigureConnectorRequest.new instead')
  static ConfigureConnectorRequest create() => ConfigureConnectorRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      ConfigureConnectorRequest._();
  @$core.override
  ConfigureConnectorRequest createEmptyInstance() =>
      ConfigureConnectorRequest._();
  @$core.pragma('dart2js:noInline')
  static ConfigureConnectorRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConfigureConnectorRequest>(
          ConfigureConnectorRequest.$_createMessage);
  static ConfigureConnectorRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// JSON text. Empty: unchanged.
  @$pb.TagNumber(2)
  $core.String get config => $_getSZ(1);
  @$pb.TagNumber(2)
  set config($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasConfig() => $_has(1);
  @$pb.TagNumber(2)
  void clearConfig() => $_clearField(2);

  /// Move the connector, and the documents only it pulled, to another party
  /// in the caller's grant. Empty: unchanged.
  @$pb.TagNumber(3)
  $core.String get partyId => $_getSZ(2);
  @$pb.TagNumber(3)
  set partyId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPartyId() => $_has(2);
  @$pb.TagNumber(3)
  void clearPartyId() => $_clearField(3);

  /// Empty: unchanged.
  @$pb.TagNumber(4)
  $core.String get label => $_getSZ(3);
  @$pb.TagNumber(4)
  set label($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasLabel() => $_has(3);
  @$pb.TagNumber(4)
  void clearLabel() => $_clearField(4);
}

class ConfigureConnectorResponse extends $pb.GeneratedMessage {
  factory ConfigureConnectorResponse({
    Connector? connector,
  }) {
    final result = ConfigureConnectorResponse._();
    if (connector != null) result.connector = connector;
    return result;
  }

  ConfigureConnectorResponse._();

  factory ConfigureConnectorResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConfigureConnectorResponse()..mergeFromBuffer(data, registry);
  factory ConfigureConnectorResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConfigureConnectorResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConfigureConnectorResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ConfigureConnectorResponse.$_createMessage)
    ..aOM<Connector>(1, _omitFieldNames ? '' : 'connector',
        subBuilder: Connector.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigureConnectorResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConfigureConnectorResponse copyWith(
          void Function(ConfigureConnectorResponse) updates) =>
      super.copyWith(
              (message) => updates(message as ConfigureConnectorResponse))
          as ConfigureConnectorResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ConfigureConnectorResponse() / ConfigureConnectorResponse.new instead')
  static ConfigureConnectorResponse create() => ConfigureConnectorResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      ConfigureConnectorResponse._();
  @$core.override
  ConfigureConnectorResponse createEmptyInstance() =>
      ConfigureConnectorResponse._();
  @$core.pragma('dart2js:noInline')
  static ConfigureConnectorResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ConfigureConnectorResponse>(
          ConfigureConnectorResponse.$_createMessage);
  static ConfigureConnectorResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Connector get connector => $_getN(0);
  @$pb.TagNumber(1)
  set connector(Connector value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasConnector() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnector() => $_clearField(1);
  @$pb.TagNumber(1)
  Connector ensureConnector() => $_ensure(0);
}

class DeleteConnectorRequest extends $pb.GeneratedMessage {
  factory DeleteConnectorRequest({
    $core.String? id,
  }) {
    final result = DeleteConnectorRequest._();
    if (id != null) result.id = id;
    return result;
  }

  DeleteConnectorRequest._();

  factory DeleteConnectorRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteConnectorRequest()..mergeFromBuffer(data, registry);
  factory DeleteConnectorRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteConnectorRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteConnectorRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeleteConnectorRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConnectorRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConnectorRequest copyWith(
          void Function(DeleteConnectorRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteConnectorRequest))
          as DeleteConnectorRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeleteConnectorRequest() / DeleteConnectorRequest.new instead')
  static DeleteConnectorRequest create() => DeleteConnectorRequest._();
  static $pb.GeneratedMessage $_createMessage() => DeleteConnectorRequest._();
  @$core.override
  DeleteConnectorRequest createEmptyInstance() => DeleteConnectorRequest._();
  @$core.pragma('dart2js:noInline')
  static DeleteConnectorRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteConnectorRequest>(
          DeleteConnectorRequest.$_createMessage);
  static DeleteConnectorRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class DeleteConnectorResponse extends $pb.GeneratedMessage {
  factory DeleteConnectorResponse() => DeleteConnectorResponse._();

  DeleteConnectorResponse._();

  factory DeleteConnectorResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteConnectorResponse()..mergeFromBuffer(data, registry);
  factory DeleteConnectorResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteConnectorResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteConnectorResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeleteConnectorResponse.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConnectorResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteConnectorResponse copyWith(
          void Function(DeleteConnectorResponse) updates) =>
      super.copyWith((message) => updates(message as DeleteConnectorResponse))
          as DeleteConnectorResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeleteConnectorResponse() / DeleteConnectorResponse.new instead')
  static DeleteConnectorResponse create() => DeleteConnectorResponse._();
  static $pb.GeneratedMessage $_createMessage() => DeleteConnectorResponse._();
  @$core.override
  DeleteConnectorResponse createEmptyInstance() => DeleteConnectorResponse._();
  @$core.pragma('dart2js:noInline')
  static DeleteConnectorResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteConnectorResponse>(
          DeleteConnectorResponse.$_createMessage);
  static DeleteConnectorResponse? _defaultInstance;
}

class MailTemplate extends $pb.GeneratedMessage {
  factory MailTemplate({
    $core.String? id,
    $core.String? partyId,
    $core.String? name,
    $core.String? subject,
    $core.String? body,
    $core.Iterable<$core.String>? to,
    $core.Iterable<$core.String>? cc,
    $core.Iterable<$core.String>? bcc,
    $core.String? updatedAt,
  }) {
    final result = MailTemplate._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (name != null) result.name = name;
    if (subject != null) result.subject = subject;
    if (body != null) result.body = body;
    if (to != null) result.to.addAll(to);
    if (cc != null) result.cc.addAll(cc);
    if (bcc != null) result.bcc.addAll(bcc);
    if (updatedAt != null) result.updatedAt = updatedAt;
    return result;
  }

  MailTemplate._();

  factory MailTemplate.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailTemplate()..mergeFromBuffer(data, registry);
  factory MailTemplate.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailTemplate()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MailTemplate',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: MailTemplate.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'name')
    ..aOS(4, _omitFieldNames ? '' : 'subject')
    ..aOS(5, _omitFieldNames ? '' : 'body')
    ..pPS(6, _omitFieldNames ? '' : 'to')
    ..pPS(7, _omitFieldNames ? '' : 'cc')
    ..pPS(8, _omitFieldNames ? '' : 'bcc')
    ..aOS(9, _omitFieldNames ? '' : 'updatedAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailTemplate clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailTemplate copyWith(void Function(MailTemplate) updates) =>
      super.copyWith((message) => updates(message as MailTemplate))
          as MailTemplate;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use MailTemplate() / MailTemplate.new instead')
  static MailTemplate create() => MailTemplate._();
  static $pb.GeneratedMessage $_createMessage() => MailTemplate._();
  @$core.override
  MailTemplate createEmptyInstance() => MailTemplate._();
  @$core.pragma('dart2js:noInline')
  static MailTemplate getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<MailTemplate>(
          MailTemplate.$_createMessage);
  static MailTemplate? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get name => $_getSZ(2);
  @$pb.TagNumber(3)
  set name($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasName() => $_has(2);
  @$pb.TagNumber(3)
  void clearName() => $_clearField(3);

  /// With helpers: {{Month}}, {{Year}}, {{MonthName}}, {{Company}}, {{Today}}.
  @$pb.TagNumber(4)
  $core.String get subject => $_getSZ(3);
  @$pb.TagNumber(4)
  set subject($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSubject() => $_has(3);
  @$pb.TagNumber(4)
  void clearSubject() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get body => $_getSZ(4);
  @$pb.TagNumber(5)
  set body($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasBody() => $_has(4);
  @$pb.TagNumber(5)
  void clearBody() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbList<$core.String> get to => $_getList(5);

  @$pb.TagNumber(7)
  $pb.PbList<$core.String> get cc => $_getList(6);

  @$pb.TagNumber(8)
  $pb.PbList<$core.String> get bcc => $_getList(7);

  @$pb.TagNumber(9)
  $core.String get updatedAt => $_getSZ(8);
  @$pb.TagNumber(9)
  set updatedAt($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasUpdatedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearUpdatedAt() => $_clearField(9);
}

class ListMailTemplatesRequest extends $pb.GeneratedMessage {
  factory ListMailTemplatesRequest({
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = ListMailTemplatesRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  ListMailTemplatesRequest._();

  factory ListMailTemplatesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListMailTemplatesRequest()..mergeFromBuffer(data, registry);
  factory ListMailTemplatesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListMailTemplatesRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListMailTemplatesRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListMailTemplatesRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListMailTemplatesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListMailTemplatesRequest copyWith(
          void Function(ListMailTemplatesRequest) updates) =>
      super.copyWith((message) => updates(message as ListMailTemplatesRequest))
          as ListMailTemplatesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListMailTemplatesRequest() / ListMailTemplatesRequest.new instead')
  static ListMailTemplatesRequest create() => ListMailTemplatesRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListMailTemplatesRequest._();
  @$core.override
  ListMailTemplatesRequest createEmptyInstance() =>
      ListMailTemplatesRequest._();
  @$core.pragma('dart2js:noInline')
  static ListMailTemplatesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListMailTemplatesRequest>(
          ListMailTemplatesRequest.$_createMessage);
  static ListMailTemplatesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);
}

class ListMailTemplatesResponse extends $pb.GeneratedMessage {
  factory ListMailTemplatesResponse({
    $core.Iterable<MailTemplate>? templates,
  }) {
    final result = ListMailTemplatesResponse._();
    if (templates != null) result.templates.addAll(templates);
    return result;
  }

  ListMailTemplatesResponse._();

  factory ListMailTemplatesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListMailTemplatesResponse()..mergeFromBuffer(data, registry);
  factory ListMailTemplatesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListMailTemplatesResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListMailTemplatesResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListMailTemplatesResponse.$_createMessage)
    ..pPM<MailTemplate>(1, _omitFieldNames ? '' : 'templates',
        subBuilder: MailTemplate.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListMailTemplatesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListMailTemplatesResponse copyWith(
          void Function(ListMailTemplatesResponse) updates) =>
      super.copyWith((message) => updates(message as ListMailTemplatesResponse))
          as ListMailTemplatesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListMailTemplatesResponse() / ListMailTemplatesResponse.new instead')
  static ListMailTemplatesResponse create() => ListMailTemplatesResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      ListMailTemplatesResponse._();
  @$core.override
  ListMailTemplatesResponse createEmptyInstance() =>
      ListMailTemplatesResponse._();
  @$core.pragma('dart2js:noInline')
  static ListMailTemplatesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListMailTemplatesResponse>(
          ListMailTemplatesResponse.$_createMessage);
  static ListMailTemplatesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<MailTemplate> get templates => $_getList(0);
}

class UpsertMailTemplateRequest extends $pb.GeneratedMessage {
  factory UpsertMailTemplateRequest({
    $core.String? id,
    $core.String? partyId,
    $core.String? name,
    $core.String? subject,
    $core.String? body,
    $core.Iterable<$core.String>? to,
    $core.Iterable<$core.String>? cc,
    $core.Iterable<$core.String>? bcc,
  }) {
    final result = UpsertMailTemplateRequest._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (name != null) result.name = name;
    if (subject != null) result.subject = subject;
    if (body != null) result.body = body;
    if (to != null) result.to.addAll(to);
    if (cc != null) result.cc.addAll(cc);
    if (bcc != null) result.bcc.addAll(bcc);
    return result;
  }

  UpsertMailTemplateRequest._();

  factory UpsertMailTemplateRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertMailTemplateRequest()..mergeFromBuffer(data, registry);
  factory UpsertMailTemplateRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertMailTemplateRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertMailTemplateRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertMailTemplateRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'name')
    ..aOS(4, _omitFieldNames ? '' : 'subject')
    ..aOS(5, _omitFieldNames ? '' : 'body')
    ..pPS(6, _omitFieldNames ? '' : 'to')
    ..pPS(7, _omitFieldNames ? '' : 'cc')
    ..pPS(8, _omitFieldNames ? '' : 'bcc')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertMailTemplateRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertMailTemplateRequest copyWith(
          void Function(UpsertMailTemplateRequest) updates) =>
      super.copyWith((message) => updates(message as UpsertMailTemplateRequest))
          as UpsertMailTemplateRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpsertMailTemplateRequest() / UpsertMailTemplateRequest.new instead')
  static UpsertMailTemplateRequest create() => UpsertMailTemplateRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      UpsertMailTemplateRequest._();
  @$core.override
  UpsertMailTemplateRequest createEmptyInstance() =>
      UpsertMailTemplateRequest._();
  @$core.pragma('dart2js:noInline')
  static UpsertMailTemplateRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertMailTemplateRequest>(
          UpsertMailTemplateRequest.$_createMessage);
  static UpsertMailTemplateRequest? _defaultInstance;

  /// Empty creates.
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get name => $_getSZ(2);
  @$pb.TagNumber(3)
  set name($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasName() => $_has(2);
  @$pb.TagNumber(3)
  void clearName() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get subject => $_getSZ(3);
  @$pb.TagNumber(4)
  set subject($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSubject() => $_has(3);
  @$pb.TagNumber(4)
  void clearSubject() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get body => $_getSZ(4);
  @$pb.TagNumber(5)
  set body($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasBody() => $_has(4);
  @$pb.TagNumber(5)
  void clearBody() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbList<$core.String> get to => $_getList(5);

  @$pb.TagNumber(7)
  $pb.PbList<$core.String> get cc => $_getList(6);

  @$pb.TagNumber(8)
  $pb.PbList<$core.String> get bcc => $_getList(7);
}

class UpsertMailTemplateResponse extends $pb.GeneratedMessage {
  factory UpsertMailTemplateResponse({
    MailTemplate? template,
  }) {
    final result = UpsertMailTemplateResponse._();
    if (template != null) result.template = template;
    return result;
  }

  UpsertMailTemplateResponse._();

  factory UpsertMailTemplateResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertMailTemplateResponse()..mergeFromBuffer(data, registry);
  factory UpsertMailTemplateResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertMailTemplateResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertMailTemplateResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertMailTemplateResponse.$_createMessage)
    ..aOM<MailTemplate>(1, _omitFieldNames ? '' : 'template',
        subBuilder: MailTemplate.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertMailTemplateResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertMailTemplateResponse copyWith(
          void Function(UpsertMailTemplateResponse) updates) =>
      super.copyWith(
              (message) => updates(message as UpsertMailTemplateResponse))
          as UpsertMailTemplateResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpsertMailTemplateResponse() / UpsertMailTemplateResponse.new instead')
  static UpsertMailTemplateResponse create() => UpsertMailTemplateResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      UpsertMailTemplateResponse._();
  @$core.override
  UpsertMailTemplateResponse createEmptyInstance() =>
      UpsertMailTemplateResponse._();
  @$core.pragma('dart2js:noInline')
  static UpsertMailTemplateResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertMailTemplateResponse>(
          UpsertMailTemplateResponse.$_createMessage);
  static UpsertMailTemplateResponse? _defaultInstance;

  @$pb.TagNumber(1)
  MailTemplate get template => $_getN(0);
  @$pb.TagNumber(1)
  set template(MailTemplate value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTemplate() => $_has(0);
  @$pb.TagNumber(1)
  void clearTemplate() => $_clearField(1);
  @$pb.TagNumber(1)
  MailTemplate ensureTemplate() => $_ensure(0);
}

class DeleteMailTemplateRequest extends $pb.GeneratedMessage {
  factory DeleteMailTemplateRequest({
    $core.String? id,
  }) {
    final result = DeleteMailTemplateRequest._();
    if (id != null) result.id = id;
    return result;
  }

  DeleteMailTemplateRequest._();

  factory DeleteMailTemplateRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteMailTemplateRequest()..mergeFromBuffer(data, registry);
  factory DeleteMailTemplateRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteMailTemplateRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteMailTemplateRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeleteMailTemplateRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteMailTemplateRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteMailTemplateRequest copyWith(
          void Function(DeleteMailTemplateRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteMailTemplateRequest))
          as DeleteMailTemplateRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeleteMailTemplateRequest() / DeleteMailTemplateRequest.new instead')
  static DeleteMailTemplateRequest create() => DeleteMailTemplateRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      DeleteMailTemplateRequest._();
  @$core.override
  DeleteMailTemplateRequest createEmptyInstance() =>
      DeleteMailTemplateRequest._();
  @$core.pragma('dart2js:noInline')
  static DeleteMailTemplateRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteMailTemplateRequest>(
          DeleteMailTemplateRequest.$_createMessage);
  static DeleteMailTemplateRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class DeleteMailTemplateResponse extends $pb.GeneratedMessage {
  factory DeleteMailTemplateResponse() => DeleteMailTemplateResponse._();

  DeleteMailTemplateResponse._();

  factory DeleteMailTemplateResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteMailTemplateResponse()..mergeFromBuffer(data, registry);
  factory DeleteMailTemplateResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteMailTemplateResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteMailTemplateResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeleteMailTemplateResponse.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteMailTemplateResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteMailTemplateResponse copyWith(
          void Function(DeleteMailTemplateResponse) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteMailTemplateResponse))
          as DeleteMailTemplateResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeleteMailTemplateResponse() / DeleteMailTemplateResponse.new instead')
  static DeleteMailTemplateResponse create() => DeleteMailTemplateResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      DeleteMailTemplateResponse._();
  @$core.override
  DeleteMailTemplateResponse createEmptyInstance() =>
      DeleteMailTemplateResponse._();
  @$core.pragma('dart2js:noInline')
  static DeleteMailTemplateResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteMailTemplateResponse>(
          DeleteMailTemplateResponse.$_createMessage);
  static DeleteMailTemplateResponse? _defaultInstance;
}

class MailDocument extends $pb.GeneratedMessage {
  factory MailDocument({
    $core.String? documentId,
    $core.String? filename,
    $core.String? contentType,
    $fixnum.Int64? sizeBytes,
  }) {
    final result = MailDocument._();
    if (documentId != null) result.documentId = documentId;
    if (filename != null) result.filename = filename;
    if (contentType != null) result.contentType = contentType;
    if (sizeBytes != null) result.sizeBytes = sizeBytes;
    return result;
  }

  MailDocument._();

  factory MailDocument.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailDocument()..mergeFromBuffer(data, registry);
  factory MailDocument.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailDocument()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MailDocument',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: MailDocument.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'documentId')
    ..aOS(2, _omitFieldNames ? '' : 'filename')
    ..aOS(3, _omitFieldNames ? '' : 'contentType')
    ..aInt64(4, _omitFieldNames ? '' : 'sizeBytes')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailDocument clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailDocument copyWith(void Function(MailDocument) updates) =>
      super.copyWith((message) => updates(message as MailDocument))
          as MailDocument;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use MailDocument() / MailDocument.new instead')
  static MailDocument create() => MailDocument._();
  static $pb.GeneratedMessage $_createMessage() => MailDocument._();
  @$core.override
  MailDocument createEmptyInstance() => MailDocument._();
  @$core.pragma('dart2js:noInline')
  static MailDocument getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<MailDocument>(
          MailDocument.$_createMessage);
  static MailDocument? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get documentId => $_getSZ(0);
  @$pb.TagNumber(1)
  set documentId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasDocumentId() => $_has(0);
  @$pb.TagNumber(1)
  void clearDocumentId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get filename => $_getSZ(1);
  @$pb.TagNumber(2)
  set filename($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFilename() => $_has(1);
  @$pb.TagNumber(2)
  void clearFilename() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get contentType => $_getSZ(2);
  @$pb.TagNumber(3)
  set contentType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasContentType() => $_has(2);
  @$pb.TagNumber(3)
  void clearContentType() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get sizeBytes => $_getI64(3);
  @$pb.TagNumber(4)
  set sizeBytes($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSizeBytes() => $_has(3);
  @$pb.TagNumber(4)
  void clearSizeBytes() => $_clearField(4);
}

class Mail extends $pb.GeneratedMessage {
  factory Mail({
    $core.String? id,
    $core.String? partyId,
    $core.String? connectorId,
    $core.String? direction,
    $core.String? from,
    $core.Iterable<$core.String>? to,
    $core.Iterable<$core.String>? cc,
    $core.Iterable<$core.String>? bcc,
    $core.String? subject,
    $core.String? body,
    $core.String? status,
    $core.String? error,
    $core.String? sentAt,
    $core.String? receivedAt,
    $core.String? templateId,
    $core.String? parentId,
    $core.int? replies,
    $core.Iterable<MailDocument>? documents,
    $core.String? threadKey,
    $core.String? bundle,
  }) {
    final result = Mail._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (connectorId != null) result.connectorId = connectorId;
    if (direction != null) result.direction = direction;
    if (from != null) result.from = from;
    if (to != null) result.to.addAll(to);
    if (cc != null) result.cc.addAll(cc);
    if (bcc != null) result.bcc.addAll(bcc);
    if (subject != null) result.subject = subject;
    if (body != null) result.body = body;
    if (status != null) result.status = status;
    if (error != null) result.error = error;
    if (sentAt != null) result.sentAt = sentAt;
    if (receivedAt != null) result.receivedAt = receivedAt;
    if (templateId != null) result.templateId = templateId;
    if (parentId != null) result.parentId = parentId;
    if (replies != null) result.replies = replies;
    if (documents != null) result.documents.addAll(documents);
    if (threadKey != null) result.threadKey = threadKey;
    if (bundle != null) result.bundle = bundle;
    return result;
  }

  Mail._();

  factory Mail.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Mail()..mergeFromBuffer(data, registry);
  factory Mail.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Mail()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Mail',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Mail.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'connectorId')
    ..aOS(4, _omitFieldNames ? '' : 'direction')
    ..aOS(5, _omitFieldNames ? '' : 'from')
    ..pPS(6, _omitFieldNames ? '' : 'to')
    ..pPS(7, _omitFieldNames ? '' : 'cc')
    ..pPS(8, _omitFieldNames ? '' : 'bcc')
    ..aOS(9, _omitFieldNames ? '' : 'subject')
    ..aOS(10, _omitFieldNames ? '' : 'body')
    ..aOS(11, _omitFieldNames ? '' : 'status')
    ..aOS(12, _omitFieldNames ? '' : 'error')
    ..aOS(13, _omitFieldNames ? '' : 'sentAt')
    ..aOS(14, _omitFieldNames ? '' : 'receivedAt')
    ..aOS(15, _omitFieldNames ? '' : 'templateId')
    ..aOS(16, _omitFieldNames ? '' : 'parentId')
    ..aI(17, _omitFieldNames ? '' : 'replies', fieldType: $pb.PbFieldType.OU3)
    ..pPM<MailDocument>(18, _omitFieldNames ? '' : 'documents',
        subBuilder: MailDocument.$_createMessage)
    ..aOS(19, _omitFieldNames ? '' : 'threadKey')
    ..aOS(20, _omitFieldNames ? '' : 'bundle')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Mail clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Mail copyWith(void Function(Mail) updates) =>
      super.copyWith((message) => updates(message as Mail)) as Mail;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Mail() / Mail.new instead')
  static Mail create() => Mail._();
  static $pb.GeneratedMessage $_createMessage() => Mail._();
  @$core.override
  Mail createEmptyInstance() => Mail._();
  @$core.pragma('dart2js:noInline')
  static Mail getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Mail>(Mail.$_createMessage);
  static Mail? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get connectorId => $_getSZ(2);
  @$pb.TagNumber(3)
  set connectorId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasConnectorId() => $_has(2);
  @$pb.TagNumber(3)
  void clearConnectorId() => $_clearField(3);

  /// out or in.
  @$pb.TagNumber(4)
  $core.String get direction => $_getSZ(3);
  @$pb.TagNumber(4)
  set direction($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDirection() => $_has(3);
  @$pb.TagNumber(4)
  void clearDirection() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get from => $_getSZ(4);
  @$pb.TagNumber(5)
  set from($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasFrom() => $_has(4);
  @$pb.TagNumber(5)
  void clearFrom() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbList<$core.String> get to => $_getList(5);

  @$pb.TagNumber(7)
  $pb.PbList<$core.String> get cc => $_getList(6);

  @$pb.TagNumber(8)
  $pb.PbList<$core.String> get bcc => $_getList(7);

  @$pb.TagNumber(9)
  $core.String get subject => $_getSZ(8);
  @$pb.TagNumber(9)
  set subject($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasSubject() => $_has(8);
  @$pb.TagNumber(9)
  void clearSubject() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get body => $_getSZ(9);
  @$pb.TagNumber(10)
  set body($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasBody() => $_has(9);
  @$pb.TagNumber(10)
  void clearBody() => $_clearField(10);

  /// sent, failed, received.
  @$pb.TagNumber(11)
  $core.String get status => $_getSZ(10);
  @$pb.TagNumber(11)
  set status($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasStatus() => $_has(10);
  @$pb.TagNumber(11)
  void clearStatus() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get error => $_getSZ(11);
  @$pb.TagNumber(12)
  set error($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasError() => $_has(11);
  @$pb.TagNumber(12)
  void clearError() => $_clearField(12);

  @$pb.TagNumber(13)
  $core.String get sentAt => $_getSZ(12);
  @$pb.TagNumber(13)
  set sentAt($core.String value) => $_setString(12, value);
  @$pb.TagNumber(13)
  $core.bool hasSentAt() => $_has(12);
  @$pb.TagNumber(13)
  void clearSentAt() => $_clearField(13);

  @$pb.TagNumber(14)
  $core.String get receivedAt => $_getSZ(13);
  @$pb.TagNumber(14)
  set receivedAt($core.String value) => $_setString(13, value);
  @$pb.TagNumber(14)
  $core.bool hasReceivedAt() => $_has(13);
  @$pb.TagNumber(14)
  void clearReceivedAt() => $_clearField(14);

  @$pb.TagNumber(15)
  $core.String get templateId => $_getSZ(14);
  @$pb.TagNumber(15)
  set templateId($core.String value) => $_setString(14, value);
  @$pb.TagNumber(15)
  $core.bool hasTemplateId() => $_has(14);
  @$pb.TagNumber(15)
  void clearTemplateId() => $_clearField(15);

  /// The outgoing mail a reply answers.
  @$pb.TagNumber(16)
  $core.String get parentId => $_getSZ(15);
  @$pb.TagNumber(16)
  set parentId($core.String value) => $_setString(15, value);
  @$pb.TagNumber(16)
  $core.bool hasParentId() => $_has(15);
  @$pb.TagNumber(16)
  void clearParentId() => $_clearField(16);

  /// Replies held for an outgoing mail.
  @$pb.TagNumber(17)
  $core.int get replies => $_getIZ(16);
  @$pb.TagNumber(17)
  set replies($core.int value) => $_setUnsignedInt32(16, value);
  @$pb.TagNumber(17)
  $core.bool hasReplies() => $_has(16);
  @$pb.TagNumber(17)
  void clearReplies() => $_clearField(17);

  @$pb.TagNumber(18)
  $pb.PbList<MailDocument> get documents => $_getList(17);

  @$pb.TagNumber(19)
  $core.String get threadKey => $_getSZ(18);
  @$pb.TagNumber(19)
  set threadKey($core.String value) => $_setString(18, value);
  @$pb.TagNumber(19)
  $core.bool hasThreadKey() => $_has(18);
  @$pb.TagNumber(19)
  void clearThreadKey() => $_clearField(19);

  /// The bundle zip's file name, when one went with the mail.
  @$pb.TagNumber(20)
  $core.String get bundle => $_getSZ(19);
  @$pb.TagNumber(20)
  set bundle($core.String value) => $_setString(19, value);
  @$pb.TagNumber(20)
  $core.bool hasBundle() => $_has(19);
  @$pb.TagNumber(20)
  void clearBundle() => $_clearField(20);
}

class SendMailRequest extends $pb.GeneratedMessage {
  factory SendMailRequest({
    $core.String? connectorId,
    $core.String? templateId,
    $core.Iterable<$core.String>? to,
    $core.Iterable<$core.String>? cc,
    $core.Iterable<$core.String>? bcc,
    $core.String? subject,
    $core.String? body,
    $core.String? html,
    $core.Iterable<$core.String>? attachmentDocumentIds,
    $core.String? inReplyToMailId,
    MailBundle? bundle,
  }) {
    final result = SendMailRequest._();
    if (connectorId != null) result.connectorId = connectorId;
    if (templateId != null) result.templateId = templateId;
    if (to != null) result.to.addAll(to);
    if (cc != null) result.cc.addAll(cc);
    if (bcc != null) result.bcc.addAll(bcc);
    if (subject != null) result.subject = subject;
    if (body != null) result.body = body;
    if (html != null) result.html = html;
    if (attachmentDocumentIds != null)
      result.attachmentDocumentIds.addAll(attachmentDocumentIds);
    if (inReplyToMailId != null) result.inReplyToMailId = inReplyToMailId;
    if (bundle != null) result.bundle = bundle;
    return result;
  }

  SendMailRequest._();

  factory SendMailRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SendMailRequest()..mergeFromBuffer(data, registry);
  factory SendMailRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SendMailRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SendMailRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SendMailRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'connectorId')
    ..aOS(2, _omitFieldNames ? '' : 'templateId')
    ..pPS(3, _omitFieldNames ? '' : 'to')
    ..pPS(4, _omitFieldNames ? '' : 'cc')
    ..pPS(5, _omitFieldNames ? '' : 'bcc')
    ..aOS(6, _omitFieldNames ? '' : 'subject')
    ..aOS(7, _omitFieldNames ? '' : 'body')
    ..aOS(8, _omitFieldNames ? '' : 'html')
    ..pPS(9, _omitFieldNames ? '' : 'attachmentDocumentIds')
    ..aOS(10, _omitFieldNames ? '' : 'inReplyToMailId')
    ..aOM<MailBundle>(11, _omitFieldNames ? '' : 'bundle',
        subBuilder: MailBundle.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SendMailRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SendMailRequest copyWith(void Function(SendMailRequest) updates) =>
      super.copyWith((message) => updates(message as SendMailRequest))
          as SendMailRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SendMailRequest() / SendMailRequest.new instead')
  static SendMailRequest create() => SendMailRequest._();
  static $pb.GeneratedMessage $_createMessage() => SendMailRequest._();
  @$core.override
  SendMailRequest createEmptyInstance() => SendMailRequest._();
  @$core.pragma('dart2js:noInline')
  static SendMailRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SendMailRequest>(
          SendMailRequest.$_createMessage);
  static SendMailRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get connectorId => $_getSZ(0);
  @$pb.TagNumber(1)
  set connectorId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasConnectorId() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnectorId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get templateId => $_getSZ(1);
  @$pb.TagNumber(2)
  set templateId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTemplateId() => $_has(1);
  @$pb.TagNumber(2)
  void clearTemplateId() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<$core.String> get to => $_getList(2);

  @$pb.TagNumber(4)
  $pb.PbList<$core.String> get cc => $_getList(3);

  @$pb.TagNumber(5)
  $pb.PbList<$core.String> get bcc => $_getList(4);

  /// Rendered: helpers already filled.
  @$pb.TagNumber(6)
  $core.String get subject => $_getSZ(5);
  @$pb.TagNumber(6)
  set subject($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSubject() => $_has(5);
  @$pb.TagNumber(6)
  void clearSubject() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get body => $_getSZ(6);
  @$pb.TagNumber(7)
  set body($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasBody() => $_has(6);
  @$pb.TagNumber(7)
  void clearBody() => $_clearField(7);

  /// Optional HTML beside the text.
  @$pb.TagNumber(8)
  $core.String get html => $_getSZ(7);
  @$pb.TagNumber(8)
  set html($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasHtml() => $_has(7);
  @$pb.TagNumber(8)
  void clearHtml() => $_clearField(8);

  @$pb.TagNumber(9)
  $pb.PbList<$core.String> get attachmentDocumentIds => $_getList(8);

  /// A mail of ours to answer in its thread.
  @$pb.TagNumber(10)
  $core.String get inReplyToMailId => $_getSZ(9);
  @$pb.TagNumber(10)
  set inReplyToMailId($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasInReplyToMailId() => $_has(9);
  @$pb.TagNumber(10)
  void clearInReplyToMailId() => $_clearField(10);

  /// The accountant's bundle, built by the service into one zip attachment.
  @$pb.TagNumber(11)
  MailBundle get bundle => $_getN(10);
  @$pb.TagNumber(11)
  set bundle(MailBundle value) => $_setField(11, value);
  @$pb.TagNumber(11)
  $core.bool hasBundle() => $_has(10);
  @$pb.TagNumber(11)
  void clearBundle() => $_clearField(11);
  @$pb.TagNumber(11)
  MailBundle ensureBundle() => $_ensure(10);
}

/// The bundle as the page prepared it: small text files with their bytes,
/// and the receipts by document with the name each takes inside the zip.
class MailBundle extends $pb.GeneratedMessage {
  factory MailBundle({
    $core.String? filename,
    $core.Iterable<MailBundleFile>? files,
    $core.Iterable<MailBundleReceipt>? receipts,
  }) {
    final result = MailBundle._();
    if (filename != null) result.filename = filename;
    if (files != null) result.files.addAll(files);
    if (receipts != null) result.receipts.addAll(receipts);
    return result;
  }

  MailBundle._();

  factory MailBundle.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailBundle()..mergeFromBuffer(data, registry);
  factory MailBundle.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailBundle()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MailBundle',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: MailBundle.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'filename')
    ..pPM<MailBundleFile>(2, _omitFieldNames ? '' : 'files',
        subBuilder: MailBundleFile.$_createMessage)
    ..pPM<MailBundleReceipt>(3, _omitFieldNames ? '' : 'receipts',
        subBuilder: MailBundleReceipt.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailBundle clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailBundle copyWith(void Function(MailBundle) updates) =>
      super.copyWith((message) => updates(message as MailBundle)) as MailBundle;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use MailBundle() / MailBundle.new instead')
  static MailBundle create() => MailBundle._();
  static $pb.GeneratedMessage $_createMessage() => MailBundle._();
  @$core.override
  MailBundle createEmptyInstance() => MailBundle._();
  @$core.pragma('dart2js:noInline')
  static MailBundle getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MailBundle>(MailBundle.$_createMessage);
  static MailBundle? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get filename => $_getSZ(0);
  @$pb.TagNumber(1)
  set filename($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasFilename() => $_has(0);
  @$pb.TagNumber(1)
  void clearFilename() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<MailBundleFile> get files => $_getList(1);

  @$pb.TagNumber(3)
  $pb.PbList<MailBundleReceipt> get receipts => $_getList(2);
}

class MailBundleFile extends $pb.GeneratedMessage {
  factory MailBundleFile({
    $core.String? name,
    $core.List<$core.int>? bytes,
  }) {
    final result = MailBundleFile._();
    if (name != null) result.name = name;
    if (bytes != null) result.bytes = bytes;
    return result;
  }

  MailBundleFile._();

  factory MailBundleFile.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailBundleFile()..mergeFromBuffer(data, registry);
  factory MailBundleFile.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailBundleFile()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MailBundleFile',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: MailBundleFile.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'name')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'bytes', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailBundleFile clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailBundleFile copyWith(void Function(MailBundleFile) updates) =>
      super.copyWith((message) => updates(message as MailBundleFile))
          as MailBundleFile;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use MailBundleFile() / MailBundleFile.new instead')
  static MailBundleFile create() => MailBundleFile._();
  static $pb.GeneratedMessage $_createMessage() => MailBundleFile._();
  @$core.override
  MailBundleFile createEmptyInstance() => MailBundleFile._();
  @$core.pragma('dart2js:noInline')
  static MailBundleFile getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<MailBundleFile>(
          MailBundleFile.$_createMessage);
  static MailBundleFile? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get name => $_getSZ(0);
  @$pb.TagNumber(1)
  set name($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasName() => $_has(0);
  @$pb.TagNumber(1)
  void clearName() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get bytes => $_getN(1);
  @$pb.TagNumber(2)
  set bytes($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasBytes() => $_has(1);
  @$pb.TagNumber(2)
  void clearBytes() => $_clearField(2);
}

class MailBundleReceipt extends $pb.GeneratedMessage {
  factory MailBundleReceipt({
    $core.String? documentId,
    $core.String? name,
  }) {
    final result = MailBundleReceipt._();
    if (documentId != null) result.documentId = documentId;
    if (name != null) result.name = name;
    return result;
  }

  MailBundleReceipt._();

  factory MailBundleReceipt.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailBundleReceipt()..mergeFromBuffer(data, registry);
  factory MailBundleReceipt.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MailBundleReceipt()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MailBundleReceipt',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: MailBundleReceipt.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'documentId')
    ..aOS(2, _omitFieldNames ? '' : 'name')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailBundleReceipt clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MailBundleReceipt copyWith(void Function(MailBundleReceipt) updates) =>
      super.copyWith((message) => updates(message as MailBundleReceipt))
          as MailBundleReceipt;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use MailBundleReceipt() / MailBundleReceipt.new instead')
  static MailBundleReceipt create() => MailBundleReceipt._();
  static $pb.GeneratedMessage $_createMessage() => MailBundleReceipt._();
  @$core.override
  MailBundleReceipt createEmptyInstance() => MailBundleReceipt._();
  @$core.pragma('dart2js:noInline')
  static MailBundleReceipt getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<MailBundleReceipt>(
          MailBundleReceipt.$_createMessage);
  static MailBundleReceipt? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get documentId => $_getSZ(0);
  @$pb.TagNumber(1)
  set documentId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasDocumentId() => $_has(0);
  @$pb.TagNumber(1)
  void clearDocumentId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get name => $_getSZ(1);
  @$pb.TagNumber(2)
  set name($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasName() => $_has(1);
  @$pb.TagNumber(2)
  void clearName() => $_clearField(2);
}

class SendMailResponse extends $pb.GeneratedMessage {
  factory SendMailResponse({
    Mail? mail,
  }) {
    final result = SendMailResponse._();
    if (mail != null) result.mail = mail;
    return result;
  }

  SendMailResponse._();

  factory SendMailResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SendMailResponse()..mergeFromBuffer(data, registry);
  factory SendMailResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SendMailResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SendMailResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SendMailResponse.$_createMessage)
    ..aOM<Mail>(1, _omitFieldNames ? '' : 'mail',
        subBuilder: Mail.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SendMailResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SendMailResponse copyWith(void Function(SendMailResponse) updates) =>
      super.copyWith((message) => updates(message as SendMailResponse))
          as SendMailResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SendMailResponse() / SendMailResponse.new instead')
  static SendMailResponse create() => SendMailResponse._();
  static $pb.GeneratedMessage $_createMessage() => SendMailResponse._();
  @$core.override
  SendMailResponse createEmptyInstance() => SendMailResponse._();
  @$core.pragma('dart2js:noInline')
  static SendMailResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<SendMailResponse>(
          SendMailResponse.$_createMessage);
  static SendMailResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Mail get mail => $_getN(0);
  @$pb.TagNumber(1)
  set mail(Mail value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMail() => $_has(0);
  @$pb.TagNumber(1)
  void clearMail() => $_clearField(1);
  @$pb.TagNumber(1)
  Mail ensureMail() => $_ensure(0);
}

class ListMailRequest extends $pb.GeneratedMessage {
  factory ListMailRequest({
    $core.Iterable<$core.String>? partyIds,
    $core.String? connectorId,
    $core.String? direction,
    $core.String? q,
    $core.int? limit,
    $core.int? offset,
  }) {
    final result = ListMailRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    if (connectorId != null) result.connectorId = connectorId;
    if (direction != null) result.direction = direction;
    if (q != null) result.q = q;
    if (limit != null) result.limit = limit;
    if (offset != null) result.offset = offset;
    return result;
  }

  ListMailRequest._();

  factory ListMailRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListMailRequest()..mergeFromBuffer(data, registry);
  factory ListMailRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListMailRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListMailRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListMailRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..aOS(2, _omitFieldNames ? '' : 'connectorId')
    ..aOS(3, _omitFieldNames ? '' : 'direction')
    ..aOS(4, _omitFieldNames ? '' : 'q')
    ..aI(5, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..aI(6, _omitFieldNames ? '' : 'offset', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListMailRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListMailRequest copyWith(void Function(ListMailRequest) updates) =>
      super.copyWith((message) => updates(message as ListMailRequest))
          as ListMailRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ListMailRequest() / ListMailRequest.new instead')
  static ListMailRequest create() => ListMailRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListMailRequest._();
  @$core.override
  ListMailRequest createEmptyInstance() => ListMailRequest._();
  @$core.pragma('dart2js:noInline')
  static ListMailRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ListMailRequest>(
          ListMailRequest.$_createMessage);
  static ListMailRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);

  @$pb.TagNumber(2)
  $core.String get connectorId => $_getSZ(1);
  @$pb.TagNumber(2)
  set connectorId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasConnectorId() => $_has(1);
  @$pb.TagNumber(2)
  void clearConnectorId() => $_clearField(2);

  /// out, in, or empty.
  @$pb.TagNumber(3)
  $core.String get direction => $_getSZ(2);
  @$pb.TagNumber(3)
  set direction($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDirection() => $_has(2);
  @$pb.TagNumber(3)
  void clearDirection() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get q => $_getSZ(3);
  @$pb.TagNumber(4)
  set q($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasQ() => $_has(3);
  @$pb.TagNumber(4)
  void clearQ() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get limit => $_getIZ(4);
  @$pb.TagNumber(5)
  set limit($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasLimit() => $_has(4);
  @$pb.TagNumber(5)
  void clearLimit() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get offset => $_getIZ(5);
  @$pb.TagNumber(6)
  set offset($core.int value) => $_setUnsignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasOffset() => $_has(5);
  @$pb.TagNumber(6)
  void clearOffset() => $_clearField(6);
}

class ListMailResponse extends $pb.GeneratedMessage {
  factory ListMailResponse({
    $core.Iterable<Mail>? mails,
    $core.int? total,
  }) {
    final result = ListMailResponse._();
    if (mails != null) result.mails.addAll(mails);
    if (total != null) result.total = total;
    return result;
  }

  ListMailResponse._();

  factory ListMailResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListMailResponse()..mergeFromBuffer(data, registry);
  factory ListMailResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListMailResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListMailResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListMailResponse.$_createMessage)
    ..pPM<Mail>(1, _omitFieldNames ? '' : 'mails',
        subBuilder: Mail.$_createMessage)
    ..aI(2, _omitFieldNames ? '' : 'total', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListMailResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListMailResponse copyWith(void Function(ListMailResponse) updates) =>
      super.copyWith((message) => updates(message as ListMailResponse))
          as ListMailResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ListMailResponse() / ListMailResponse.new instead')
  static ListMailResponse create() => ListMailResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListMailResponse._();
  @$core.override
  ListMailResponse createEmptyInstance() => ListMailResponse._();
  @$core.pragma('dart2js:noInline')
  static ListMailResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ListMailResponse>(
          ListMailResponse.$_createMessage);
  static ListMailResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Mail> get mails => $_getList(0);

  @$pb.TagNumber(2)
  $core.int get total => $_getIZ(1);
  @$pb.TagNumber(2)
  set total($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTotal() => $_has(1);
  @$pb.TagNumber(2)
  void clearTotal() => $_clearField(2);
}

class GetMailRequest extends $pb.GeneratedMessage {
  factory GetMailRequest({
    $core.String? id,
  }) {
    final result = GetMailRequest._();
    if (id != null) result.id = id;
    return result;
  }

  GetMailRequest._();

  factory GetMailRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetMailRequest()..mergeFromBuffer(data, registry);
  factory GetMailRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetMailRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetMailRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetMailRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetMailRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetMailRequest copyWith(void Function(GetMailRequest) updates) =>
      super.copyWith((message) => updates(message as GetMailRequest))
          as GetMailRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetMailRequest() / GetMailRequest.new instead')
  static GetMailRequest create() => GetMailRequest._();
  static $pb.GeneratedMessage $_createMessage() => GetMailRequest._();
  @$core.override
  GetMailRequest createEmptyInstance() => GetMailRequest._();
  @$core.pragma('dart2js:noInline')
  static GetMailRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetMailRequest>(
          GetMailRequest.$_createMessage);
  static GetMailRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetMailResponse extends $pb.GeneratedMessage {
  factory GetMailResponse({
    Mail? mail,
    $core.Iterable<Mail>? thread,
  }) {
    final result = GetMailResponse._();
    if (mail != null) result.mail = mail;
    if (thread != null) result.thread.addAll(thread);
    return result;
  }

  GetMailResponse._();

  factory GetMailResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetMailResponse()..mergeFromBuffer(data, registry);
  factory GetMailResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetMailResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetMailResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetMailResponse.$_createMessage)
    ..aOM<Mail>(1, _omitFieldNames ? '' : 'mail',
        subBuilder: Mail.$_createMessage)
    ..pPM<Mail>(2, _omitFieldNames ? '' : 'thread',
        subBuilder: Mail.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetMailResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetMailResponse copyWith(void Function(GetMailResponse) updates) =>
      super.copyWith((message) => updates(message as GetMailResponse))
          as GetMailResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetMailResponse() / GetMailResponse.new instead')
  static GetMailResponse create() => GetMailResponse._();
  static $pb.GeneratedMessage $_createMessage() => GetMailResponse._();
  @$core.override
  GetMailResponse createEmptyInstance() => GetMailResponse._();
  @$core.pragma('dart2js:noInline')
  static GetMailResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetMailResponse>(
          GetMailResponse.$_createMessage);
  static GetMailResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Mail get mail => $_getN(0);
  @$pb.TagNumber(1)
  set mail(Mail value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasMail() => $_has(0);
  @$pb.TagNumber(1)
  void clearMail() => $_clearField(1);
  @$pb.TagNumber(1)
  Mail ensureMail() => $_ensure(0);

  /// The whole thread, oldest first; `mail` is among them.
  @$pb.TagNumber(2)
  $pb.PbList<Mail> get thread => $_getList(1);
}

class ConnectorRun extends $pb.GeneratedMessage {
  factory ConnectorRun({
    $core.String? id,
    $core.String? startedAt,
    $core.String? finishedAt,
    $core.String? trigger,
    $core.String? outcome,
    $core.int? found,
    $core.int? stored,
    $core.int? skipped,
    $core.String? error,
  }) {
    final result = ConnectorRun._();
    if (id != null) result.id = id;
    if (startedAt != null) result.startedAt = startedAt;
    if (finishedAt != null) result.finishedAt = finishedAt;
    if (trigger != null) result.trigger = trigger;
    if (outcome != null) result.outcome = outcome;
    if (found != null) result.found = found;
    if (stored != null) result.stored = stored;
    if (skipped != null) result.skipped = skipped;
    if (error != null) result.error = error;
    return result;
  }

  ConnectorRun._();

  factory ConnectorRun.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConnectorRun()..mergeFromBuffer(data, registry);
  factory ConnectorRun.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ConnectorRun()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ConnectorRun',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ConnectorRun.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'startedAt')
    ..aOS(3, _omitFieldNames ? '' : 'finishedAt')
    ..aOS(4, _omitFieldNames ? '' : 'trigger')
    ..aOS(5, _omitFieldNames ? '' : 'outcome')
    ..aI(6, _omitFieldNames ? '' : 'found', fieldType: $pb.PbFieldType.OU3)
    ..aI(7, _omitFieldNames ? '' : 'stored', fieldType: $pb.PbFieldType.OU3)
    ..aI(8, _omitFieldNames ? '' : 'skipped', fieldType: $pb.PbFieldType.OU3)
    ..aOS(9, _omitFieldNames ? '' : 'error')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectorRun clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ConnectorRun copyWith(void Function(ConnectorRun) updates) =>
      super.copyWith((message) => updates(message as ConnectorRun))
          as ConnectorRun;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ConnectorRun() / ConnectorRun.new instead')
  static ConnectorRun create() => ConnectorRun._();
  static $pb.GeneratedMessage $_createMessage() => ConnectorRun._();
  @$core.override
  ConnectorRun createEmptyInstance() => ConnectorRun._();
  @$core.pragma('dart2js:noInline')
  static ConnectorRun getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ConnectorRun>(
          ConnectorRun.$_createMessage);
  static ConnectorRun? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get startedAt => $_getSZ(1);
  @$pb.TagNumber(2)
  set startedAt($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasStartedAt() => $_has(1);
  @$pb.TagNumber(2)
  void clearStartedAt() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get finishedAt => $_getSZ(2);
  @$pb.TagNumber(3)
  set finishedAt($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFinishedAt() => $_has(2);
  @$pb.TagNumber(3)
  void clearFinishedAt() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get trigger => $_getSZ(3);
  @$pb.TagNumber(4)
  set trigger($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTrigger() => $_has(3);
  @$pb.TagNumber(4)
  void clearTrigger() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get outcome => $_getSZ(4);
  @$pb.TagNumber(5)
  set outcome($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasOutcome() => $_has(4);
  @$pb.TagNumber(5)
  void clearOutcome() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get found => $_getIZ(5);
  @$pb.TagNumber(6)
  set found($core.int value) => $_setUnsignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasFound() => $_has(5);
  @$pb.TagNumber(6)
  void clearFound() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.int get stored => $_getIZ(6);
  @$pb.TagNumber(7)
  set stored($core.int value) => $_setUnsignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasStored() => $_has(6);
  @$pb.TagNumber(7)
  void clearStored() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.int get skipped => $_getIZ(7);
  @$pb.TagNumber(8)
  set skipped($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasSkipped() => $_has(7);
  @$pb.TagNumber(8)
  void clearSkipped() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get error => $_getSZ(8);
  @$pb.TagNumber(9)
  set error($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasError() => $_has(8);
  @$pb.TagNumber(9)
  void clearError() => $_clearField(9);
}

class WatchConnectorsRequest extends $pb.GeneratedMessage {
  factory WatchConnectorsRequest({
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = WatchConnectorsRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  WatchConnectorsRequest._();

  factory WatchConnectorsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      WatchConnectorsRequest()..mergeFromBuffer(data, registry);
  factory WatchConnectorsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      WatchConnectorsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WatchConnectorsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: WatchConnectorsRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchConnectorsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchConnectorsRequest copyWith(
          void Function(WatchConnectorsRequest) updates) =>
      super.copyWith((message) => updates(message as WatchConnectorsRequest))
          as WatchConnectorsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use WatchConnectorsRequest() / WatchConnectorsRequest.new instead')
  static WatchConnectorsRequest create() => WatchConnectorsRequest._();
  static $pb.GeneratedMessage $_createMessage() => WatchConnectorsRequest._();
  @$core.override
  WatchConnectorsRequest createEmptyInstance() => WatchConnectorsRequest._();
  @$core.pragma('dart2js:noInline')
  static WatchConnectorsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WatchConnectorsRequest>(
          WatchConnectorsRequest.$_createMessage);
  static WatchConnectorsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);
}

class WatchConnectorsResponse extends $pb.GeneratedMessage {
  factory WatchConnectorsResponse({
    Connector? connector,
    ConnectorRun? run,
    $core.bool? deleted,
  }) {
    final result = WatchConnectorsResponse._();
    if (connector != null) result.connector = connector;
    if (run != null) result.run = run;
    if (deleted != null) result.deleted = deleted;
    return result;
  }

  WatchConnectorsResponse._();

  factory WatchConnectorsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      WatchConnectorsResponse()..mergeFromBuffer(data, registry);
  factory WatchConnectorsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      WatchConnectorsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'WatchConnectorsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: WatchConnectorsResponse.$_createMessage)
    ..aOM<Connector>(1, _omitFieldNames ? '' : 'connector',
        subBuilder: Connector.$_createMessage)
    ..aOM<ConnectorRun>(2, _omitFieldNames ? '' : 'run',
        subBuilder: ConnectorRun.$_createMessage)
    ..aOB(3, _omitFieldNames ? '' : 'deleted')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchConnectorsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  WatchConnectorsResponse copyWith(
          void Function(WatchConnectorsResponse) updates) =>
      super.copyWith((message) => updates(message as WatchConnectorsResponse))
          as WatchConnectorsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use WatchConnectorsResponse() / WatchConnectorsResponse.new instead')
  static WatchConnectorsResponse create() => WatchConnectorsResponse._();
  static $pb.GeneratedMessage $_createMessage() => WatchConnectorsResponse._();
  @$core.override
  WatchConnectorsResponse createEmptyInstance() => WatchConnectorsResponse._();
  @$core.pragma('dart2js:noInline')
  static WatchConnectorsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<WatchConnectorsResponse>(
          WatchConnectorsResponse.$_createMessage);
  static WatchConnectorsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Connector get connector => $_getN(0);
  @$pb.TagNumber(1)
  set connector(Connector value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasConnector() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnector() => $_clearField(1);
  @$pb.TagNumber(1)
  Connector ensureConnector() => $_ensure(0);

  /// The latest run, if any; `outcome` empty while it pulls.
  @$pb.TagNumber(2)
  ConnectorRun get run => $_getN(1);
  @$pb.TagNumber(2)
  set run(ConnectorRun value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasRun() => $_has(1);
  @$pb.TagNumber(2)
  void clearRun() => $_clearField(2);
  @$pb.TagNumber(2)
  ConnectorRun ensureRun() => $_ensure(1);

  /// The connector was removed; only `connector.id` is meaningful.
  @$pb.TagNumber(3)
  $core.bool get deleted => $_getBF(2);
  @$pb.TagNumber(3)
  set deleted($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDeleted() => $_has(2);
  @$pb.TagNumber(3)
  void clearDeleted() => $_clearField(3);
}

class ListConnectorRunsRequest extends $pb.GeneratedMessage {
  factory ListConnectorRunsRequest({
    $core.String? id,
  }) {
    final result = ListConnectorRunsRequest._();
    if (id != null) result.id = id;
    return result;
  }

  ListConnectorRunsRequest._();

  factory ListConnectorRunsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorRunsRequest()..mergeFromBuffer(data, registry);
  factory ListConnectorRunsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorRunsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConnectorRunsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListConnectorRunsRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorRunsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorRunsRequest copyWith(
          void Function(ListConnectorRunsRequest) updates) =>
      super.copyWith((message) => updates(message as ListConnectorRunsRequest))
          as ListConnectorRunsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListConnectorRunsRequest() / ListConnectorRunsRequest.new instead')
  static ListConnectorRunsRequest create() => ListConnectorRunsRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListConnectorRunsRequest._();
  @$core.override
  ListConnectorRunsRequest createEmptyInstance() =>
      ListConnectorRunsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListConnectorRunsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConnectorRunsRequest>(
          ListConnectorRunsRequest.$_createMessage);
  static ListConnectorRunsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class ListConnectorRunsResponse extends $pb.GeneratedMessage {
  factory ListConnectorRunsResponse({
    $core.Iterable<ConnectorRun>? runs,
  }) {
    final result = ListConnectorRunsResponse._();
    if (runs != null) result.runs.addAll(runs);
    return result;
  }

  ListConnectorRunsResponse._();

  factory ListConnectorRunsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorRunsResponse()..mergeFromBuffer(data, registry);
  factory ListConnectorRunsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectorRunsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConnectorRunsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListConnectorRunsResponse.$_createMessage)
    ..pPM<ConnectorRun>(1, _omitFieldNames ? '' : 'runs',
        subBuilder: ConnectorRun.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorRunsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectorRunsResponse copyWith(
          void Function(ListConnectorRunsResponse) updates) =>
      super.copyWith((message) => updates(message as ListConnectorRunsResponse))
          as ListConnectorRunsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListConnectorRunsResponse() / ListConnectorRunsResponse.new instead')
  static ListConnectorRunsResponse create() => ListConnectorRunsResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      ListConnectorRunsResponse._();
  @$core.override
  ListConnectorRunsResponse createEmptyInstance() =>
      ListConnectorRunsResponse._();
  @$core.pragma('dart2js:noInline')
  static ListConnectorRunsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConnectorRunsResponse>(
          ListConnectorRunsResponse.$_createMessage);
  static ListConnectorRunsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ConnectorRun> get runs => $_getList(0);
}

class Document extends $pb.GeneratedMessage {
  factory Document({
    $core.String? id,
    $core.String? partyId,
    $core.String? kind,
    $core.String? filename,
    $core.String? contentType,
    $fixnum.Int64? sizeBytes,
    $core.String? sha256,
    $core.String? vendor,
    $core.String? docDate,
    $core.String? totalMinor,
    $core.String? currency,
    $core.String? createdAt,
    $core.Iterable<DocumentSource>? sources,
    $core.String? invoiceNo,
    $core.String? extractedAt,
    $core.bool? declared,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? foundBy,
  }) {
    final result = Document._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (kind != null) result.kind = kind;
    if (filename != null) result.filename = filename;
    if (contentType != null) result.contentType = contentType;
    if (sizeBytes != null) result.sizeBytes = sizeBytes;
    if (sha256 != null) result.sha256 = sha256;
    if (vendor != null) result.vendor = vendor;
    if (docDate != null) result.docDate = docDate;
    if (totalMinor != null) result.totalMinor = totalMinor;
    if (currency != null) result.currency = currency;
    if (createdAt != null) result.createdAt = createdAt;
    if (sources != null) result.sources.addAll(sources);
    if (invoiceNo != null) result.invoiceNo = invoiceNo;
    if (extractedAt != null) result.extractedAt = extractedAt;
    if (declared != null) result.declared = declared;
    if (foundBy != null) result.foundBy.addEntries(foundBy);
    return result;
  }

  Document._();

  factory Document.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Document()..mergeFromBuffer(data, registry);
  factory Document.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Document()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Document',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Document.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'kind')
    ..aOS(4, _omitFieldNames ? '' : 'filename')
    ..aOS(5, _omitFieldNames ? '' : 'contentType')
    ..aInt64(6, _omitFieldNames ? '' : 'sizeBytes')
    ..aOS(7, _omitFieldNames ? '' : 'sha256')
    ..aOS(8, _omitFieldNames ? '' : 'vendor')
    ..aOS(9, _omitFieldNames ? '' : 'docDate')
    ..aOS(10, _omitFieldNames ? '' : 'totalMinor')
    ..aOS(11, _omitFieldNames ? '' : 'currency')
    ..aOS(12, _omitFieldNames ? '' : 'createdAt')
    ..pPM<DocumentSource>(13, _omitFieldNames ? '' : 'sources',
        subBuilder: DocumentSource.$_createMessage)
    ..aOS(14, _omitFieldNames ? '' : 'invoiceNo')
    ..aOS(15, _omitFieldNames ? '' : 'extractedAt')
    ..aOB(16, _omitFieldNames ? '' : 'declared')
    ..m<$core.String, $core.String>(17, _omitFieldNames ? '' : 'foundBy',
        entryClassName: 'Document.FoundByEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('tbd.finance.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Document clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Document copyWith(void Function(Document) updates) =>
      super.copyWith((message) => updates(message as Document)) as Document;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Document() / Document.new instead')
  static Document create() => Document._();
  static $pb.GeneratedMessage $_createMessage() => Document._();
  @$core.override
  Document createEmptyInstance() => Document._();
  @$core.pragma('dart2js:noInline')
  static Document getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Document>(Document.$_createMessage);
  static Document? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  /// invoice or receipt.
  @$pb.TagNumber(3)
  $core.String get kind => $_getSZ(2);
  @$pb.TagNumber(3)
  set kind($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasKind() => $_has(2);
  @$pb.TagNumber(3)
  void clearKind() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get filename => $_getSZ(3);
  @$pb.TagNumber(4)
  set filename($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasFilename() => $_has(3);
  @$pb.TagNumber(4)
  void clearFilename() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get contentType => $_getSZ(4);
  @$pb.TagNumber(5)
  set contentType($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasContentType() => $_has(4);
  @$pb.TagNumber(5)
  void clearContentType() => $_clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get sizeBytes => $_getI64(5);
  @$pb.TagNumber(6)
  set sizeBytes($fixnum.Int64 value) => $_setInt64(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSizeBytes() => $_has(5);
  @$pb.TagNumber(6)
  void clearSizeBytes() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get sha256 => $_getSZ(6);
  @$pb.TagNumber(7)
  set sha256($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasSha256() => $_has(6);
  @$pb.TagNumber(7)
  void clearSha256() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get vendor => $_getSZ(7);
  @$pb.TagNumber(8)
  set vendor($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasVendor() => $_has(7);
  @$pb.TagNumber(8)
  void clearVendor() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get docDate => $_getSZ(8);
  @$pb.TagNumber(9)
  set docDate($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasDocDate() => $_has(8);
  @$pb.TagNumber(9)
  void clearDocDate() => $_clearField(9);

  /// Empty when unknown.
  @$pb.TagNumber(10)
  $core.String get totalMinor => $_getSZ(9);
  @$pb.TagNumber(10)
  set totalMinor($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasTotalMinor() => $_has(9);
  @$pb.TagNumber(10)
  void clearTotalMinor() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.String get currency => $_getSZ(10);
  @$pb.TagNumber(11)
  set currency($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasCurrency() => $_has(10);
  @$pb.TagNumber(11)
  void clearCurrency() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get createdAt => $_getSZ(11);
  @$pb.TagNumber(12)
  set createdAt($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasCreatedAt() => $_has(11);
  @$pb.TagNumber(12)
  void clearCreatedAt() => $_clearField(12);

  /// Where it came from.
  @$pb.TagNumber(13)
  $pb.PbList<DocumentSource> get sources => $_getList(12);

  @$pb.TagNumber(14)
  $core.String get invoiceNo => $_getSZ(13);
  @$pb.TagNumber(14)
  set invoiceNo($core.String value) => $_setString(13, value);
  @$pb.TagNumber(14)
  $core.bool hasInvoiceNo() => $_has(13);
  @$pb.TagNumber(14)
  void clearInvoiceNo() => $_clearField(14);

  /// When the reader last ran; empty if never.
  @$pb.TagNumber(15)
  $core.String get extractedAt => $_getSZ(14);
  @$pb.TagNumber(15)
  set extractedAt($core.String value) => $_setString(14, value);
  @$pb.TagNumber(15)
  $core.bool hasExtractedAt() => $_has(14);
  @$pb.TagNumber(15)
  void clearExtractedAt() => $_clearField(15);

  /// A person set the fields; a re-read leaves them.
  @$pb.TagNumber(16)
  $core.bool get declared => $_getBF(15);
  @$pb.TagNumber(16)
  set declared($core.bool value) => $_setBool(15, value);
  @$pb.TagNumber(16)
  $core.bool hasDeclared() => $_has(15);
  @$pb.TagNumber(16)
  void clearDeclared() => $_clearField(16);

  /// How each field was found: `vendor`, `date`, `amount`, `invoice_no` to
  /// `label`, `sender`, `first`, `received`, `declared` or empty; `party` to
  /// `declared`, `payment` (the account that paid it), `text` (the document
  /// names the party) or `mailbox` (the connector's, for want of better).
  @$pb.TagNumber(17)
  $pb.PbMap<$core.String, $core.String> get foundBy => $_getMap(16);
}

class DocumentSource extends $pb.GeneratedMessage {
  factory DocumentSource({
    $core.String? connectorId,
    $core.String? externalRef,
    $core.String? subject,
    $core.String? sender,
    $core.String? receivedAt,
  }) {
    final result = DocumentSource._();
    if (connectorId != null) result.connectorId = connectorId;
    if (externalRef != null) result.externalRef = externalRef;
    if (subject != null) result.subject = subject;
    if (sender != null) result.sender = sender;
    if (receivedAt != null) result.receivedAt = receivedAt;
    return result;
  }

  DocumentSource._();

  factory DocumentSource.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DocumentSource()..mergeFromBuffer(data, registry);
  factory DocumentSource.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DocumentSource()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DocumentSource',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DocumentSource.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'connectorId')
    ..aOS(2, _omitFieldNames ? '' : 'externalRef')
    ..aOS(3, _omitFieldNames ? '' : 'subject')
    ..aOS(4, _omitFieldNames ? '' : 'sender')
    ..aOS(5, _omitFieldNames ? '' : 'receivedAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DocumentSource clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DocumentSource copyWith(void Function(DocumentSource) updates) =>
      super.copyWith((message) => updates(message as DocumentSource))
          as DocumentSource;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use DocumentSource() / DocumentSource.new instead')
  static DocumentSource create() => DocumentSource._();
  static $pb.GeneratedMessage $_createMessage() => DocumentSource._();
  @$core.override
  DocumentSource createEmptyInstance() => DocumentSource._();
  @$core.pragma('dart2js:noInline')
  static DocumentSource getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<DocumentSource>(
          DocumentSource.$_createMessage);
  static DocumentSource? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get connectorId => $_getSZ(0);
  @$pb.TagNumber(1)
  set connectorId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasConnectorId() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnectorId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get externalRef => $_getSZ(1);
  @$pb.TagNumber(2)
  set externalRef($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasExternalRef() => $_has(1);
  @$pb.TagNumber(2)
  void clearExternalRef() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get subject => $_getSZ(2);
  @$pb.TagNumber(3)
  set subject($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSubject() => $_has(2);
  @$pb.TagNumber(3)
  void clearSubject() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get sender => $_getSZ(3);
  @$pb.TagNumber(4)
  set sender($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasSender() => $_has(3);
  @$pb.TagNumber(4)
  void clearSender() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get receivedAt => $_getSZ(4);
  @$pb.TagNumber(5)
  set receivedAt($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasReceivedAt() => $_has(4);
  @$pb.TagNumber(5)
  void clearReceivedAt() => $_clearField(5);
}

class ListDocumentsRequest extends $pb.GeneratedMessage {
  factory ListDocumentsRequest({
    $core.Iterable<$core.String>? partyIds,
    $core.String? kind,
    $core.int? limit,
    $core.int? offset,
    $core.String? q,
    $core.String? from,
    $core.String? to,
    $core.String? vendor,
  }) {
    final result = ListDocumentsRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    if (kind != null) result.kind = kind;
    if (limit != null) result.limit = limit;
    if (offset != null) result.offset = offset;
    if (q != null) result.q = q;
    if (from != null) result.from = from;
    if (to != null) result.to = to;
    if (vendor != null) result.vendor = vendor;
    return result;
  }

  ListDocumentsRequest._();

  factory ListDocumentsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListDocumentsRequest()..mergeFromBuffer(data, registry);
  factory ListDocumentsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListDocumentsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListDocumentsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListDocumentsRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..aOS(2, _omitFieldNames ? '' : 'kind')
    ..aI(3, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'offset', fieldType: $pb.PbFieldType.OU3)
    ..aOS(5, _omitFieldNames ? '' : 'q')
    ..aOS(6, _omitFieldNames ? '' : 'from')
    ..aOS(7, _omitFieldNames ? '' : 'to')
    ..aOS(8, _omitFieldNames ? '' : 'vendor')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDocumentsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDocumentsRequest copyWith(void Function(ListDocumentsRequest) updates) =>
      super.copyWith((message) => updates(message as ListDocumentsRequest))
          as ListDocumentsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListDocumentsRequest() / ListDocumentsRequest.new instead')
  static ListDocumentsRequest create() => ListDocumentsRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListDocumentsRequest._();
  @$core.override
  ListDocumentsRequest createEmptyInstance() => ListDocumentsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListDocumentsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListDocumentsRequest>(
          ListDocumentsRequest.$_createMessage);
  static ListDocumentsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);

  /// receipt, invoice, or empty for any.
  @$pb.TagNumber(2)
  $core.String get kind => $_getSZ(1);
  @$pb.TagNumber(2)
  set kind($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearKind() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get limit => $_getIZ(2);
  @$pb.TagNumber(3)
  set limit($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasLimit() => $_has(2);
  @$pb.TagNumber(3)
  void clearLimit() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get offset => $_getIZ(3);
  @$pb.TagNumber(4)
  set offset($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOffset() => $_has(3);
  @$pb.TagNumber(4)
  void clearOffset() => $_clearField(4);

  /// Matches vendor, file name, invoice number, sender, subject and the
  /// document's text, case-insensitively.
  @$pb.TagNumber(5)
  $core.String get q => $_getSZ(4);
  @$pb.TagNumber(5)
  set q($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasQ() => $_has(4);
  @$pb.TagNumber(5)
  void clearQ() => $_clearField(5);

  /// Inclusive bounds on the document date, YYYY-MM-DD; a document with no
  /// date falls back to when it was received.
  @$pb.TagNumber(6)
  $core.String get from => $_getSZ(5);
  @$pb.TagNumber(6)
  set from($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasFrom() => $_has(5);
  @$pb.TagNumber(6)
  void clearFrom() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get to => $_getSZ(6);
  @$pb.TagNumber(7)
  set to($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasTo() => $_has(6);
  @$pb.TagNumber(7)
  void clearTo() => $_clearField(7);

  /// Exact vendor, as listed.
  @$pb.TagNumber(8)
  $core.String get vendor => $_getSZ(7);
  @$pb.TagNumber(8)
  set vendor($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasVendor() => $_has(7);
  @$pb.TagNumber(8)
  void clearVendor() => $_clearField(8);
}

class ListDocumentsResponse extends $pb.GeneratedMessage {
  factory ListDocumentsResponse({
    $core.Iterable<Document>? documents,
    $core.int? total,
    $core.Iterable<VendorCount>? vendors,
  }) {
    final result = ListDocumentsResponse._();
    if (documents != null) result.documents.addAll(documents);
    if (total != null) result.total = total;
    if (vendors != null) result.vendors.addAll(vendors);
    return result;
  }

  ListDocumentsResponse._();

  factory ListDocumentsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListDocumentsResponse()..mergeFromBuffer(data, registry);
  factory ListDocumentsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListDocumentsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListDocumentsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListDocumentsResponse.$_createMessage)
    ..pPM<Document>(1, _omitFieldNames ? '' : 'documents',
        subBuilder: Document.$_createMessage)
    ..aI(2, _omitFieldNames ? '' : 'total', fieldType: $pb.PbFieldType.OU3)
    ..pPM<VendorCount>(3, _omitFieldNames ? '' : 'vendors',
        subBuilder: VendorCount.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDocumentsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListDocumentsResponse copyWith(
          void Function(ListDocumentsResponse) updates) =>
      super.copyWith((message) => updates(message as ListDocumentsResponse))
          as ListDocumentsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListDocumentsResponse() / ListDocumentsResponse.new instead')
  static ListDocumentsResponse create() => ListDocumentsResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListDocumentsResponse._();
  @$core.override
  ListDocumentsResponse createEmptyInstance() => ListDocumentsResponse._();
  @$core.pragma('dart2js:noInline')
  static ListDocumentsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListDocumentsResponse>(
          ListDocumentsResponse.$_createMessage);
  static ListDocumentsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Document> get documents => $_getList(0);

  /// Matches in all, before limit and offset.
  @$pb.TagNumber(2)
  $core.int get total => $_getIZ(1);
  @$pb.TagNumber(2)
  set total($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasTotal() => $_has(1);
  @$pb.TagNumber(2)
  void clearTotal() => $_clearField(2);

  /// Every vendor in view with how many documents it has, for a filter.
  @$pb.TagNumber(3)
  $pb.PbList<VendorCount> get vendors => $_getList(2);
}

class VendorCount extends $pb.GeneratedMessage {
  factory VendorCount({
    $core.String? vendor,
    $core.int? count,
  }) {
    final result = VendorCount._();
    if (vendor != null) result.vendor = vendor;
    if (count != null) result.count = count;
    return result;
  }

  VendorCount._();

  factory VendorCount.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VendorCount()..mergeFromBuffer(data, registry);
  factory VendorCount.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      VendorCount()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'VendorCount',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: VendorCount.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'vendor')
    ..aI(2, _omitFieldNames ? '' : 'count', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VendorCount clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  VendorCount copyWith(void Function(VendorCount) updates) =>
      super.copyWith((message) => updates(message as VendorCount))
          as VendorCount;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use VendorCount() / VendorCount.new instead')
  static VendorCount create() => VendorCount._();
  static $pb.GeneratedMessage $_createMessage() => VendorCount._();
  @$core.override
  VendorCount createEmptyInstance() => VendorCount._();
  @$core.pragma('dart2js:noInline')
  static VendorCount getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<VendorCount>(
          VendorCount.$_createMessage);
  static VendorCount? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get vendor => $_getSZ(0);
  @$pb.TagNumber(1)
  set vendor($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasVendor() => $_has(0);
  @$pb.TagNumber(1)
  void clearVendor() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get count => $_getIZ(1);
  @$pb.TagNumber(2)
  set count($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCount() => $_has(1);
  @$pb.TagNumber(2)
  void clearCount() => $_clearField(2);
}

class MonthlyReconciliationRequest extends $pb.GeneratedMessage {
  factory MonthlyReconciliationRequest({
    $core.String? partyId,
    $core.String? month,
  }) {
    final result = MonthlyReconciliationRequest._();
    if (partyId != null) result.partyId = partyId;
    if (month != null) result.month = month;
    return result;
  }

  MonthlyReconciliationRequest._();

  factory MonthlyReconciliationRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MonthlyReconciliationRequest()..mergeFromBuffer(data, registry);
  factory MonthlyReconciliationRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MonthlyReconciliationRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MonthlyReconciliationRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: MonthlyReconciliationRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'partyId')
    ..aOS(2, _omitFieldNames ? '' : 'month')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonthlyReconciliationRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonthlyReconciliationRequest copyWith(
          void Function(MonthlyReconciliationRequest) updates) =>
      super.copyWith(
              (message) => updates(message as MonthlyReconciliationRequest))
          as MonthlyReconciliationRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use MonthlyReconciliationRequest() / MonthlyReconciliationRequest.new instead')
  static MonthlyReconciliationRequest create() =>
      MonthlyReconciliationRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      MonthlyReconciliationRequest._();
  @$core.override
  MonthlyReconciliationRequest createEmptyInstance() =>
      MonthlyReconciliationRequest._();
  @$core.pragma('dart2js:noInline')
  static MonthlyReconciliationRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MonthlyReconciliationRequest>(
          MonthlyReconciliationRequest.$_createMessage);
  static MonthlyReconciliationRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get partyId => $_getSZ(0);
  @$pb.TagNumber(1)
  set partyId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPartyId() => $_has(0);
  @$pb.TagNumber(1)
  void clearPartyId() => $_clearField(1);

  /// YYYY-MM.
  @$pb.TagNumber(2)
  $core.String get month => $_getSZ(1);
  @$pb.TagNumber(2)
  set month($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMonth() => $_has(1);
  @$pb.TagNumber(2)
  void clearMonth() => $_clearField(2);
}

class MonthlyReconciliationResponse extends $pb.GeneratedMessage {
  factory MonthlyReconciliationResponse({
    $core.Iterable<ReconciliationRow>? rows,
    ReconciliationSummary? summary,
    $core.Iterable<CounterpartyPolicy>? policies,
  }) {
    final result = MonthlyReconciliationResponse._();
    if (rows != null) result.rows.addAll(rows);
    if (summary != null) result.summary = summary;
    if (policies != null) result.policies.addAll(policies);
    return result;
  }

  MonthlyReconciliationResponse._();

  factory MonthlyReconciliationResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MonthlyReconciliationResponse()..mergeFromBuffer(data, registry);
  factory MonthlyReconciliationResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MonthlyReconciliationResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MonthlyReconciliationResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: MonthlyReconciliationResponse.$_createMessage)
    ..pPM<ReconciliationRow>(1, _omitFieldNames ? '' : 'rows',
        subBuilder: ReconciliationRow.$_createMessage)
    ..aOM<ReconciliationSummary>(2, _omitFieldNames ? '' : 'summary',
        subBuilder: ReconciliationSummary.$_createMessage)
    ..pPM<CounterpartyPolicy>(3, _omitFieldNames ? '' : 'policies',
        subBuilder: CounterpartyPolicy.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonthlyReconciliationResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonthlyReconciliationResponse copyWith(
          void Function(MonthlyReconciliationResponse) updates) =>
      super.copyWith(
              (message) => updates(message as MonthlyReconciliationResponse))
          as MonthlyReconciliationResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use MonthlyReconciliationResponse() / MonthlyReconciliationResponse.new instead')
  static MonthlyReconciliationResponse create() =>
      MonthlyReconciliationResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      MonthlyReconciliationResponse._();
  @$core.override
  MonthlyReconciliationResponse createEmptyInstance() =>
      MonthlyReconciliationResponse._();
  @$core.pragma('dart2js:noInline')
  static MonthlyReconciliationResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MonthlyReconciliationResponse>(
          MonthlyReconciliationResponse.$_createMessage);
  static MonthlyReconciliationResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ReconciliationRow> get rows => $_getList(0);

  @$pb.TagNumber(2)
  ReconciliationSummary get summary => $_getN(1);
  @$pb.TagNumber(2)
  set summary(ReconciliationSummary value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasSummary() => $_has(1);
  @$pb.TagNumber(2)
  void clearSummary() => $_clearField(2);
  @$pb.TagNumber(2)
  ReconciliationSummary ensureSummary() => $_ensure(1);

  @$pb.TagNumber(3)
  $pb.PbList<CounterpartyPolicy> get policies => $_getList(2);
}

class ReconciliationRow extends $pb.GeneratedMessage {
  factory ReconciliationRow({
    Transaction? transaction,
    $core.String? need,
    $core.String? needReason,
    $core.String? status,
    $core.Iterable<LinkedDocument>? documents,
    $core.Iterable<LinkedDocument>? suggestions,
    $core.String? policyId,
    $core.String? originalAmountMinor,
    $core.String? originalCurrency,
    Reason? needWhy,
    $core.String? note,
  }) {
    final result = ReconciliationRow._();
    if (transaction != null) result.transaction = transaction;
    if (need != null) result.need = need;
    if (needReason != null) result.needReason = needReason;
    if (status != null) result.status = status;
    if (documents != null) result.documents.addAll(documents);
    if (suggestions != null) result.suggestions.addAll(suggestions);
    if (policyId != null) result.policyId = policyId;
    if (originalAmountMinor != null)
      result.originalAmountMinor = originalAmountMinor;
    if (originalCurrency != null) result.originalCurrency = originalCurrency;
    if (needWhy != null) result.needWhy = needWhy;
    if (note != null) result.note = note;
    return result;
  }

  ReconciliationRow._();

  factory ReconciliationRow.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ReconciliationRow()..mergeFromBuffer(data, registry);
  factory ReconciliationRow.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ReconciliationRow()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReconciliationRow',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ReconciliationRow.$_createMessage)
    ..aOM<Transaction>(1, _omitFieldNames ? '' : 'transaction',
        subBuilder: Transaction.$_createMessage)
    ..aOS(2, _omitFieldNames ? '' : 'need')
    ..aOS(3, _omitFieldNames ? '' : 'needReason')
    ..aOS(4, _omitFieldNames ? '' : 'status')
    ..pPM<LinkedDocument>(5, _omitFieldNames ? '' : 'documents',
        subBuilder: LinkedDocument.$_createMessage)
    ..pPM<LinkedDocument>(6, _omitFieldNames ? '' : 'suggestions',
        subBuilder: LinkedDocument.$_createMessage)
    ..aOS(7, _omitFieldNames ? '' : 'policyId')
    ..aOS(8, _omitFieldNames ? '' : 'originalAmountMinor')
    ..aOS(9, _omitFieldNames ? '' : 'originalCurrency')
    ..aOM<Reason>(10, _omitFieldNames ? '' : 'needWhy',
        subBuilder: Reason.$_createMessage)
    ..aOS(11, _omitFieldNames ? '' : 'note')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReconciliationRow clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReconciliationRow copyWith(void Function(ReconciliationRow) updates) =>
      super.copyWith((message) => updates(message as ReconciliationRow))
          as ReconciliationRow;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ReconciliationRow() / ReconciliationRow.new instead')
  static ReconciliationRow create() => ReconciliationRow._();
  static $pb.GeneratedMessage $_createMessage() => ReconciliationRow._();
  @$core.override
  ReconciliationRow createEmptyInstance() => ReconciliationRow._();
  @$core.pragma('dart2js:noInline')
  static ReconciliationRow getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ReconciliationRow>(
          ReconciliationRow.$_createMessage);
  static ReconciliationRow? _defaultInstance;

  @$pb.TagNumber(1)
  Transaction get transaction => $_getN(0);
  @$pb.TagNumber(1)
  set transaction(Transaction value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTransaction() => $_has(0);
  @$pb.TagNumber(1)
  void clearTransaction() => $_clearField(1);
  @$pb.TagNumber(1)
  Transaction ensureTransaction() => $_ensure(0);

  /// What the accountant needs from us: eracun (the supplier's e-invoice
  /// reaches them), receipt (we supply it), none (tax, salary, bank fee,
  /// cash: nothing exists), personal (not a business cost), income (our
  /// own invoice), internal (a transfer between own accounts).
  @$pb.TagNumber(2)
  $core.String get need => $_getSZ(1);
  @$pb.TagNumber(2)
  set need($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNeed() => $_has(1);
  @$pb.TagNumber(2)
  void clearNeed() => $_clearField(2);

  /// Why: "HR IBAN", "state budget (HR68)", "card, 75,00 USD", "policy".
  @$pb.TagNumber(3)
  $core.String get needReason => $_getSZ(2);
  @$pb.TagNumber(3)
  set needReason($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasNeedReason() => $_has(2);
  @$pb.TagNumber(3)
  void clearNeedReason() => $_clearField(3);

  /// For receipt: covered | missing. Otherwise empty.
  @$pb.TagNumber(4)
  $core.String get status => $_getSZ(3);
  @$pb.TagNumber(4)
  set status($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasStatus() => $_has(3);
  @$pb.TagNumber(4)
  void clearStatus() => $_clearField(4);

  /// Receipts linked to it.
  @$pb.TagNumber(5)
  $pb.PbList<LinkedDocument> get documents => $_getList(4);

  /// Likely receipts the matcher was not sure enough about, best first.
  @$pb.TagNumber(6)
  $pb.PbList<LinkedDocument> get suggestions => $_getList(5);

  /// The policy that decided `need`, if a person set one.
  @$pb.TagNumber(7)
  $core.String get policyId => $_getSZ(6);
  @$pb.TagNumber(7)
  set policyId($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasPolicyId() => $_has(6);
  @$pb.TagNumber(7)
  void clearPolicyId() => $_clearField(7);

  /// The original amount and currency the card was charged in, when the
  /// bank said so ("75,00 USD"); empty otherwise.
  @$pb.TagNumber(8)
  $core.String get originalAmountMinor => $_getSZ(7);
  @$pb.TagNumber(8)
  set originalAmountMinor($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasOriginalAmountMinor() => $_has(7);
  @$pb.TagNumber(8)
  void clearOriginalAmountMinor() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get originalCurrency => $_getSZ(8);
  @$pb.TagNumber(9)
  set originalCurrency($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasOriginalCurrency() => $_has(8);
  @$pb.TagNumber(9)
  void clearOriginalCurrency() => $_clearField(9);

  /// `need_reason` as a code with arguments, for a page in any language:
  /// internal, income, policy, state_budget, payout_person, cash, bank_fee,
  /// domestic_iban, card_original {amount_minor, currency}, card,
  /// foreign_transfer.
  @$pb.TagNumber(10)
  Reason get needWhy => $_getN(9);
  @$pb.TagNumber(10)
  set needWhy(Reason value) => $_setField(10, value);
  @$pb.TagNumber(10)
  $core.bool hasNeedWhy() => $_has(9);
  @$pb.TagNumber(10)
  void clearNeedWhy() => $_clearField(10);
  @$pb.TagNumber(10)
  Reason ensureNeedWhy() => $_ensure(9);

  /// A person's note for the accountant; goes into the month's README,
  /// summary and mail. Empty when none.
  @$pb.TagNumber(11)
  $core.String get note => $_getSZ(10);
  @$pb.TagNumber(11)
  set note($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasNote() => $_has(10);
  @$pb.TagNumber(11)
  void clearNote() => $_clearField(11);
}

/// A reason as a code with its arguments, so the page can say it in its own
/// language; the English sentence beside it is the same thing spelled out.
class Reason extends $pb.GeneratedMessage {
  factory Reason({
    $core.String? code,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? args,
  }) {
    final result = Reason._();
    if (code != null) result.code = code;
    if (args != null) result.args.addEntries(args);
    return result;
  }

  Reason._();

  factory Reason.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Reason()..mergeFromBuffer(data, registry);
  factory Reason.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Reason()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Reason',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Reason.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'code')
    ..m<$core.String, $core.String>(2, _omitFieldNames ? '' : 'args',
        entryClassName: 'Reason.ArgsEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('tbd.finance.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Reason clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Reason copyWith(void Function(Reason) updates) =>
      super.copyWith((message) => updates(message as Reason)) as Reason;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Reason() / Reason.new instead')
  static Reason create() => Reason._();
  static $pb.GeneratedMessage $_createMessage() => Reason._();
  @$core.override
  Reason createEmptyInstance() => Reason._();
  @$core.pragma('dart2js:noInline')
  static Reason getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Reason>(Reason.$_createMessage);
  static Reason? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get code => $_getSZ(0);
  @$pb.TagNumber(1)
  set code($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasCode() => $_has(0);
  @$pb.TagNumber(1)
  void clearCode() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbMap<$core.String, $core.String> get args => $_getMap(1);
}

class LinkedDocument extends $pb.GeneratedMessage {
  factory LinkedDocument({
    $core.String? documentId,
    $core.String? vendor,
    $core.String? docDate,
    $core.String? totalMinor,
    $core.String? currency,
    $core.String? filename,
    $core.String? invoiceNo,
    $core.String? source,
    $core.int? confidence,
    $core.String? reason,
    $core.Iterable<Reason>? why,
  }) {
    final result = LinkedDocument._();
    if (documentId != null) result.documentId = documentId;
    if (vendor != null) result.vendor = vendor;
    if (docDate != null) result.docDate = docDate;
    if (totalMinor != null) result.totalMinor = totalMinor;
    if (currency != null) result.currency = currency;
    if (filename != null) result.filename = filename;
    if (invoiceNo != null) result.invoiceNo = invoiceNo;
    if (source != null) result.source = source;
    if (confidence != null) result.confidence = confidence;
    if (reason != null) result.reason = reason;
    if (why != null) result.why.addAll(why);
    return result;
  }

  LinkedDocument._();

  factory LinkedDocument.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LinkedDocument()..mergeFromBuffer(data, registry);
  factory LinkedDocument.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LinkedDocument()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LinkedDocument',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: LinkedDocument.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'documentId')
    ..aOS(2, _omitFieldNames ? '' : 'vendor')
    ..aOS(3, _omitFieldNames ? '' : 'docDate')
    ..aOS(4, _omitFieldNames ? '' : 'totalMinor')
    ..aOS(5, _omitFieldNames ? '' : 'currency')
    ..aOS(6, _omitFieldNames ? '' : 'filename')
    ..aOS(7, _omitFieldNames ? '' : 'invoiceNo')
    ..aOS(8, _omitFieldNames ? '' : 'source')
    ..aI(9, _omitFieldNames ? '' : 'confidence', fieldType: $pb.PbFieldType.OU3)
    ..aOS(10, _omitFieldNames ? '' : 'reason')
    ..pPM<Reason>(11, _omitFieldNames ? '' : 'why',
        subBuilder: Reason.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LinkedDocument clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LinkedDocument copyWith(void Function(LinkedDocument) updates) =>
      super.copyWith((message) => updates(message as LinkedDocument))
          as LinkedDocument;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use LinkedDocument() / LinkedDocument.new instead')
  static LinkedDocument create() => LinkedDocument._();
  static $pb.GeneratedMessage $_createMessage() => LinkedDocument._();
  @$core.override
  LinkedDocument createEmptyInstance() => LinkedDocument._();
  @$core.pragma('dart2js:noInline')
  static LinkedDocument getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<LinkedDocument>(
          LinkedDocument.$_createMessage);
  static LinkedDocument? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get documentId => $_getSZ(0);
  @$pb.TagNumber(1)
  set documentId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasDocumentId() => $_has(0);
  @$pb.TagNumber(1)
  void clearDocumentId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get vendor => $_getSZ(1);
  @$pb.TagNumber(2)
  set vendor($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasVendor() => $_has(1);
  @$pb.TagNumber(2)
  void clearVendor() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get docDate => $_getSZ(2);
  @$pb.TagNumber(3)
  set docDate($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDocDate() => $_has(2);
  @$pb.TagNumber(3)
  void clearDocDate() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get totalMinor => $_getSZ(3);
  @$pb.TagNumber(4)
  set totalMinor($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTotalMinor() => $_has(3);
  @$pb.TagNumber(4)
  void clearTotalMinor() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get currency => $_getSZ(4);
  @$pb.TagNumber(5)
  set currency($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCurrency() => $_has(4);
  @$pb.TagNumber(5)
  void clearCurrency() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get filename => $_getSZ(5);
  @$pb.TagNumber(6)
  set filename($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasFilename() => $_has(5);
  @$pb.TagNumber(6)
  void clearFilename() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get invoiceNo => $_getSZ(6);
  @$pb.TagNumber(7)
  set invoiceNo($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasInvoiceNo() => $_has(6);
  @$pb.TagNumber(7)
  void clearInvoiceNo() => $_clearField(7);

  /// declared or inferred; empty for a suggestion.
  @$pb.TagNumber(8)
  $core.String get source => $_getSZ(7);
  @$pb.TagNumber(8)
  set source($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasSource() => $_has(7);
  @$pb.TagNumber(8)
  void clearSource() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.int get confidence => $_getIZ(8);
  @$pb.TagNumber(9)
  set confidence($core.int value) => $_setUnsignedInt32(8, value);
  @$pb.TagNumber(9)
  $core.bool hasConfidence() => $_has(8);
  @$pb.TagNumber(9)
  void clearConfidence() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get reason => $_getSZ(9);
  @$pb.TagNumber(10)
  set reason($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasReason() => $_has(9);
  @$pb.TagNumber(10)
  void clearReason() => $_clearField(10);

  /// `reason` as codes: amount_original / amount / amount_fx
  /// {amount_minor, currency}, vendor, same_days, days_apart {days}, by_hand.
  @$pb.TagNumber(11)
  $pb.PbList<Reason> get why => $_getList(10);
}

class ReconciliationSummary extends $pb.GeneratedMessage {
  factory ReconciliationSummary({
    $core.int? transactions,
    $core.int? eracun,
    $core.int? receiptCovered,
    $core.int? receiptMissing,
    $core.int? none,
    $core.int? personal,
    $core.int? income,
    $core.int? internal,
    $core.Iterable<$core.MapEntry<$core.String, $core.String>>? missingMinor,
  }) {
    final result = ReconciliationSummary._();
    if (transactions != null) result.transactions = transactions;
    if (eracun != null) result.eracun = eracun;
    if (receiptCovered != null) result.receiptCovered = receiptCovered;
    if (receiptMissing != null) result.receiptMissing = receiptMissing;
    if (none != null) result.none = none;
    if (personal != null) result.personal = personal;
    if (income != null) result.income = income;
    if (internal != null) result.internal = internal;
    if (missingMinor != null) result.missingMinor.addEntries(missingMinor);
    return result;
  }

  ReconciliationSummary._();

  factory ReconciliationSummary.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ReconciliationSummary()..mergeFromBuffer(data, registry);
  factory ReconciliationSummary.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ReconciliationSummary()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ReconciliationSummary',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ReconciliationSummary.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'transactions',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(2, _omitFieldNames ? '' : 'eracun', fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'receiptCovered',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'receiptMissing',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(5, _omitFieldNames ? '' : 'none', fieldType: $pb.PbFieldType.OU3)
    ..aI(6, _omitFieldNames ? '' : 'personal', fieldType: $pb.PbFieldType.OU3)
    ..aI(7, _omitFieldNames ? '' : 'income', fieldType: $pb.PbFieldType.OU3)
    ..aI(8, _omitFieldNames ? '' : 'internal', fieldType: $pb.PbFieldType.OU3)
    ..m<$core.String, $core.String>(9, _omitFieldNames ? '' : 'missingMinor',
        entryClassName: 'ReconciliationSummary.MissingMinorEntry',
        keyFieldType: $pb.PbFieldType.OS,
        valueFieldType: $pb.PbFieldType.OS,
        packageName: const $pb.PackageName('tbd.finance.v1'))
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReconciliationSummary clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ReconciliationSummary copyWith(
          void Function(ReconciliationSummary) updates) =>
      super.copyWith((message) => updates(message as ReconciliationSummary))
          as ReconciliationSummary;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ReconciliationSummary() / ReconciliationSummary.new instead')
  static ReconciliationSummary create() => ReconciliationSummary._();
  static $pb.GeneratedMessage $_createMessage() => ReconciliationSummary._();
  @$core.override
  ReconciliationSummary createEmptyInstance() => ReconciliationSummary._();
  @$core.pragma('dart2js:noInline')
  static ReconciliationSummary getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ReconciliationSummary>(
          ReconciliationSummary.$_createMessage);
  static ReconciliationSummary? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get transactions => $_getIZ(0);
  @$pb.TagNumber(1)
  set transactions($core.int value) => $_setUnsignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTransactions() => $_has(0);
  @$pb.TagNumber(1)
  void clearTransactions() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.int get eracun => $_getIZ(1);
  @$pb.TagNumber(2)
  set eracun($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEracun() => $_has(1);
  @$pb.TagNumber(2)
  void clearEracun() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get receiptCovered => $_getIZ(2);
  @$pb.TagNumber(3)
  set receiptCovered($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasReceiptCovered() => $_has(2);
  @$pb.TagNumber(3)
  void clearReceiptCovered() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get receiptMissing => $_getIZ(3);
  @$pb.TagNumber(4)
  set receiptMissing($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasReceiptMissing() => $_has(3);
  @$pb.TagNumber(4)
  void clearReceiptMissing() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get none => $_getIZ(4);
  @$pb.TagNumber(5)
  set none($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasNone() => $_has(4);
  @$pb.TagNumber(5)
  void clearNone() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get personal => $_getIZ(5);
  @$pb.TagNumber(6)
  set personal($core.int value) => $_setUnsignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasPersonal() => $_has(5);
  @$pb.TagNumber(6)
  void clearPersonal() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.int get income => $_getIZ(6);
  @$pb.TagNumber(7)
  set income($core.int value) => $_setUnsignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasIncome() => $_has(6);
  @$pb.TagNumber(7)
  void clearIncome() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.int get internal => $_getIZ(7);
  @$pb.TagNumber(8)
  set internal($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasInternal() => $_has(7);
  @$pb.TagNumber(8)
  void clearInternal() => $_clearField(8);

  /// Sum of amounts still missing a receipt, minor units, by currency.
  @$pb.TagNumber(9)
  $pb.PbMap<$core.String, $core.String> get missingMinor => $_getMap(8);
}

class CounterpartyPolicy extends $pb.GeneratedMessage {
  factory CounterpartyPolicy({
    $core.String? id,
    $core.String? partyId,
    $core.String? match,
    $core.bool? exact,
    $core.String? policy,
    $core.String? note,
  }) {
    final result = CounterpartyPolicy._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (match != null) result.match = match;
    if (exact != null) result.exact = exact;
    if (policy != null) result.policy = policy;
    if (note != null) result.note = note;
    return result;
  }

  CounterpartyPolicy._();

  factory CounterpartyPolicy.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CounterpartyPolicy()..mergeFromBuffer(data, registry);
  factory CounterpartyPolicy.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CounterpartyPolicy()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CounterpartyPolicy',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: CounterpartyPolicy.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'match')
    ..aOB(4, _omitFieldNames ? '' : 'exact')
    ..aOS(5, _omitFieldNames ? '' : 'policy')
    ..aOS(6, _omitFieldNames ? '' : 'note')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CounterpartyPolicy clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CounterpartyPolicy copyWith(void Function(CounterpartyPolicy) updates) =>
      super.copyWith((message) => updates(message as CounterpartyPolicy))
          as CounterpartyPolicy;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use CounterpartyPolicy() / CounterpartyPolicy.new instead')
  static CounterpartyPolicy create() => CounterpartyPolicy._();
  static $pb.GeneratedMessage $_createMessage() => CounterpartyPolicy._();
  @$core.override
  CounterpartyPolicy createEmptyInstance() => CounterpartyPolicy._();
  @$core.pragma('dart2js:noInline')
  static CounterpartyPolicy getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CounterpartyPolicy>(
          CounterpartyPolicy.$_createMessage);
  static CounterpartyPolicy? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get match => $_getSZ(2);
  @$pb.TagNumber(3)
  set match($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMatch() => $_has(2);
  @$pb.TagNumber(3)
  void clearMatch() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.bool get exact => $_getBF(3);
  @$pb.TagNumber(4)
  set exact($core.bool value) => $_setBool(3, value);
  @$pb.TagNumber(4)
  $core.bool hasExact() => $_has(3);
  @$pb.TagNumber(4)
  void clearExact() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get policy => $_getSZ(4);
  @$pb.TagNumber(5)
  set policy($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasPolicy() => $_has(4);
  @$pb.TagNumber(5)
  void clearPolicy() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get note => $_getSZ(5);
  @$pb.TagNumber(6)
  set note($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasNote() => $_has(5);
  @$pb.TagNumber(6)
  void clearNote() => $_clearField(6);
}

class LinkDocumentRequest extends $pb.GeneratedMessage {
  factory LinkDocumentRequest({
    $core.String? transactionId,
    $core.String? documentId,
    $core.bool? force,
  }) {
    final result = LinkDocumentRequest._();
    if (transactionId != null) result.transactionId = transactionId;
    if (documentId != null) result.documentId = documentId;
    if (force != null) result.force = force;
    return result;
  }

  LinkDocumentRequest._();

  factory LinkDocumentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LinkDocumentRequest()..mergeFromBuffer(data, registry);
  factory LinkDocumentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LinkDocumentRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LinkDocumentRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: LinkDocumentRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'transactionId')
    ..aOS(2, _omitFieldNames ? '' : 'documentId')
    ..aOB(3, _omitFieldNames ? '' : 'force')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LinkDocumentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LinkDocumentRequest copyWith(void Function(LinkDocumentRequest) updates) =>
      super.copyWith((message) => updates(message as LinkDocumentRequest))
          as LinkDocumentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use LinkDocumentRequest() / LinkDocumentRequest.new instead')
  static LinkDocumentRequest create() => LinkDocumentRequest._();
  static $pb.GeneratedMessage $_createMessage() => LinkDocumentRequest._();
  @$core.override
  LinkDocumentRequest createEmptyInstance() => LinkDocumentRequest._();
  @$core.pragma('dart2js:noInline')
  static LinkDocumentRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<LinkDocumentRequest>(
          LinkDocumentRequest.$_createMessage);
  static LinkDocumentRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get transactionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set transactionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTransactionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTransactionId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get documentId => $_getSZ(1);
  @$pb.TagNumber(2)
  set documentId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDocumentId() => $_has(1);
  @$pb.TagNumber(2)
  void clearDocumentId() => $_clearField(2);

  /// A receipt whose amount was read and disagrees with the charge is
  /// refused (FAILED_PRECONDITION, saying what was read) unless this is set.
  @$pb.TagNumber(3)
  $core.bool get force => $_getBF(2);
  @$pb.TagNumber(3)
  set force($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasForce() => $_has(2);
  @$pb.TagNumber(3)
  void clearForce() => $_clearField(3);
}

class LinkDocumentResponse extends $pb.GeneratedMessage {
  factory LinkDocumentResponse({
    ReconciliationRow? row,
  }) {
    final result = LinkDocumentResponse._();
    if (row != null) result.row = row;
    return result;
  }

  LinkDocumentResponse._();

  factory LinkDocumentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LinkDocumentResponse()..mergeFromBuffer(data, registry);
  factory LinkDocumentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LinkDocumentResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LinkDocumentResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: LinkDocumentResponse.$_createMessage)
    ..aOM<ReconciliationRow>(1, _omitFieldNames ? '' : 'row',
        subBuilder: ReconciliationRow.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LinkDocumentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LinkDocumentResponse copyWith(void Function(LinkDocumentResponse) updates) =>
      super.copyWith((message) => updates(message as LinkDocumentResponse))
          as LinkDocumentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use LinkDocumentResponse() / LinkDocumentResponse.new instead')
  static LinkDocumentResponse create() => LinkDocumentResponse._();
  static $pb.GeneratedMessage $_createMessage() => LinkDocumentResponse._();
  @$core.override
  LinkDocumentResponse createEmptyInstance() => LinkDocumentResponse._();
  @$core.pragma('dart2js:noInline')
  static LinkDocumentResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<LinkDocumentResponse>(
          LinkDocumentResponse.$_createMessage);
  static LinkDocumentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ReconciliationRow get row => $_getN(0);
  @$pb.TagNumber(1)
  set row(ReconciliationRow value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRow() => $_has(0);
  @$pb.TagNumber(1)
  void clearRow() => $_clearField(1);
  @$pb.TagNumber(1)
  ReconciliationRow ensureRow() => $_ensure(0);
}

class UnlinkDocumentRequest extends $pb.GeneratedMessage {
  factory UnlinkDocumentRequest({
    $core.String? transactionId,
    $core.String? documentId,
  }) {
    final result = UnlinkDocumentRequest._();
    if (transactionId != null) result.transactionId = transactionId;
    if (documentId != null) result.documentId = documentId;
    return result;
  }

  UnlinkDocumentRequest._();

  factory UnlinkDocumentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UnlinkDocumentRequest()..mergeFromBuffer(data, registry);
  factory UnlinkDocumentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UnlinkDocumentRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UnlinkDocumentRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UnlinkDocumentRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'transactionId')
    ..aOS(2, _omitFieldNames ? '' : 'documentId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnlinkDocumentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnlinkDocumentRequest copyWith(
          void Function(UnlinkDocumentRequest) updates) =>
      super.copyWith((message) => updates(message as UnlinkDocumentRequest))
          as UnlinkDocumentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UnlinkDocumentRequest() / UnlinkDocumentRequest.new instead')
  static UnlinkDocumentRequest create() => UnlinkDocumentRequest._();
  static $pb.GeneratedMessage $_createMessage() => UnlinkDocumentRequest._();
  @$core.override
  UnlinkDocumentRequest createEmptyInstance() => UnlinkDocumentRequest._();
  @$core.pragma('dart2js:noInline')
  static UnlinkDocumentRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UnlinkDocumentRequest>(
          UnlinkDocumentRequest.$_createMessage);
  static UnlinkDocumentRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get transactionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set transactionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTransactionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTransactionId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get documentId => $_getSZ(1);
  @$pb.TagNumber(2)
  set documentId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDocumentId() => $_has(1);
  @$pb.TagNumber(2)
  void clearDocumentId() => $_clearField(2);
}

class UnlinkDocumentResponse extends $pb.GeneratedMessage {
  factory UnlinkDocumentResponse({
    ReconciliationRow? row,
  }) {
    final result = UnlinkDocumentResponse._();
    if (row != null) result.row = row;
    return result;
  }

  UnlinkDocumentResponse._();

  factory UnlinkDocumentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UnlinkDocumentResponse()..mergeFromBuffer(data, registry);
  factory UnlinkDocumentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UnlinkDocumentResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UnlinkDocumentResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UnlinkDocumentResponse.$_createMessage)
    ..aOM<ReconciliationRow>(1, _omitFieldNames ? '' : 'row',
        subBuilder: ReconciliationRow.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnlinkDocumentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UnlinkDocumentResponse copyWith(
          void Function(UnlinkDocumentResponse) updates) =>
      super.copyWith((message) => updates(message as UnlinkDocumentResponse))
          as UnlinkDocumentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UnlinkDocumentResponse() / UnlinkDocumentResponse.new instead')
  static UnlinkDocumentResponse create() => UnlinkDocumentResponse._();
  static $pb.GeneratedMessage $_createMessage() => UnlinkDocumentResponse._();
  @$core.override
  UnlinkDocumentResponse createEmptyInstance() => UnlinkDocumentResponse._();
  @$core.pragma('dart2js:noInline')
  static UnlinkDocumentResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UnlinkDocumentResponse>(
          UnlinkDocumentResponse.$_createMessage);
  static UnlinkDocumentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ReconciliationRow get row => $_getN(0);
  @$pb.TagNumber(1)
  set row(ReconciliationRow value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRow() => $_has(0);
  @$pb.TagNumber(1)
  void clearRow() => $_clearField(1);
  @$pb.TagNumber(1)
  ReconciliationRow ensureRow() => $_ensure(0);
}

class SetTransactionNoteRequest extends $pb.GeneratedMessage {
  factory SetTransactionNoteRequest({
    $core.String? transactionId,
    $core.String? note,
  }) {
    final result = SetTransactionNoteRequest._();
    if (transactionId != null) result.transactionId = transactionId;
    if (note != null) result.note = note;
    return result;
  }

  SetTransactionNoteRequest._();

  factory SetTransactionNoteRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetTransactionNoteRequest()..mergeFromBuffer(data, registry);
  factory SetTransactionNoteRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetTransactionNoteRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SetTransactionNoteRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SetTransactionNoteRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'transactionId')
    ..aOS(2, _omitFieldNames ? '' : 'note')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetTransactionNoteRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetTransactionNoteRequest copyWith(
          void Function(SetTransactionNoteRequest) updates) =>
      super.copyWith((message) => updates(message as SetTransactionNoteRequest))
          as SetTransactionNoteRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use SetTransactionNoteRequest() / SetTransactionNoteRequest.new instead')
  static SetTransactionNoteRequest create() => SetTransactionNoteRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      SetTransactionNoteRequest._();
  @$core.override
  SetTransactionNoteRequest createEmptyInstance() =>
      SetTransactionNoteRequest._();
  @$core.pragma('dart2js:noInline')
  static SetTransactionNoteRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetTransactionNoteRequest>(
          SetTransactionNoteRequest.$_createMessage);
  static SetTransactionNoteRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get transactionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set transactionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTransactionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTransactionId() => $_clearField(1);

  /// At most 2,000 characters; empty removes the note.
  @$pb.TagNumber(2)
  $core.String get note => $_getSZ(1);
  @$pb.TagNumber(2)
  set note($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNote() => $_has(1);
  @$pb.TagNumber(2)
  void clearNote() => $_clearField(2);
}

class SetTransactionNoteResponse extends $pb.GeneratedMessage {
  factory SetTransactionNoteResponse({
    ReconciliationRow? row,
  }) {
    final result = SetTransactionNoteResponse._();
    if (row != null) result.row = row;
    return result;
  }

  SetTransactionNoteResponse._();

  factory SetTransactionNoteResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetTransactionNoteResponse()..mergeFromBuffer(data, registry);
  factory SetTransactionNoteResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetTransactionNoteResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SetTransactionNoteResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SetTransactionNoteResponse.$_createMessage)
    ..aOM<ReconciliationRow>(1, _omitFieldNames ? '' : 'row',
        subBuilder: ReconciliationRow.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetTransactionNoteResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetTransactionNoteResponse copyWith(
          void Function(SetTransactionNoteResponse) updates) =>
      super.copyWith(
              (message) => updates(message as SetTransactionNoteResponse))
          as SetTransactionNoteResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use SetTransactionNoteResponse() / SetTransactionNoteResponse.new instead')
  static SetTransactionNoteResponse create() => SetTransactionNoteResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      SetTransactionNoteResponse._();
  @$core.override
  SetTransactionNoteResponse createEmptyInstance() =>
      SetTransactionNoteResponse._();
  @$core.pragma('dart2js:noInline')
  static SetTransactionNoteResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetTransactionNoteResponse>(
          SetTransactionNoteResponse.$_createMessage);
  static SetTransactionNoteResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ReconciliationRow get row => $_getN(0);
  @$pb.TagNumber(1)
  set row(ReconciliationRow value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRow() => $_has(0);
  @$pb.TagNumber(1)
  void clearRow() => $_clearField(1);
  @$pb.TagNumber(1)
  ReconciliationRow ensureRow() => $_ensure(0);
}

class SetCounterpartyPolicyRequest extends $pb.GeneratedMessage {
  factory SetCounterpartyPolicyRequest({
    $core.String? partyId,
    $core.String? match,
    $core.bool? exact,
    $core.String? policy,
    $core.String? note,
  }) {
    final result = SetCounterpartyPolicyRequest._();
    if (partyId != null) result.partyId = partyId;
    if (match != null) result.match = match;
    if (exact != null) result.exact = exact;
    if (policy != null) result.policy = policy;
    if (note != null) result.note = note;
    return result;
  }

  SetCounterpartyPolicyRequest._();

  factory SetCounterpartyPolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetCounterpartyPolicyRequest()..mergeFromBuffer(data, registry);
  factory SetCounterpartyPolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetCounterpartyPolicyRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SetCounterpartyPolicyRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SetCounterpartyPolicyRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'partyId')
    ..aOS(2, _omitFieldNames ? '' : 'match')
    ..aOB(3, _omitFieldNames ? '' : 'exact')
    ..aOS(4, _omitFieldNames ? '' : 'policy')
    ..aOS(5, _omitFieldNames ? '' : 'note')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetCounterpartyPolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetCounterpartyPolicyRequest copyWith(
          void Function(SetCounterpartyPolicyRequest) updates) =>
      super.copyWith(
              (message) => updates(message as SetCounterpartyPolicyRequest))
          as SetCounterpartyPolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use SetCounterpartyPolicyRequest() / SetCounterpartyPolicyRequest.new instead')
  static SetCounterpartyPolicyRequest create() =>
      SetCounterpartyPolicyRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      SetCounterpartyPolicyRequest._();
  @$core.override
  SetCounterpartyPolicyRequest createEmptyInstance() =>
      SetCounterpartyPolicyRequest._();
  @$core.pragma('dart2js:noInline')
  static SetCounterpartyPolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetCounterpartyPolicyRequest>(
          SetCounterpartyPolicyRequest.$_createMessage);
  static SetCounterpartyPolicyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get partyId => $_getSZ(0);
  @$pb.TagNumber(1)
  set partyId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPartyId() => $_has(0);
  @$pb.TagNumber(1)
  void clearPartyId() => $_clearField(1);

  /// The counterparty name as the bank gives it, or a fragment of it.
  @$pb.TagNumber(2)
  $core.String get match => $_getSZ(1);
  @$pb.TagNumber(2)
  set match($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasMatch() => $_has(1);
  @$pb.TagNumber(2)
  void clearMatch() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.bool get exact => $_getBF(2);
  @$pb.TagNumber(3)
  set exact($core.bool value) => $_setBool(2, value);
  @$pb.TagNumber(3)
  $core.bool hasExact() => $_has(2);
  @$pb.TagNumber(3)
  void clearExact() => $_clearField(3);

  /// eracun, receipt, none, personal.
  @$pb.TagNumber(4)
  $core.String get policy => $_getSZ(3);
  @$pb.TagNumber(4)
  set policy($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPolicy() => $_has(3);
  @$pb.TagNumber(4)
  void clearPolicy() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get note => $_getSZ(4);
  @$pb.TagNumber(5)
  set note($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasNote() => $_has(4);
  @$pb.TagNumber(5)
  void clearNote() => $_clearField(5);
}

class SetCounterpartyPolicyResponse extends $pb.GeneratedMessage {
  factory SetCounterpartyPolicyResponse({
    CounterpartyPolicy? policy,
  }) {
    final result = SetCounterpartyPolicyResponse._();
    if (policy != null) result.policy = policy;
    return result;
  }

  SetCounterpartyPolicyResponse._();

  factory SetCounterpartyPolicyResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetCounterpartyPolicyResponse()..mergeFromBuffer(data, registry);
  factory SetCounterpartyPolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetCounterpartyPolicyResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SetCounterpartyPolicyResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SetCounterpartyPolicyResponse.$_createMessage)
    ..aOM<CounterpartyPolicy>(1, _omitFieldNames ? '' : 'policy',
        subBuilder: CounterpartyPolicy.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetCounterpartyPolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetCounterpartyPolicyResponse copyWith(
          void Function(SetCounterpartyPolicyResponse) updates) =>
      super.copyWith(
              (message) => updates(message as SetCounterpartyPolicyResponse))
          as SetCounterpartyPolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use SetCounterpartyPolicyResponse() / SetCounterpartyPolicyResponse.new instead')
  static SetCounterpartyPolicyResponse create() =>
      SetCounterpartyPolicyResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      SetCounterpartyPolicyResponse._();
  @$core.override
  SetCounterpartyPolicyResponse createEmptyInstance() =>
      SetCounterpartyPolicyResponse._();
  @$core.pragma('dart2js:noInline')
  static SetCounterpartyPolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetCounterpartyPolicyResponse>(
          SetCounterpartyPolicyResponse.$_createMessage);
  static SetCounterpartyPolicyResponse? _defaultInstance;

  @$pb.TagNumber(1)
  CounterpartyPolicy get policy => $_getN(0);
  @$pb.TagNumber(1)
  set policy(CounterpartyPolicy value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasPolicy() => $_has(0);
  @$pb.TagNumber(1)
  void clearPolicy() => $_clearField(1);
  @$pb.TagNumber(1)
  CounterpartyPolicy ensurePolicy() => $_ensure(0);
}

class DeleteCounterpartyPolicyRequest extends $pb.GeneratedMessage {
  factory DeleteCounterpartyPolicyRequest({
    $core.String? id,
  }) {
    final result = DeleteCounterpartyPolicyRequest._();
    if (id != null) result.id = id;
    return result;
  }

  DeleteCounterpartyPolicyRequest._();

  factory DeleteCounterpartyPolicyRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteCounterpartyPolicyRequest()..mergeFromBuffer(data, registry);
  factory DeleteCounterpartyPolicyRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteCounterpartyPolicyRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteCounterpartyPolicyRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeleteCounterpartyPolicyRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCounterpartyPolicyRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCounterpartyPolicyRequest copyWith(
          void Function(DeleteCounterpartyPolicyRequest) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteCounterpartyPolicyRequest))
          as DeleteCounterpartyPolicyRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeleteCounterpartyPolicyRequest() / DeleteCounterpartyPolicyRequest.new instead')
  static DeleteCounterpartyPolicyRequest create() =>
      DeleteCounterpartyPolicyRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      DeleteCounterpartyPolicyRequest._();
  @$core.override
  DeleteCounterpartyPolicyRequest createEmptyInstance() =>
      DeleteCounterpartyPolicyRequest._();
  @$core.pragma('dart2js:noInline')
  static DeleteCounterpartyPolicyRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteCounterpartyPolicyRequest>(
          DeleteCounterpartyPolicyRequest.$_createMessage);
  static DeleteCounterpartyPolicyRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class DeleteCounterpartyPolicyResponse extends $pb.GeneratedMessage {
  factory DeleteCounterpartyPolicyResponse() =>
      DeleteCounterpartyPolicyResponse._();

  DeleteCounterpartyPolicyResponse._();

  factory DeleteCounterpartyPolicyResponse.fromBuffer(
          $core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteCounterpartyPolicyResponse()..mergeFromBuffer(data, registry);
  factory DeleteCounterpartyPolicyResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteCounterpartyPolicyResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteCounterpartyPolicyResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeleteCounterpartyPolicyResponse.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCounterpartyPolicyResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteCounterpartyPolicyResponse copyWith(
          void Function(DeleteCounterpartyPolicyResponse) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteCounterpartyPolicyResponse))
          as DeleteCounterpartyPolicyResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeleteCounterpartyPolicyResponse() / DeleteCounterpartyPolicyResponse.new instead')
  static DeleteCounterpartyPolicyResponse create() =>
      DeleteCounterpartyPolicyResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      DeleteCounterpartyPolicyResponse._();
  @$core.override
  DeleteCounterpartyPolicyResponse createEmptyInstance() =>
      DeleteCounterpartyPolicyResponse._();
  @$core.pragma('dart2js:noInline')
  static DeleteCounterpartyPolicyResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteCounterpartyPolicyResponse>(
          DeleteCounterpartyPolicyResponse.$_createMessage);
  static DeleteCounterpartyPolicyResponse? _defaultInstance;
}

class UpdateDocumentRequest extends $pb.GeneratedMessage {
  factory UpdateDocumentRequest({
    $core.String? id,
    $core.String? vendor,
    $core.String? docDate,
    $core.String? totalMinor,
    $core.String? currency,
    $core.String? invoiceNo,
    $core.String? partyId,
  }) {
    final result = UpdateDocumentRequest._();
    if (id != null) result.id = id;
    if (vendor != null) result.vendor = vendor;
    if (docDate != null) result.docDate = docDate;
    if (totalMinor != null) result.totalMinor = totalMinor;
    if (currency != null) result.currency = currency;
    if (invoiceNo != null) result.invoiceNo = invoiceNo;
    if (partyId != null) result.partyId = partyId;
    return result;
  }

  UpdateDocumentRequest._();

  factory UpdateDocumentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpdateDocumentRequest()..mergeFromBuffer(data, registry);
  factory UpdateDocumentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpdateDocumentRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateDocumentRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpdateDocumentRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'vendor')
    ..aOS(3, _omitFieldNames ? '' : 'docDate')
    ..aOS(4, _omitFieldNames ? '' : 'totalMinor')
    ..aOS(5, _omitFieldNames ? '' : 'currency')
    ..aOS(6, _omitFieldNames ? '' : 'invoiceNo')
    ..aOS(7, _omitFieldNames ? '' : 'partyId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateDocumentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateDocumentRequest copyWith(
          void Function(UpdateDocumentRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateDocumentRequest))
          as UpdateDocumentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpdateDocumentRequest() / UpdateDocumentRequest.new instead')
  static UpdateDocumentRequest create() => UpdateDocumentRequest._();
  static $pb.GeneratedMessage $_createMessage() => UpdateDocumentRequest._();
  @$core.override
  UpdateDocumentRequest createEmptyInstance() => UpdateDocumentRequest._();
  @$core.pragma('dart2js:noInline')
  static UpdateDocumentRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateDocumentRequest>(
          UpdateDocumentRequest.$_createMessage);
  static UpdateDocumentRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get vendor => $_getSZ(1);
  @$pb.TagNumber(2)
  set vendor($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasVendor() => $_has(1);
  @$pb.TagNumber(2)
  void clearVendor() => $_clearField(2);

  /// YYYY-MM-DD, or empty to clear.
  @$pb.TagNumber(3)
  $core.String get docDate => $_getSZ(2);
  @$pb.TagNumber(3)
  set docDate($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDocDate() => $_has(2);
  @$pb.TagNumber(3)
  void clearDocDate() => $_clearField(3);

  /// Minor units as a decimal string, or empty to clear.
  @$pb.TagNumber(4)
  $core.String get totalMinor => $_getSZ(3);
  @$pb.TagNumber(4)
  set totalMinor($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasTotalMinor() => $_has(3);
  @$pb.TagNumber(4)
  void clearTotalMinor() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get currency => $_getSZ(4);
  @$pb.TagNumber(5)
  set currency($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCurrency() => $_has(4);
  @$pb.TagNumber(5)
  void clearCurrency() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get invoiceNo => $_getSZ(5);
  @$pb.TagNumber(6)
  set invoiceNo($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasInvoiceNo() => $_has(5);
  @$pb.TagNumber(6)
  void clearInvoiceNo() => $_clearField(6);

  /// Whose it is: a party the caller may read, or empty to leave it. Once
  /// set by a person, no re-read decides it again.
  @$pb.TagNumber(7)
  $core.String get partyId => $_getSZ(6);
  @$pb.TagNumber(7)
  set partyId($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasPartyId() => $_has(6);
  @$pb.TagNumber(7)
  void clearPartyId() => $_clearField(7);
}

class UpdateDocumentResponse extends $pb.GeneratedMessage {
  factory UpdateDocumentResponse({
    Document? document,
  }) {
    final result = UpdateDocumentResponse._();
    if (document != null) result.document = document;
    return result;
  }

  UpdateDocumentResponse._();

  factory UpdateDocumentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpdateDocumentResponse()..mergeFromBuffer(data, registry);
  factory UpdateDocumentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpdateDocumentResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateDocumentResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpdateDocumentResponse.$_createMessage)
    ..aOM<Document>(1, _omitFieldNames ? '' : 'document',
        subBuilder: Document.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateDocumentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateDocumentResponse copyWith(
          void Function(UpdateDocumentResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateDocumentResponse))
          as UpdateDocumentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpdateDocumentResponse() / UpdateDocumentResponse.new instead')
  static UpdateDocumentResponse create() => UpdateDocumentResponse._();
  static $pb.GeneratedMessage $_createMessage() => UpdateDocumentResponse._();
  @$core.override
  UpdateDocumentResponse createEmptyInstance() => UpdateDocumentResponse._();
  @$core.pragma('dart2js:noInline')
  static UpdateDocumentResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateDocumentResponse>(
          UpdateDocumentResponse.$_createMessage);
  static UpdateDocumentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Document get document => $_getN(0);
  @$pb.TagNumber(1)
  set document(Document value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasDocument() => $_has(0);
  @$pb.TagNumber(1)
  void clearDocument() => $_clearField(1);
  @$pb.TagNumber(1)
  Document ensureDocument() => $_ensure(0);
}

class ExtractDocumentRequest extends $pb.GeneratedMessage {
  factory ExtractDocumentRequest({
    $core.String? id,
  }) {
    final result = ExtractDocumentRequest._();
    if (id != null) result.id = id;
    return result;
  }

  ExtractDocumentRequest._();

  factory ExtractDocumentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ExtractDocumentRequest()..mergeFromBuffer(data, registry);
  factory ExtractDocumentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ExtractDocumentRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExtractDocumentRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ExtractDocumentRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExtractDocumentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExtractDocumentRequest copyWith(
          void Function(ExtractDocumentRequest) updates) =>
      super.copyWith((message) => updates(message as ExtractDocumentRequest))
          as ExtractDocumentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ExtractDocumentRequest() / ExtractDocumentRequest.new instead')
  static ExtractDocumentRequest create() => ExtractDocumentRequest._();
  static $pb.GeneratedMessage $_createMessage() => ExtractDocumentRequest._();
  @$core.override
  ExtractDocumentRequest createEmptyInstance() => ExtractDocumentRequest._();
  @$core.pragma('dart2js:noInline')
  static ExtractDocumentRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExtractDocumentRequest>(
          ExtractDocumentRequest.$_createMessage);
  static ExtractDocumentRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class ExtractDocumentResponse extends $pb.GeneratedMessage {
  factory ExtractDocumentResponse({
    Document? document,
  }) {
    final result = ExtractDocumentResponse._();
    if (document != null) result.document = document;
    return result;
  }

  ExtractDocumentResponse._();

  factory ExtractDocumentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ExtractDocumentResponse()..mergeFromBuffer(data, registry);
  factory ExtractDocumentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ExtractDocumentResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ExtractDocumentResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ExtractDocumentResponse.$_createMessage)
    ..aOM<Document>(1, _omitFieldNames ? '' : 'document',
        subBuilder: Document.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExtractDocumentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ExtractDocumentResponse copyWith(
          void Function(ExtractDocumentResponse) updates) =>
      super.copyWith((message) => updates(message as ExtractDocumentResponse))
          as ExtractDocumentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ExtractDocumentResponse() / ExtractDocumentResponse.new instead')
  static ExtractDocumentResponse create() => ExtractDocumentResponse._();
  static $pb.GeneratedMessage $_createMessage() => ExtractDocumentResponse._();
  @$core.override
  ExtractDocumentResponse createEmptyInstance() => ExtractDocumentResponse._();
  @$core.pragma('dart2js:noInline')
  static ExtractDocumentResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ExtractDocumentResponse>(
          ExtractDocumentResponse.$_createMessage);
  static ExtractDocumentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Document get document => $_getN(0);
  @$pb.TagNumber(1)
  set document(Document value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasDocument() => $_has(0);
  @$pb.TagNumber(1)
  void clearDocument() => $_clearField(1);
  @$pb.TagNumber(1)
  Document ensureDocument() => $_ensure(0);
}

class GetDocumentRequest extends $pb.GeneratedMessage {
  factory GetDocumentRequest({
    $core.String? id,
  }) {
    final result = GetDocumentRequest._();
    if (id != null) result.id = id;
    return result;
  }

  GetDocumentRequest._();

  factory GetDocumentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetDocumentRequest()..mergeFromBuffer(data, registry);
  factory GetDocumentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetDocumentRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetDocumentRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetDocumentRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDocumentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDocumentRequest copyWith(void Function(GetDocumentRequest) updates) =>
      super.copyWith((message) => updates(message as GetDocumentRequest))
          as GetDocumentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetDocumentRequest() / GetDocumentRequest.new instead')
  static GetDocumentRequest create() => GetDocumentRequest._();
  static $pb.GeneratedMessage $_createMessage() => GetDocumentRequest._();
  @$core.override
  GetDocumentRequest createEmptyInstance() => GetDocumentRequest._();
  @$core.pragma('dart2js:noInline')
  static GetDocumentRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetDocumentRequest>(
          GetDocumentRequest.$_createMessage);
  static GetDocumentRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetDocumentResponse extends $pb.GeneratedMessage {
  factory GetDocumentResponse({
    Document? document,
    $core.List<$core.int>? bytes,
  }) {
    final result = GetDocumentResponse._();
    if (document != null) result.document = document;
    if (bytes != null) result.bytes = bytes;
    return result;
  }

  GetDocumentResponse._();

  factory GetDocumentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetDocumentResponse()..mergeFromBuffer(data, registry);
  factory GetDocumentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetDocumentResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetDocumentResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetDocumentResponse.$_createMessage)
    ..aOM<Document>(1, _omitFieldNames ? '' : 'document',
        subBuilder: Document.$_createMessage)
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'bytes', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDocumentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetDocumentResponse copyWith(void Function(GetDocumentResponse) updates) =>
      super.copyWith((message) => updates(message as GetDocumentResponse))
          as GetDocumentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use GetDocumentResponse() / GetDocumentResponse.new instead')
  static GetDocumentResponse create() => GetDocumentResponse._();
  static $pb.GeneratedMessage $_createMessage() => GetDocumentResponse._();
  @$core.override
  GetDocumentResponse createEmptyInstance() => GetDocumentResponse._();
  @$core.pragma('dart2js:noInline')
  static GetDocumentResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetDocumentResponse>(
          GetDocumentResponse.$_createMessage);
  static GetDocumentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Document get document => $_getN(0);
  @$pb.TagNumber(1)
  set document(Document value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasDocument() => $_has(0);
  @$pb.TagNumber(1)
  void clearDocument() => $_clearField(1);
  @$pb.TagNumber(1)
  Document ensureDocument() => $_ensure(0);

  @$pb.TagNumber(2)
  $core.List<$core.int> get bytes => $_getN(1);
  @$pb.TagNumber(2)
  set bytes($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasBytes() => $_has(1);
  @$pb.TagNumber(2)
  void clearBytes() => $_clearField(2);
}

class UploadDocumentRequest extends $pb.GeneratedMessage {
  factory UploadDocumentRequest({
    $core.String? partyId,
    $core.String? filename,
    $core.String? contentType,
    $core.List<$core.int>? bytes,
  }) {
    final result = UploadDocumentRequest._();
    if (partyId != null) result.partyId = partyId;
    if (filename != null) result.filename = filename;
    if (contentType != null) result.contentType = contentType;
    if (bytes != null) result.bytes = bytes;
    return result;
  }

  UploadDocumentRequest._();

  factory UploadDocumentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UploadDocumentRequest()..mergeFromBuffer(data, registry);
  factory UploadDocumentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UploadDocumentRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UploadDocumentRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UploadDocumentRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'partyId')
    ..aOS(2, _omitFieldNames ? '' : 'filename')
    ..aOS(3, _omitFieldNames ? '' : 'contentType')
    ..a<$core.List<$core.int>>(
        4, _omitFieldNames ? '' : 'bytes', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UploadDocumentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UploadDocumentRequest copyWith(
          void Function(UploadDocumentRequest) updates) =>
      super.copyWith((message) => updates(message as UploadDocumentRequest))
          as UploadDocumentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UploadDocumentRequest() / UploadDocumentRequest.new instead')
  static UploadDocumentRequest create() => UploadDocumentRequest._();
  static $pb.GeneratedMessage $_createMessage() => UploadDocumentRequest._();
  @$core.override
  UploadDocumentRequest createEmptyInstance() => UploadDocumentRequest._();
  @$core.pragma('dart2js:noInline')
  static UploadDocumentRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UploadDocumentRequest>(
          UploadDocumentRequest.$_createMessage);
  static UploadDocumentRequest? _defaultInstance;

  /// A party the caller may read.
  @$pb.TagNumber(1)
  $core.String get partyId => $_getSZ(0);
  @$pb.TagNumber(1)
  set partyId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPartyId() => $_has(0);
  @$pb.TagNumber(1)
  void clearPartyId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get filename => $_getSZ(1);
  @$pb.TagNumber(2)
  set filename($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasFilename() => $_has(1);
  @$pb.TagNumber(2)
  void clearFilename() => $_clearField(2);

  /// application/pdf, image/jpeg or image/png.
  @$pb.TagNumber(3)
  $core.String get contentType => $_getSZ(2);
  @$pb.TagNumber(3)
  set contentType($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasContentType() => $_has(2);
  @$pb.TagNumber(3)
  void clearContentType() => $_clearField(3);

  /// The file. The gateway takes a body of at most 2 MiB, base64 included.
  @$pb.TagNumber(4)
  $core.List<$core.int> get bytes => $_getN(3);
  @$pb.TagNumber(4)
  set bytes($core.List<$core.int> value) => $_setBytes(3, value);
  @$pb.TagNumber(4)
  $core.bool hasBytes() => $_has(3);
  @$pb.TagNumber(4)
  void clearBytes() => $_clearField(4);
}

class UploadDocumentResponse extends $pb.GeneratedMessage {
  factory UploadDocumentResponse({
    Document? document,
  }) {
    final result = UploadDocumentResponse._();
    if (document != null) result.document = document;
    return result;
  }

  UploadDocumentResponse._();

  factory UploadDocumentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UploadDocumentResponse()..mergeFromBuffer(data, registry);
  factory UploadDocumentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UploadDocumentResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UploadDocumentResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UploadDocumentResponse.$_createMessage)
    ..aOM<Document>(1, _omitFieldNames ? '' : 'document',
        subBuilder: Document.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UploadDocumentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UploadDocumentResponse copyWith(
          void Function(UploadDocumentResponse) updates) =>
      super.copyWith((message) => updates(message as UploadDocumentResponse))
          as UploadDocumentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UploadDocumentResponse() / UploadDocumentResponse.new instead')
  static UploadDocumentResponse create() => UploadDocumentResponse._();
  static $pb.GeneratedMessage $_createMessage() => UploadDocumentResponse._();
  @$core.override
  UploadDocumentResponse createEmptyInstance() => UploadDocumentResponse._();
  @$core.pragma('dart2js:noInline')
  static UploadDocumentResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UploadDocumentResponse>(
          UploadDocumentResponse.$_createMessage);
  static UploadDocumentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Document get document => $_getN(0);
  @$pb.TagNumber(1)
  set document(Document value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasDocument() => $_has(0);
  @$pb.TagNumber(1)
  void clearDocument() => $_clearField(1);
  @$pb.TagNumber(1)
  Document ensureDocument() => $_ensure(0);
}

class LineTemplate extends $pb.GeneratedMessage {
  factory LineTemplate({
    $core.String? id,
    $core.String? clientId,
    $core.int? position,
    $core.String? description,
    $core.String? mode,
    $fixnum.Int64? quantityMilli,
    $fixnum.Int64? unitPriceMinor,
    $core.bool? enabled,
  }) {
    final result = LineTemplate._();
    if (id != null) result.id = id;
    if (clientId != null) result.clientId = clientId;
    if (position != null) result.position = position;
    if (description != null) result.description = description;
    if (mode != null) result.mode = mode;
    if (quantityMilli != null) result.quantityMilli = quantityMilli;
    if (unitPriceMinor != null) result.unitPriceMinor = unitPriceMinor;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  LineTemplate._();

  factory LineTemplate.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LineTemplate()..mergeFromBuffer(data, registry);
  factory LineTemplate.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      LineTemplate()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'LineTemplate',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: LineTemplate.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'clientId')
    ..aI(3, _omitFieldNames ? '' : 'position')
    ..aOS(4, _omitFieldNames ? '' : 'description')
    ..aOS(5, _omitFieldNames ? '' : 'mode')
    ..aInt64(6, _omitFieldNames ? '' : 'quantityMilli')
    ..aInt64(7, _omitFieldNames ? '' : 'unitPriceMinor')
    ..aOB(8, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LineTemplate clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  LineTemplate copyWith(void Function(LineTemplate) updates) =>
      super.copyWith((message) => updates(message as LineTemplate))
          as LineTemplate;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use LineTemplate() / LineTemplate.new instead')
  static LineTemplate create() => LineTemplate._();
  static $pb.GeneratedMessage $_createMessage() => LineTemplate._();
  @$core.override
  LineTemplate createEmptyInstance() => LineTemplate._();
  @$core.pragma('dart2js:noInline')
  static LineTemplate getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<LineTemplate>(
          LineTemplate.$_createMessage);
  static LineTemplate? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get clientId => $_getSZ(1);
  @$pb.TagNumber(2)
  set clientId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasClientId() => $_has(1);
  @$pb.TagNumber(2)
  void clearClientId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get position => $_getIZ(2);
  @$pb.TagNumber(3)
  set position($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPosition() => $_has(2);
  @$pb.TagNumber(3)
  void clearPosition() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get description => $_getSZ(3);
  @$pb.TagNumber(4)
  set description($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasDescription() => $_has(3);
  @$pb.TagNumber(4)
  void clearDescription() => $_clearField(4);

  /// fixed: always on a draft with this price. variable: always on a draft,
  /// price pre-filled from the last invoice that carried it. optional:
  /// offered in the editor, off until chosen.
  @$pb.TagNumber(5)
  $core.String get mode => $_getSZ(4);
  @$pb.TagNumber(5)
  set mode($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasMode() => $_has(4);
  @$pb.TagNumber(5)
  void clearMode() => $_clearField(5);

  @$pb.TagNumber(6)
  $fixnum.Int64 get quantityMilli => $_getI64(5);
  @$pb.TagNumber(6)
  set quantityMilli($fixnum.Int64 value) => $_setInt64(5, value);
  @$pb.TagNumber(6)
  $core.bool hasQuantityMilli() => $_has(5);
  @$pb.TagNumber(6)
  void clearQuantityMilli() => $_clearField(6);

  @$pb.TagNumber(7)
  $fixnum.Int64 get unitPriceMinor => $_getI64(6);
  @$pb.TagNumber(7)
  set unitPriceMinor($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasUnitPriceMinor() => $_has(6);
  @$pb.TagNumber(7)
  void clearUnitPriceMinor() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.bool get enabled => $_getBF(7);
  @$pb.TagNumber(8)
  set enabled($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasEnabled() => $_has(7);
  @$pb.TagNumber(8)
  void clearEnabled() => $_clearField(8);
}

class ListLineTemplatesRequest extends $pb.GeneratedMessage {
  factory ListLineTemplatesRequest({
    $core.String? clientId,
  }) {
    final result = ListLineTemplatesRequest._();
    if (clientId != null) result.clientId = clientId;
    return result;
  }

  ListLineTemplatesRequest._();

  factory ListLineTemplatesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListLineTemplatesRequest()..mergeFromBuffer(data, registry);
  factory ListLineTemplatesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListLineTemplatesRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListLineTemplatesRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListLineTemplatesRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'clientId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListLineTemplatesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListLineTemplatesRequest copyWith(
          void Function(ListLineTemplatesRequest) updates) =>
      super.copyWith((message) => updates(message as ListLineTemplatesRequest))
          as ListLineTemplatesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListLineTemplatesRequest() / ListLineTemplatesRequest.new instead')
  static ListLineTemplatesRequest create() => ListLineTemplatesRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListLineTemplatesRequest._();
  @$core.override
  ListLineTemplatesRequest createEmptyInstance() =>
      ListLineTemplatesRequest._();
  @$core.pragma('dart2js:noInline')
  static ListLineTemplatesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListLineTemplatesRequest>(
          ListLineTemplatesRequest.$_createMessage);
  static ListLineTemplatesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get clientId => $_getSZ(0);
  @$pb.TagNumber(1)
  set clientId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasClientId() => $_has(0);
  @$pb.TagNumber(1)
  void clearClientId() => $_clearField(1);
}

class ListLineTemplatesResponse extends $pb.GeneratedMessage {
  factory ListLineTemplatesResponse({
    $core.Iterable<LineTemplate>? templates,
  }) {
    final result = ListLineTemplatesResponse._();
    if (templates != null) result.templates.addAll(templates);
    return result;
  }

  ListLineTemplatesResponse._();

  factory ListLineTemplatesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListLineTemplatesResponse()..mergeFromBuffer(data, registry);
  factory ListLineTemplatesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListLineTemplatesResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListLineTemplatesResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListLineTemplatesResponse.$_createMessage)
    ..pPM<LineTemplate>(1, _omitFieldNames ? '' : 'templates',
        subBuilder: LineTemplate.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListLineTemplatesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListLineTemplatesResponse copyWith(
          void Function(ListLineTemplatesResponse) updates) =>
      super.copyWith((message) => updates(message as ListLineTemplatesResponse))
          as ListLineTemplatesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListLineTemplatesResponse() / ListLineTemplatesResponse.new instead')
  static ListLineTemplatesResponse create() => ListLineTemplatesResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      ListLineTemplatesResponse._();
  @$core.override
  ListLineTemplatesResponse createEmptyInstance() =>
      ListLineTemplatesResponse._();
  @$core.pragma('dart2js:noInline')
  static ListLineTemplatesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListLineTemplatesResponse>(
          ListLineTemplatesResponse.$_createMessage);
  static ListLineTemplatesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<LineTemplate> get templates => $_getList(0);
}

class UpsertLineTemplateRequest extends $pb.GeneratedMessage {
  factory UpsertLineTemplateRequest({
    $core.String? clientId,
    LineTemplate? template,
  }) {
    final result = UpsertLineTemplateRequest._();
    if (clientId != null) result.clientId = clientId;
    if (template != null) result.template = template;
    return result;
  }

  UpsertLineTemplateRequest._();

  factory UpsertLineTemplateRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertLineTemplateRequest()..mergeFromBuffer(data, registry);
  factory UpsertLineTemplateRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertLineTemplateRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertLineTemplateRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertLineTemplateRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'clientId')
    ..aOM<LineTemplate>(2, _omitFieldNames ? '' : 'template',
        subBuilder: LineTemplate.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertLineTemplateRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertLineTemplateRequest copyWith(
          void Function(UpsertLineTemplateRequest) updates) =>
      super.copyWith((message) => updates(message as UpsertLineTemplateRequest))
          as UpsertLineTemplateRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpsertLineTemplateRequest() / UpsertLineTemplateRequest.new instead')
  static UpsertLineTemplateRequest create() => UpsertLineTemplateRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      UpsertLineTemplateRequest._();
  @$core.override
  UpsertLineTemplateRequest createEmptyInstance() =>
      UpsertLineTemplateRequest._();
  @$core.pragma('dart2js:noInline')
  static UpsertLineTemplateRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertLineTemplateRequest>(
          UpsertLineTemplateRequest.$_createMessage);
  static UpsertLineTemplateRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get clientId => $_getSZ(0);
  @$pb.TagNumber(1)
  set clientId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasClientId() => $_has(0);
  @$pb.TagNumber(1)
  void clearClientId() => $_clearField(1);

  /// Empty id creates.
  @$pb.TagNumber(2)
  LineTemplate get template => $_getN(1);
  @$pb.TagNumber(2)
  set template(LineTemplate value) => $_setField(2, value);
  @$pb.TagNumber(2)
  $core.bool hasTemplate() => $_has(1);
  @$pb.TagNumber(2)
  void clearTemplate() => $_clearField(2);
  @$pb.TagNumber(2)
  LineTemplate ensureTemplate() => $_ensure(1);
}

class UpsertLineTemplateResponse extends $pb.GeneratedMessage {
  factory UpsertLineTemplateResponse({
    LineTemplate? template,
  }) {
    final result = UpsertLineTemplateResponse._();
    if (template != null) result.template = template;
    return result;
  }

  UpsertLineTemplateResponse._();

  factory UpsertLineTemplateResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertLineTemplateResponse()..mergeFromBuffer(data, registry);
  factory UpsertLineTemplateResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertLineTemplateResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertLineTemplateResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertLineTemplateResponse.$_createMessage)
    ..aOM<LineTemplate>(1, _omitFieldNames ? '' : 'template',
        subBuilder: LineTemplate.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertLineTemplateResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertLineTemplateResponse copyWith(
          void Function(UpsertLineTemplateResponse) updates) =>
      super.copyWith(
              (message) => updates(message as UpsertLineTemplateResponse))
          as UpsertLineTemplateResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpsertLineTemplateResponse() / UpsertLineTemplateResponse.new instead')
  static UpsertLineTemplateResponse create() => UpsertLineTemplateResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      UpsertLineTemplateResponse._();
  @$core.override
  UpsertLineTemplateResponse createEmptyInstance() =>
      UpsertLineTemplateResponse._();
  @$core.pragma('dart2js:noInline')
  static UpsertLineTemplateResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertLineTemplateResponse>(
          UpsertLineTemplateResponse.$_createMessage);
  static UpsertLineTemplateResponse? _defaultInstance;

  @$pb.TagNumber(1)
  LineTemplate get template => $_getN(0);
  @$pb.TagNumber(1)
  set template(LineTemplate value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTemplate() => $_has(0);
  @$pb.TagNumber(1)
  void clearTemplate() => $_clearField(1);
  @$pb.TagNumber(1)
  LineTemplate ensureTemplate() => $_ensure(0);
}

class DeleteLineTemplateRequest extends $pb.GeneratedMessage {
  factory DeleteLineTemplateRequest({
    $core.String? clientId,
    $core.String? id,
  }) {
    final result = DeleteLineTemplateRequest._();
    if (clientId != null) result.clientId = clientId;
    if (id != null) result.id = id;
    return result;
  }

  DeleteLineTemplateRequest._();

  factory DeleteLineTemplateRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteLineTemplateRequest()..mergeFromBuffer(data, registry);
  factory DeleteLineTemplateRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteLineTemplateRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteLineTemplateRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeleteLineTemplateRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'clientId')
    ..aOS(2, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteLineTemplateRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteLineTemplateRequest copyWith(
          void Function(DeleteLineTemplateRequest) updates) =>
      super.copyWith((message) => updates(message as DeleteLineTemplateRequest))
          as DeleteLineTemplateRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeleteLineTemplateRequest() / DeleteLineTemplateRequest.new instead')
  static DeleteLineTemplateRequest create() => DeleteLineTemplateRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      DeleteLineTemplateRequest._();
  @$core.override
  DeleteLineTemplateRequest createEmptyInstance() =>
      DeleteLineTemplateRequest._();
  @$core.pragma('dart2js:noInline')
  static DeleteLineTemplateRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteLineTemplateRequest>(
          DeleteLineTemplateRequest.$_createMessage);
  static DeleteLineTemplateRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get clientId => $_getSZ(0);
  @$pb.TagNumber(1)
  set clientId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasClientId() => $_has(0);
  @$pb.TagNumber(1)
  void clearClientId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get id => $_getSZ(1);
  @$pb.TagNumber(2)
  set id($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasId() => $_has(1);
  @$pb.TagNumber(2)
  void clearId() => $_clearField(2);
}

class DeleteLineTemplateResponse extends $pb.GeneratedMessage {
  factory DeleteLineTemplateResponse() => DeleteLineTemplateResponse._();

  DeleteLineTemplateResponse._();

  factory DeleteLineTemplateResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteLineTemplateResponse()..mergeFromBuffer(data, registry);
  factory DeleteLineTemplateResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeleteLineTemplateResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeleteLineTemplateResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeleteLineTemplateResponse.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteLineTemplateResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeleteLineTemplateResponse copyWith(
          void Function(DeleteLineTemplateResponse) updates) =>
      super.copyWith(
              (message) => updates(message as DeleteLineTemplateResponse))
          as DeleteLineTemplateResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeleteLineTemplateResponse() / DeleteLineTemplateResponse.new instead')
  static DeleteLineTemplateResponse create() => DeleteLineTemplateResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      DeleteLineTemplateResponse._();
  @$core.override
  DeleteLineTemplateResponse createEmptyInstance() =>
      DeleteLineTemplateResponse._();
  @$core.pragma('dart2js:noInline')
  static DeleteLineTemplateResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeleteLineTemplateResponse>(
          DeleteLineTemplateResponse.$_createMessage);
  static DeleteLineTemplateResponse? _defaultInstance;
}

class IssuerProfile extends $pb.GeneratedMessage {
  factory IssuerProfile({
    $core.String? partyId,
    $core.String? legalName,
    $core.Iterable<$core.String>? addressLines,
    $core.String? oib,
    $core.String? vatId,
    $core.String? iban,
    $core.String? swift,
    $core.String? bankName,
    $core.String? court,
    $core.String? registrationNo,
    $core.String? shareCapital,
    $core.String? boardMember,
    $core.String? issuedBy,
    $core.String? placeOfIssue,
    $core.String? operatorId,
    $core.String? premises,
    $core.String? device,
    $core.int? dueDays,
  }) {
    final result = IssuerProfile._();
    if (partyId != null) result.partyId = partyId;
    if (legalName != null) result.legalName = legalName;
    if (addressLines != null) result.addressLines.addAll(addressLines);
    if (oib != null) result.oib = oib;
    if (vatId != null) result.vatId = vatId;
    if (iban != null) result.iban = iban;
    if (swift != null) result.swift = swift;
    if (bankName != null) result.bankName = bankName;
    if (court != null) result.court = court;
    if (registrationNo != null) result.registrationNo = registrationNo;
    if (shareCapital != null) result.shareCapital = shareCapital;
    if (boardMember != null) result.boardMember = boardMember;
    if (issuedBy != null) result.issuedBy = issuedBy;
    if (placeOfIssue != null) result.placeOfIssue = placeOfIssue;
    if (operatorId != null) result.operatorId = operatorId;
    if (premises != null) result.premises = premises;
    if (device != null) result.device = device;
    if (dueDays != null) result.dueDays = dueDays;
    return result;
  }

  IssuerProfile._();

  factory IssuerProfile.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      IssuerProfile()..mergeFromBuffer(data, registry);
  factory IssuerProfile.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      IssuerProfile()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'IssuerProfile',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: IssuerProfile.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'partyId')
    ..aOS(2, _omitFieldNames ? '' : 'legalName')
    ..pPS(3, _omitFieldNames ? '' : 'addressLines')
    ..aOS(4, _omitFieldNames ? '' : 'oib')
    ..aOS(5, _omitFieldNames ? '' : 'vatId')
    ..aOS(6, _omitFieldNames ? '' : 'iban')
    ..aOS(7, _omitFieldNames ? '' : 'swift')
    ..aOS(8, _omitFieldNames ? '' : 'bankName')
    ..aOS(9, _omitFieldNames ? '' : 'court')
    ..aOS(10, _omitFieldNames ? '' : 'registrationNo')
    ..aOS(11, _omitFieldNames ? '' : 'shareCapital')
    ..aOS(12, _omitFieldNames ? '' : 'boardMember')
    ..aOS(13, _omitFieldNames ? '' : 'issuedBy')
    ..aOS(14, _omitFieldNames ? '' : 'placeOfIssue')
    ..aOS(15, _omitFieldNames ? '' : 'operatorId')
    ..aOS(16, _omitFieldNames ? '' : 'premises')
    ..aOS(17, _omitFieldNames ? '' : 'device')
    ..aI(18, _omitFieldNames ? '' : 'dueDays')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  IssuerProfile clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  IssuerProfile copyWith(void Function(IssuerProfile) updates) =>
      super.copyWith((message) => updates(message as IssuerProfile))
          as IssuerProfile;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use IssuerProfile() / IssuerProfile.new instead')
  static IssuerProfile create() => IssuerProfile._();
  static $pb.GeneratedMessage $_createMessage() => IssuerProfile._();
  @$core.override
  IssuerProfile createEmptyInstance() => IssuerProfile._();
  @$core.pragma('dart2js:noInline')
  static IssuerProfile getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<IssuerProfile>(
          IssuerProfile.$_createMessage);
  static IssuerProfile? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get partyId => $_getSZ(0);
  @$pb.TagNumber(1)
  set partyId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPartyId() => $_has(0);
  @$pb.TagNumber(1)
  void clearPartyId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get legalName => $_getSZ(1);
  @$pb.TagNumber(2)
  set legalName($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLegalName() => $_has(1);
  @$pb.TagNumber(2)
  void clearLegalName() => $_clearField(2);

  @$pb.TagNumber(3)
  $pb.PbList<$core.String> get addressLines => $_getList(2);

  @$pb.TagNumber(4)
  $core.String get oib => $_getSZ(3);
  @$pb.TagNumber(4)
  set oib($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasOib() => $_has(3);
  @$pb.TagNumber(4)
  void clearOib() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get vatId => $_getSZ(4);
  @$pb.TagNumber(5)
  set vatId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasVatId() => $_has(4);
  @$pb.TagNumber(5)
  void clearVatId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get iban => $_getSZ(5);
  @$pb.TagNumber(6)
  set iban($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasIban() => $_has(5);
  @$pb.TagNumber(6)
  void clearIban() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get swift => $_getSZ(6);
  @$pb.TagNumber(7)
  set swift($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasSwift() => $_has(6);
  @$pb.TagNumber(7)
  void clearSwift() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get bankName => $_getSZ(7);
  @$pb.TagNumber(8)
  set bankName($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasBankName() => $_has(7);
  @$pb.TagNumber(8)
  void clearBankName() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get court => $_getSZ(8);
  @$pb.TagNumber(9)
  set court($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasCourt() => $_has(8);
  @$pb.TagNumber(9)
  void clearCourt() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get registrationNo => $_getSZ(9);
  @$pb.TagNumber(10)
  set registrationNo($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasRegistrationNo() => $_has(9);
  @$pb.TagNumber(10)
  void clearRegistrationNo() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.String get shareCapital => $_getSZ(10);
  @$pb.TagNumber(11)
  set shareCapital($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasShareCapital() => $_has(10);
  @$pb.TagNumber(11)
  void clearShareCapital() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get boardMember => $_getSZ(11);
  @$pb.TagNumber(12)
  set boardMember($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasBoardMember() => $_has(11);
  @$pb.TagNumber(12)
  void clearBoardMember() => $_clearField(12);

  @$pb.TagNumber(13)
  $core.String get issuedBy => $_getSZ(12);
  @$pb.TagNumber(13)
  set issuedBy($core.String value) => $_setString(12, value);
  @$pb.TagNumber(13)
  $core.bool hasIssuedBy() => $_has(12);
  @$pb.TagNumber(13)
  void clearIssuedBy() => $_clearField(13);

  @$pb.TagNumber(14)
  $core.String get placeOfIssue => $_getSZ(13);
  @$pb.TagNumber(14)
  set placeOfIssue($core.String value) => $_setString(13, value);
  @$pb.TagNumber(14)
  $core.bool hasPlaceOfIssue() => $_has(13);
  @$pb.TagNumber(14)
  void clearPlaceOfIssue() => $_clearField(14);

  @$pb.TagNumber(15)
  $core.String get operatorId => $_getSZ(14);
  @$pb.TagNumber(15)
  set operatorId($core.String value) => $_setString(14, value);
  @$pb.TagNumber(15)
  $core.bool hasOperatorId() => $_has(14);
  @$pb.TagNumber(15)
  void clearOperatorId() => $_clearField(15);

  @$pb.TagNumber(16)
  $core.String get premises => $_getSZ(15);
  @$pb.TagNumber(16)
  set premises($core.String value) => $_setString(15, value);
  @$pb.TagNumber(16)
  $core.bool hasPremises() => $_has(15);
  @$pb.TagNumber(16)
  void clearPremises() => $_clearField(16);

  @$pb.TagNumber(17)
  $core.String get device => $_getSZ(16);
  @$pb.TagNumber(17)
  set device($core.String value) => $_setString(16, value);
  @$pb.TagNumber(17)
  $core.bool hasDevice() => $_has(16);
  @$pb.TagNumber(17)
  void clearDevice() => $_clearField(17);

  @$pb.TagNumber(18)
  $core.int get dueDays => $_getIZ(17);
  @$pb.TagNumber(18)
  set dueDays($core.int value) => $_setSignedInt32(17, value);
  @$pb.TagNumber(18)
  $core.bool hasDueDays() => $_has(17);
  @$pb.TagNumber(18)
  void clearDueDays() => $_clearField(18);
}

class GetIssuerRequest extends $pb.GeneratedMessage {
  factory GetIssuerRequest({
    $core.String? partyId,
  }) {
    final result = GetIssuerRequest._();
    if (partyId != null) result.partyId = partyId;
    return result;
  }

  GetIssuerRequest._();

  factory GetIssuerRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetIssuerRequest()..mergeFromBuffer(data, registry);
  factory GetIssuerRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetIssuerRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetIssuerRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetIssuerRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'partyId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetIssuerRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetIssuerRequest copyWith(void Function(GetIssuerRequest) updates) =>
      super.copyWith((message) => updates(message as GetIssuerRequest))
          as GetIssuerRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetIssuerRequest() / GetIssuerRequest.new instead')
  static GetIssuerRequest create() => GetIssuerRequest._();
  static $pb.GeneratedMessage $_createMessage() => GetIssuerRequest._();
  @$core.override
  GetIssuerRequest createEmptyInstance() => GetIssuerRequest._();
  @$core.pragma('dart2js:noInline')
  static GetIssuerRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetIssuerRequest>(
          GetIssuerRequest.$_createMessage);
  static GetIssuerRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get partyId => $_getSZ(0);
  @$pb.TagNumber(1)
  set partyId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPartyId() => $_has(0);
  @$pb.TagNumber(1)
  void clearPartyId() => $_clearField(1);
}

class GetIssuerResponse extends $pb.GeneratedMessage {
  factory GetIssuerResponse({
    IssuerProfile? issuer,
  }) {
    final result = GetIssuerResponse._();
    if (issuer != null) result.issuer = issuer;
    return result;
  }

  GetIssuerResponse._();

  factory GetIssuerResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetIssuerResponse()..mergeFromBuffer(data, registry);
  factory GetIssuerResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetIssuerResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetIssuerResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetIssuerResponse.$_createMessage)
    ..aOM<IssuerProfile>(1, _omitFieldNames ? '' : 'issuer',
        subBuilder: IssuerProfile.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetIssuerResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetIssuerResponse copyWith(void Function(GetIssuerResponse) updates) =>
      super.copyWith((message) => updates(message as GetIssuerResponse))
          as GetIssuerResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetIssuerResponse() / GetIssuerResponse.new instead')
  static GetIssuerResponse create() => GetIssuerResponse._();
  static $pb.GeneratedMessage $_createMessage() => GetIssuerResponse._();
  @$core.override
  GetIssuerResponse createEmptyInstance() => GetIssuerResponse._();
  @$core.pragma('dart2js:noInline')
  static GetIssuerResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetIssuerResponse>(
          GetIssuerResponse.$_createMessage);
  static GetIssuerResponse? _defaultInstance;

  /// Absent when none has been set.
  @$pb.TagNumber(1)
  IssuerProfile get issuer => $_getN(0);
  @$pb.TagNumber(1)
  set issuer(IssuerProfile value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasIssuer() => $_has(0);
  @$pb.TagNumber(1)
  void clearIssuer() => $_clearField(1);
  @$pb.TagNumber(1)
  IssuerProfile ensureIssuer() => $_ensure(0);
}

class UpsertIssuerRequest extends $pb.GeneratedMessage {
  factory UpsertIssuerRequest({
    IssuerProfile? issuer,
  }) {
    final result = UpsertIssuerRequest._();
    if (issuer != null) result.issuer = issuer;
    return result;
  }

  UpsertIssuerRequest._();

  factory UpsertIssuerRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertIssuerRequest()..mergeFromBuffer(data, registry);
  factory UpsertIssuerRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertIssuerRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertIssuerRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertIssuerRequest.$_createMessage)
    ..aOM<IssuerProfile>(1, _omitFieldNames ? '' : 'issuer',
        subBuilder: IssuerProfile.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertIssuerRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertIssuerRequest copyWith(void Function(UpsertIssuerRequest) updates) =>
      super.copyWith((message) => updates(message as UpsertIssuerRequest))
          as UpsertIssuerRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use UpsertIssuerRequest() / UpsertIssuerRequest.new instead')
  static UpsertIssuerRequest create() => UpsertIssuerRequest._();
  static $pb.GeneratedMessage $_createMessage() => UpsertIssuerRequest._();
  @$core.override
  UpsertIssuerRequest createEmptyInstance() => UpsertIssuerRequest._();
  @$core.pragma('dart2js:noInline')
  static UpsertIssuerRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertIssuerRequest>(
          UpsertIssuerRequest.$_createMessage);
  static UpsertIssuerRequest? _defaultInstance;

  @$pb.TagNumber(1)
  IssuerProfile get issuer => $_getN(0);
  @$pb.TagNumber(1)
  set issuer(IssuerProfile value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasIssuer() => $_has(0);
  @$pb.TagNumber(1)
  void clearIssuer() => $_clearField(1);
  @$pb.TagNumber(1)
  IssuerProfile ensureIssuer() => $_ensure(0);
}

class UpsertIssuerResponse extends $pb.GeneratedMessage {
  factory UpsertIssuerResponse({
    IssuerProfile? issuer,
  }) {
    final result = UpsertIssuerResponse._();
    if (issuer != null) result.issuer = issuer;
    return result;
  }

  UpsertIssuerResponse._();

  factory UpsertIssuerResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertIssuerResponse()..mergeFromBuffer(data, registry);
  factory UpsertIssuerResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertIssuerResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertIssuerResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertIssuerResponse.$_createMessage)
    ..aOM<IssuerProfile>(1, _omitFieldNames ? '' : 'issuer',
        subBuilder: IssuerProfile.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertIssuerResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertIssuerResponse copyWith(void Function(UpsertIssuerResponse) updates) =>
      super.copyWith((message) => updates(message as UpsertIssuerResponse))
          as UpsertIssuerResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpsertIssuerResponse() / UpsertIssuerResponse.new instead')
  static UpsertIssuerResponse create() => UpsertIssuerResponse._();
  static $pb.GeneratedMessage $_createMessage() => UpsertIssuerResponse._();
  @$core.override
  UpsertIssuerResponse createEmptyInstance() => UpsertIssuerResponse._();
  @$core.pragma('dart2js:noInline')
  static UpsertIssuerResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertIssuerResponse>(
          UpsertIssuerResponse.$_createMessage);
  static UpsertIssuerResponse? _defaultInstance;

  @$pb.TagNumber(1)
  IssuerProfile get issuer => $_getN(0);
  @$pb.TagNumber(1)
  set issuer(IssuerProfile value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasIssuer() => $_has(0);
  @$pb.TagNumber(1)
  void clearIssuer() => $_clearField(1);
  @$pb.TagNumber(1)
  IssuerProfile ensureIssuer() => $_ensure(0);
}

class ClientProfile extends $pb.GeneratedMessage {
  factory ClientProfile({
    $core.String? id,
    $core.String? partyId,
    $core.String? name,
    $core.Iterable<$core.String>? addressLines,
    $core.String? countryCode,
    $core.String? taxId,
    $core.String? vatTreatment,
    $core.Iterable<$core.String>? recipients,
    $core.String? currency,
    $core.bool? archived,
  }) {
    final result = ClientProfile._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (name != null) result.name = name;
    if (addressLines != null) result.addressLines.addAll(addressLines);
    if (countryCode != null) result.countryCode = countryCode;
    if (taxId != null) result.taxId = taxId;
    if (vatTreatment != null) result.vatTreatment = vatTreatment;
    if (recipients != null) result.recipients.addAll(recipients);
    if (currency != null) result.currency = currency;
    if (archived != null) result.archived = archived;
    return result;
  }

  ClientProfile._();

  factory ClientProfile.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ClientProfile()..mergeFromBuffer(data, registry);
  factory ClientProfile.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ClientProfile()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ClientProfile',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ClientProfile.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'name')
    ..pPS(4, _omitFieldNames ? '' : 'addressLines')
    ..aOS(5, _omitFieldNames ? '' : 'countryCode')
    ..aOS(6, _omitFieldNames ? '' : 'taxId')
    ..aOS(7, _omitFieldNames ? '' : 'vatTreatment')
    ..pPS(8, _omitFieldNames ? '' : 'recipients')
    ..aOS(9, _omitFieldNames ? '' : 'currency')
    ..aOB(10, _omitFieldNames ? '' : 'archived')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientProfile clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ClientProfile copyWith(void Function(ClientProfile) updates) =>
      super.copyWith((message) => updates(message as ClientProfile))
          as ClientProfile;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ClientProfile() / ClientProfile.new instead')
  static ClientProfile create() => ClientProfile._();
  static $pb.GeneratedMessage $_createMessage() => ClientProfile._();
  @$core.override
  ClientProfile createEmptyInstance() => ClientProfile._();
  @$core.pragma('dart2js:noInline')
  static ClientProfile getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ClientProfile>(
          ClientProfile.$_createMessage);
  static ClientProfile? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get name => $_getSZ(2);
  @$pb.TagNumber(3)
  set name($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasName() => $_has(2);
  @$pb.TagNumber(3)
  void clearName() => $_clearField(3);

  @$pb.TagNumber(4)
  $pb.PbList<$core.String> get addressLines => $_getList(3);

  @$pb.TagNumber(5)
  $core.String get countryCode => $_getSZ(4);
  @$pb.TagNumber(5)
  set countryCode($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCountryCode() => $_has(4);
  @$pb.TagNumber(5)
  void clearCountryCode() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get taxId => $_getSZ(5);
  @$pb.TagNumber(6)
  set taxId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTaxId() => $_has(5);
  @$pb.TagNumber(6)
  void clearTaxId() => $_clearField(6);

  /// standard_hr, reverse_charge_eu, outside_scope_non_eu, exempt_issuer.
  @$pb.TagNumber(7)
  $core.String get vatTreatment => $_getSZ(6);
  @$pb.TagNumber(7)
  set vatTreatment($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasVatTreatment() => $_has(6);
  @$pb.TagNumber(7)
  void clearVatTreatment() => $_clearField(7);

  @$pb.TagNumber(8)
  $pb.PbList<$core.String> get recipients => $_getList(7);

  @$pb.TagNumber(9)
  $core.String get currency => $_getSZ(8);
  @$pb.TagNumber(9)
  set currency($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasCurrency() => $_has(8);
  @$pb.TagNumber(9)
  void clearCurrency() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.bool get archived => $_getBF(9);
  @$pb.TagNumber(10)
  set archived($core.bool value) => $_setBool(9, value);
  @$pb.TagNumber(10)
  $core.bool hasArchived() => $_has(9);
  @$pb.TagNumber(10)
  void clearArchived() => $_clearField(10);
}

class ListClientsRequest extends $pb.GeneratedMessage {
  factory ListClientsRequest({
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = ListClientsRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  ListClientsRequest._();

  factory ListClientsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListClientsRequest()..mergeFromBuffer(data, registry);
  factory ListClientsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListClientsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListClientsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListClientsRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListClientsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListClientsRequest copyWith(void Function(ListClientsRequest) updates) =>
      super.copyWith((message) => updates(message as ListClientsRequest))
          as ListClientsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ListClientsRequest() / ListClientsRequest.new instead')
  static ListClientsRequest create() => ListClientsRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListClientsRequest._();
  @$core.override
  ListClientsRequest createEmptyInstance() => ListClientsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListClientsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListClientsRequest>(
          ListClientsRequest.$_createMessage);
  static ListClientsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);
}

class ListClientsResponse extends $pb.GeneratedMessage {
  factory ListClientsResponse({
    $core.Iterable<ClientProfile>? clients,
  }) {
    final result = ListClientsResponse._();
    if (clients != null) result.clients.addAll(clients);
    return result;
  }

  ListClientsResponse._();

  factory ListClientsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListClientsResponse()..mergeFromBuffer(data, registry);
  factory ListClientsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListClientsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListClientsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListClientsResponse.$_createMessage)
    ..pPM<ClientProfile>(1, _omitFieldNames ? '' : 'clients',
        subBuilder: ClientProfile.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListClientsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListClientsResponse copyWith(void Function(ListClientsResponse) updates) =>
      super.copyWith((message) => updates(message as ListClientsResponse))
          as ListClientsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use ListClientsResponse() / ListClientsResponse.new instead')
  static ListClientsResponse create() => ListClientsResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListClientsResponse._();
  @$core.override
  ListClientsResponse createEmptyInstance() => ListClientsResponse._();
  @$core.pragma('dart2js:noInline')
  static ListClientsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListClientsResponse>(
          ListClientsResponse.$_createMessage);
  static ListClientsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<ClientProfile> get clients => $_getList(0);
}

class UpsertClientRequest extends $pb.GeneratedMessage {
  factory UpsertClientRequest({
    ClientProfile? client,
  }) {
    final result = UpsertClientRequest._();
    if (client != null) result.client = client;
    return result;
  }

  UpsertClientRequest._();

  factory UpsertClientRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertClientRequest()..mergeFromBuffer(data, registry);
  factory UpsertClientRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertClientRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertClientRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertClientRequest.$_createMessage)
    ..aOM<ClientProfile>(1, _omitFieldNames ? '' : 'client',
        subBuilder: ClientProfile.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertClientRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertClientRequest copyWith(void Function(UpsertClientRequest) updates) =>
      super.copyWith((message) => updates(message as UpsertClientRequest))
          as UpsertClientRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use UpsertClientRequest() / UpsertClientRequest.new instead')
  static UpsertClientRequest create() => UpsertClientRequest._();
  static $pb.GeneratedMessage $_createMessage() => UpsertClientRequest._();
  @$core.override
  UpsertClientRequest createEmptyInstance() => UpsertClientRequest._();
  @$core.pragma('dart2js:noInline')
  static UpsertClientRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertClientRequest>(
          UpsertClientRequest.$_createMessage);
  static UpsertClientRequest? _defaultInstance;

  /// Empty id creates.
  @$pb.TagNumber(1)
  ClientProfile get client => $_getN(0);
  @$pb.TagNumber(1)
  set client(ClientProfile value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasClient() => $_has(0);
  @$pb.TagNumber(1)
  void clearClient() => $_clearField(1);
  @$pb.TagNumber(1)
  ClientProfile ensureClient() => $_ensure(0);
}

class UpsertClientResponse extends $pb.GeneratedMessage {
  factory UpsertClientResponse({
    ClientProfile? client,
  }) {
    final result = UpsertClientResponse._();
    if (client != null) result.client = client;
    return result;
  }

  UpsertClientResponse._();

  factory UpsertClientResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertClientResponse()..mergeFromBuffer(data, registry);
  factory UpsertClientResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertClientResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertClientResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertClientResponse.$_createMessage)
    ..aOM<ClientProfile>(1, _omitFieldNames ? '' : 'client',
        subBuilder: ClientProfile.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertClientResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertClientResponse copyWith(void Function(UpsertClientResponse) updates) =>
      super.copyWith((message) => updates(message as UpsertClientResponse))
          as UpsertClientResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpsertClientResponse() / UpsertClientResponse.new instead')
  static UpsertClientResponse create() => UpsertClientResponse._();
  static $pb.GeneratedMessage $_createMessage() => UpsertClientResponse._();
  @$core.override
  UpsertClientResponse createEmptyInstance() => UpsertClientResponse._();
  @$core.pragma('dart2js:noInline')
  static UpsertClientResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertClientResponse>(
          UpsertClientResponse.$_createMessage);
  static UpsertClientResponse? _defaultInstance;

  @$pb.TagNumber(1)
  ClientProfile get client => $_getN(0);
  @$pb.TagNumber(1)
  set client(ClientProfile value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasClient() => $_has(0);
  @$pb.TagNumber(1)
  void clearClient() => $_clearField(1);
  @$pb.TagNumber(1)
  ClientProfile ensureClient() => $_ensure(0);
}

class InvoiceLine extends $pb.GeneratedMessage {
  factory InvoiceLine({
    $core.int? position,
    $core.String? description,
    $fixnum.Int64? quantityMilli,
    $fixnum.Int64? unitPriceMinor,
    $fixnum.Int64? amountMinor,
    $core.String? templateId,
  }) {
    final result = InvoiceLine._();
    if (position != null) result.position = position;
    if (description != null) result.description = description;
    if (quantityMilli != null) result.quantityMilli = quantityMilli;
    if (unitPriceMinor != null) result.unitPriceMinor = unitPriceMinor;
    if (amountMinor != null) result.amountMinor = amountMinor;
    if (templateId != null) result.templateId = templateId;
    return result;
  }

  InvoiceLine._();

  factory InvoiceLine.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InvoiceLine()..mergeFromBuffer(data, registry);
  factory InvoiceLine.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      InvoiceLine()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'InvoiceLine',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: InvoiceLine.$_createMessage)
    ..aI(1, _omitFieldNames ? '' : 'position')
    ..aOS(2, _omitFieldNames ? '' : 'description')
    ..aInt64(3, _omitFieldNames ? '' : 'quantityMilli')
    ..aInt64(4, _omitFieldNames ? '' : 'unitPriceMinor')
    ..aInt64(5, _omitFieldNames ? '' : 'amountMinor')
    ..aOS(6, _omitFieldNames ? '' : 'templateId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InvoiceLine clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  InvoiceLine copyWith(void Function(InvoiceLine) updates) =>
      super.copyWith((message) => updates(message as InvoiceLine))
          as InvoiceLine;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use InvoiceLine() / InvoiceLine.new instead')
  static InvoiceLine create() => InvoiceLine._();
  static $pb.GeneratedMessage $_createMessage() => InvoiceLine._();
  @$core.override
  InvoiceLine createEmptyInstance() => InvoiceLine._();
  @$core.pragma('dart2js:noInline')
  static InvoiceLine getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<InvoiceLine>(
          InvoiceLine.$_createMessage);
  static InvoiceLine? _defaultInstance;

  @$pb.TagNumber(1)
  $core.int get position => $_getIZ(0);
  @$pb.TagNumber(1)
  set position($core.int value) => $_setSignedInt32(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPosition() => $_has(0);
  @$pb.TagNumber(1)
  void clearPosition() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get description => $_getSZ(1);
  @$pb.TagNumber(2)
  set description($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDescription() => $_has(1);
  @$pb.TagNumber(2)
  void clearDescription() => $_clearField(2);

  /// Thousandths: 1500 is 1.5.
  @$pb.TagNumber(3)
  $fixnum.Int64 get quantityMilli => $_getI64(2);
  @$pb.TagNumber(3)
  set quantityMilli($fixnum.Int64 value) => $_setInt64(2, value);
  @$pb.TagNumber(3)
  $core.bool hasQuantityMilli() => $_has(2);
  @$pb.TagNumber(3)
  void clearQuantityMilli() => $_clearField(3);

  @$pb.TagNumber(4)
  $fixnum.Int64 get unitPriceMinor => $_getI64(3);
  @$pb.TagNumber(4)
  set unitPriceMinor($fixnum.Int64 value) => $_setInt64(3, value);
  @$pb.TagNumber(4)
  $core.bool hasUnitPriceMinor() => $_has(3);
  @$pb.TagNumber(4)
  void clearUnitPriceMinor() => $_clearField(4);

  /// Computed by the service.
  @$pb.TagNumber(5)
  $fixnum.Int64 get amountMinor => $_getI64(4);
  @$pb.TagNumber(5)
  set amountMinor($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasAmountMinor() => $_has(4);
  @$pb.TagNumber(5)
  void clearAmountMinor() => $_clearField(5);

  /// The template this line came from; empty for one typed by hand.
  @$pb.TagNumber(6)
  $core.String get templateId => $_getSZ(5);
  @$pb.TagNumber(6)
  set templateId($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasTemplateId() => $_has(5);
  @$pb.TagNumber(6)
  void clearTemplateId() => $_clearField(6);
}

class Invoice extends $pb.GeneratedMessage {
  factory Invoice({
    $core.String? id,
    $core.String? partyId,
    $core.String? clientId,
    $core.String? status,
    $core.String? number,
    $core.int? year,
    $core.String? issuedAt,
    $core.String? deliveryDate,
    $core.String? dueDate,
    $core.String? placeOfIssue,
    $core.String? currency,
    $fixnum.Int64? subtotalMinor,
    $fixnum.Int64? vatMinor,
    $fixnum.Int64? totalMinor,
    $core.String? vatTreatment,
    $core.String? vatNote,
    $core.String? note,
    $core.String? contentHash,
    $core.String? approvedAt,
    $core.String? documentId,
    $core.String? prefilledFrom,
    $core.String? cancelledAt,
    $core.String? createdAt,
    $core.String? updatedAt,
    $core.Iterable<InvoiceLine>? lines,
  }) {
    final result = Invoice._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (clientId != null) result.clientId = clientId;
    if (status != null) result.status = status;
    if (number != null) result.number = number;
    if (year != null) result.year = year;
    if (issuedAt != null) result.issuedAt = issuedAt;
    if (deliveryDate != null) result.deliveryDate = deliveryDate;
    if (dueDate != null) result.dueDate = dueDate;
    if (placeOfIssue != null) result.placeOfIssue = placeOfIssue;
    if (currency != null) result.currency = currency;
    if (subtotalMinor != null) result.subtotalMinor = subtotalMinor;
    if (vatMinor != null) result.vatMinor = vatMinor;
    if (totalMinor != null) result.totalMinor = totalMinor;
    if (vatTreatment != null) result.vatTreatment = vatTreatment;
    if (vatNote != null) result.vatNote = vatNote;
    if (note != null) result.note = note;
    if (contentHash != null) result.contentHash = contentHash;
    if (approvedAt != null) result.approvedAt = approvedAt;
    if (documentId != null) result.documentId = documentId;
    if (prefilledFrom != null) result.prefilledFrom = prefilledFrom;
    if (cancelledAt != null) result.cancelledAt = cancelledAt;
    if (createdAt != null) result.createdAt = createdAt;
    if (updatedAt != null) result.updatedAt = updatedAt;
    if (lines != null) result.lines.addAll(lines);
    return result;
  }

  Invoice._();

  factory Invoice.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Invoice()..mergeFromBuffer(data, registry);
  factory Invoice.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Invoice()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Invoice',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Invoice.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'clientId')
    ..aOS(4, _omitFieldNames ? '' : 'status')
    ..aOS(5, _omitFieldNames ? '' : 'number')
    ..aI(6, _omitFieldNames ? '' : 'year')
    ..aOS(7, _omitFieldNames ? '' : 'issuedAt')
    ..aOS(8, _omitFieldNames ? '' : 'deliveryDate')
    ..aOS(9, _omitFieldNames ? '' : 'dueDate')
    ..aOS(10, _omitFieldNames ? '' : 'placeOfIssue')
    ..aOS(11, _omitFieldNames ? '' : 'currency')
    ..aInt64(12, _omitFieldNames ? '' : 'subtotalMinor')
    ..aInt64(13, _omitFieldNames ? '' : 'vatMinor')
    ..aInt64(14, _omitFieldNames ? '' : 'totalMinor')
    ..aOS(15, _omitFieldNames ? '' : 'vatTreatment')
    ..aOS(16, _omitFieldNames ? '' : 'vatNote')
    ..aOS(17, _omitFieldNames ? '' : 'note')
    ..aOS(18, _omitFieldNames ? '' : 'contentHash')
    ..aOS(19, _omitFieldNames ? '' : 'approvedAt')
    ..aOS(20, _omitFieldNames ? '' : 'documentId')
    ..aOS(21, _omitFieldNames ? '' : 'prefilledFrom')
    ..aOS(22, _omitFieldNames ? '' : 'cancelledAt')
    ..aOS(23, _omitFieldNames ? '' : 'createdAt')
    ..aOS(24, _omitFieldNames ? '' : 'updatedAt')
    ..pPM<InvoiceLine>(25, _omitFieldNames ? '' : 'lines',
        subBuilder: InvoiceLine.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Invoice clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Invoice copyWith(void Function(Invoice) updates) =>
      super.copyWith((message) => updates(message as Invoice)) as Invoice;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Invoice() / Invoice.new instead')
  static Invoice create() => Invoice._();
  static $pb.GeneratedMessage $_createMessage() => Invoice._();
  @$core.override
  Invoice createEmptyInstance() => Invoice._();
  @$core.pragma('dart2js:noInline')
  static Invoice getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Invoice>(Invoice.$_createMessage);
  static Invoice? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get clientId => $_getSZ(2);
  @$pb.TagNumber(3)
  set clientId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasClientId() => $_has(2);
  @$pb.TagNumber(3)
  void clearClientId() => $_clearField(3);

  /// draft, approved, sent, paid, cancelled.
  @$pb.TagNumber(4)
  $core.String get status => $_getSZ(3);
  @$pb.TagNumber(4)
  set status($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasStatus() => $_has(3);
  @$pb.TagNumber(4)
  void clearStatus() => $_clearField(4);

  /// Empty until approved.
  @$pb.TagNumber(5)
  $core.String get number => $_getSZ(4);
  @$pb.TagNumber(5)
  set number($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasNumber() => $_has(4);
  @$pb.TagNumber(5)
  void clearNumber() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.int get year => $_getIZ(5);
  @$pb.TagNumber(6)
  set year($core.int value) => $_setSignedInt32(5, value);
  @$pb.TagNumber(6)
  $core.bool hasYear() => $_has(5);
  @$pb.TagNumber(6)
  void clearYear() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get issuedAt => $_getSZ(6);
  @$pb.TagNumber(7)
  set issuedAt($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasIssuedAt() => $_has(6);
  @$pb.TagNumber(7)
  void clearIssuedAt() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get deliveryDate => $_getSZ(7);
  @$pb.TagNumber(8)
  set deliveryDate($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasDeliveryDate() => $_has(7);
  @$pb.TagNumber(8)
  void clearDeliveryDate() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get dueDate => $_getSZ(8);
  @$pb.TagNumber(9)
  set dueDate($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasDueDate() => $_has(8);
  @$pb.TagNumber(9)
  void clearDueDate() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get placeOfIssue => $_getSZ(9);
  @$pb.TagNumber(10)
  set placeOfIssue($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasPlaceOfIssue() => $_has(9);
  @$pb.TagNumber(10)
  void clearPlaceOfIssue() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.String get currency => $_getSZ(10);
  @$pb.TagNumber(11)
  set currency($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasCurrency() => $_has(10);
  @$pb.TagNumber(11)
  void clearCurrency() => $_clearField(11);

  @$pb.TagNumber(12)
  $fixnum.Int64 get subtotalMinor => $_getI64(11);
  @$pb.TagNumber(12)
  set subtotalMinor($fixnum.Int64 value) => $_setInt64(11, value);
  @$pb.TagNumber(12)
  $core.bool hasSubtotalMinor() => $_has(11);
  @$pb.TagNumber(12)
  void clearSubtotalMinor() => $_clearField(12);

  @$pb.TagNumber(13)
  $fixnum.Int64 get vatMinor => $_getI64(12);
  @$pb.TagNumber(13)
  set vatMinor($fixnum.Int64 value) => $_setInt64(12, value);
  @$pb.TagNumber(13)
  $core.bool hasVatMinor() => $_has(12);
  @$pb.TagNumber(13)
  void clearVatMinor() => $_clearField(13);

  @$pb.TagNumber(14)
  $fixnum.Int64 get totalMinor => $_getI64(13);
  @$pb.TagNumber(14)
  set totalMinor($fixnum.Int64 value) => $_setInt64(13, value);
  @$pb.TagNumber(14)
  $core.bool hasTotalMinor() => $_has(13);
  @$pb.TagNumber(14)
  void clearTotalMinor() => $_clearField(14);

  @$pb.TagNumber(15)
  $core.String get vatTreatment => $_getSZ(14);
  @$pb.TagNumber(15)
  set vatTreatment($core.String value) => $_setString(14, value);
  @$pb.TagNumber(15)
  $core.bool hasVatTreatment() => $_has(14);
  @$pb.TagNumber(15)
  void clearVatTreatment() => $_clearField(15);

  @$pb.TagNumber(16)
  $core.String get vatNote => $_getSZ(15);
  @$pb.TagNumber(16)
  set vatNote($core.String value) => $_setString(15, value);
  @$pb.TagNumber(16)
  $core.bool hasVatNote() => $_has(15);
  @$pb.TagNumber(16)
  void clearVatNote() => $_clearField(16);

  @$pb.TagNumber(17)
  $core.String get note => $_getSZ(16);
  @$pb.TagNumber(17)
  set note($core.String value) => $_setString(16, value);
  @$pb.TagNumber(17)
  $core.bool hasNote() => $_has(16);
  @$pb.TagNumber(17)
  void clearNote() => $_clearField(17);

  @$pb.TagNumber(18)
  $core.String get contentHash => $_getSZ(17);
  @$pb.TagNumber(18)
  set contentHash($core.String value) => $_setString(17, value);
  @$pb.TagNumber(18)
  $core.bool hasContentHash() => $_has(17);
  @$pb.TagNumber(18)
  void clearContentHash() => $_clearField(18);

  @$pb.TagNumber(19)
  $core.String get approvedAt => $_getSZ(18);
  @$pb.TagNumber(19)
  set approvedAt($core.String value) => $_setString(18, value);
  @$pb.TagNumber(19)
  $core.bool hasApprovedAt() => $_has(18);
  @$pb.TagNumber(19)
  void clearApprovedAt() => $_clearField(19);

  @$pb.TagNumber(20)
  $core.String get documentId => $_getSZ(19);
  @$pb.TagNumber(20)
  set documentId($core.String value) => $_setString(19, value);
  @$pb.TagNumber(20)
  $core.bool hasDocumentId() => $_has(19);
  @$pb.TagNumber(20)
  void clearDocumentId() => $_clearField(20);

  @$pb.TagNumber(21)
  $core.String get prefilledFrom => $_getSZ(20);
  @$pb.TagNumber(21)
  set prefilledFrom($core.String value) => $_setString(20, value);
  @$pb.TagNumber(21)
  $core.bool hasPrefilledFrom() => $_has(20);
  @$pb.TagNumber(21)
  void clearPrefilledFrom() => $_clearField(21);

  @$pb.TagNumber(22)
  $core.String get cancelledAt => $_getSZ(21);
  @$pb.TagNumber(22)
  set cancelledAt($core.String value) => $_setString(21, value);
  @$pb.TagNumber(22)
  $core.bool hasCancelledAt() => $_has(21);
  @$pb.TagNumber(22)
  void clearCancelledAt() => $_clearField(22);

  @$pb.TagNumber(23)
  $core.String get createdAt => $_getSZ(22);
  @$pb.TagNumber(23)
  set createdAt($core.String value) => $_setString(22, value);
  @$pb.TagNumber(23)
  $core.bool hasCreatedAt() => $_has(22);
  @$pb.TagNumber(23)
  void clearCreatedAt() => $_clearField(23);

  @$pb.TagNumber(24)
  $core.String get updatedAt => $_getSZ(23);
  @$pb.TagNumber(24)
  set updatedAt($core.String value) => $_setString(23, value);
  @$pb.TagNumber(24)
  $core.bool hasUpdatedAt() => $_has(23);
  @$pb.TagNumber(24)
  void clearUpdatedAt() => $_clearField(24);

  @$pb.TagNumber(25)
  $pb.PbList<InvoiceLine> get lines => $_getList(24);
}

class ListInvoicesRequest extends $pb.GeneratedMessage {
  factory ListInvoicesRequest({
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = ListInvoicesRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  ListInvoicesRequest._();

  factory ListInvoicesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListInvoicesRequest()..mergeFromBuffer(data, registry);
  factory ListInvoicesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListInvoicesRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListInvoicesRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListInvoicesRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInvoicesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInvoicesRequest copyWith(void Function(ListInvoicesRequest) updates) =>
      super.copyWith((message) => updates(message as ListInvoicesRequest))
          as ListInvoicesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use ListInvoicesRequest() / ListInvoicesRequest.new instead')
  static ListInvoicesRequest create() => ListInvoicesRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListInvoicesRequest._();
  @$core.override
  ListInvoicesRequest createEmptyInstance() => ListInvoicesRequest._();
  @$core.pragma('dart2js:noInline')
  static ListInvoicesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListInvoicesRequest>(
          ListInvoicesRequest.$_createMessage);
  static ListInvoicesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);
}

class ListInvoicesResponse extends $pb.GeneratedMessage {
  factory ListInvoicesResponse({
    $core.Iterable<Invoice>? invoices,
  }) {
    final result = ListInvoicesResponse._();
    if (invoices != null) result.invoices.addAll(invoices);
    return result;
  }

  ListInvoicesResponse._();

  factory ListInvoicesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListInvoicesResponse()..mergeFromBuffer(data, registry);
  factory ListInvoicesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListInvoicesResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListInvoicesResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListInvoicesResponse.$_createMessage)
    ..pPM<Invoice>(1, _omitFieldNames ? '' : 'invoices',
        subBuilder: Invoice.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInvoicesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListInvoicesResponse copyWith(void Function(ListInvoicesResponse) updates) =>
      super.copyWith((message) => updates(message as ListInvoicesResponse))
          as ListInvoicesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListInvoicesResponse() / ListInvoicesResponse.new instead')
  static ListInvoicesResponse create() => ListInvoicesResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListInvoicesResponse._();
  @$core.override
  ListInvoicesResponse createEmptyInstance() => ListInvoicesResponse._();
  @$core.pragma('dart2js:noInline')
  static ListInvoicesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListInvoicesResponse>(
          ListInvoicesResponse.$_createMessage);
  static ListInvoicesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Invoice> get invoices => $_getList(0);
}

class GetInvoiceRequest extends $pb.GeneratedMessage {
  factory GetInvoiceRequest({
    $core.String? id,
  }) {
    final result = GetInvoiceRequest._();
    if (id != null) result.id = id;
    return result;
  }

  GetInvoiceRequest._();

  factory GetInvoiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetInvoiceRequest()..mergeFromBuffer(data, registry);
  factory GetInvoiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetInvoiceRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetInvoiceRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetInvoiceRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInvoiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInvoiceRequest copyWith(void Function(GetInvoiceRequest) updates) =>
      super.copyWith((message) => updates(message as GetInvoiceRequest))
          as GetInvoiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetInvoiceRequest() / GetInvoiceRequest.new instead')
  static GetInvoiceRequest create() => GetInvoiceRequest._();
  static $pb.GeneratedMessage $_createMessage() => GetInvoiceRequest._();
  @$core.override
  GetInvoiceRequest createEmptyInstance() => GetInvoiceRequest._();
  @$core.pragma('dart2js:noInline')
  static GetInvoiceRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<GetInvoiceRequest>(
          GetInvoiceRequest.$_createMessage);
  static GetInvoiceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetInvoiceResponse extends $pb.GeneratedMessage {
  factory GetInvoiceResponse({
    Invoice? invoice,
  }) {
    final result = GetInvoiceResponse._();
    if (invoice != null) result.invoice = invoice;
    return result;
  }

  GetInvoiceResponse._();

  factory GetInvoiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetInvoiceResponse()..mergeFromBuffer(data, registry);
  factory GetInvoiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetInvoiceResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetInvoiceResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetInvoiceResponse.$_createMessage)
    ..aOM<Invoice>(1, _omitFieldNames ? '' : 'invoice',
        subBuilder: Invoice.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInvoiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInvoiceResponse copyWith(void Function(GetInvoiceResponse) updates) =>
      super.copyWith((message) => updates(message as GetInvoiceResponse))
          as GetInvoiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use GetInvoiceResponse() / GetInvoiceResponse.new instead')
  static GetInvoiceResponse create() => GetInvoiceResponse._();
  static $pb.GeneratedMessage $_createMessage() => GetInvoiceResponse._();
  @$core.override
  GetInvoiceResponse createEmptyInstance() => GetInvoiceResponse._();
  @$core.pragma('dart2js:noInline')
  static GetInvoiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetInvoiceResponse>(
          GetInvoiceResponse.$_createMessage);
  static GetInvoiceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Invoice get invoice => $_getN(0);
  @$pb.TagNumber(1)
  set invoice(Invoice value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasInvoice() => $_has(0);
  @$pb.TagNumber(1)
  void clearInvoice() => $_clearField(1);
  @$pb.TagNumber(1)
  Invoice ensureInvoice() => $_ensure(0);
}

class CreateInvoiceRequest extends $pb.GeneratedMessage {
  factory CreateInvoiceRequest({
    $core.String? clientId,
  }) {
    final result = CreateInvoiceRequest._();
    if (clientId != null) result.clientId = clientId;
    return result;
  }

  CreateInvoiceRequest._();

  factory CreateInvoiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateInvoiceRequest()..mergeFromBuffer(data, registry);
  factory CreateInvoiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateInvoiceRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateInvoiceRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: CreateInvoiceRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'clientId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateInvoiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateInvoiceRequest copyWith(void Function(CreateInvoiceRequest) updates) =>
      super.copyWith((message) => updates(message as CreateInvoiceRequest))
          as CreateInvoiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use CreateInvoiceRequest() / CreateInvoiceRequest.new instead')
  static CreateInvoiceRequest create() => CreateInvoiceRequest._();
  static $pb.GeneratedMessage $_createMessage() => CreateInvoiceRequest._();
  @$core.override
  CreateInvoiceRequest createEmptyInstance() => CreateInvoiceRequest._();
  @$core.pragma('dart2js:noInline')
  static CreateInvoiceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateInvoiceRequest>(
          CreateInvoiceRequest.$_createMessage);
  static CreateInvoiceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get clientId => $_getSZ(0);
  @$pb.TagNumber(1)
  set clientId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasClientId() => $_has(0);
  @$pb.TagNumber(1)
  void clearClientId() => $_clearField(1);
}

class CreateInvoiceResponse extends $pb.GeneratedMessage {
  factory CreateInvoiceResponse({
    Invoice? invoice,
  }) {
    final result = CreateInvoiceResponse._();
    if (invoice != null) result.invoice = invoice;
    return result;
  }

  CreateInvoiceResponse._();

  factory CreateInvoiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateInvoiceResponse()..mergeFromBuffer(data, registry);
  factory CreateInvoiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CreateInvoiceResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CreateInvoiceResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: CreateInvoiceResponse.$_createMessage)
    ..aOM<Invoice>(1, _omitFieldNames ? '' : 'invoice',
        subBuilder: Invoice.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateInvoiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CreateInvoiceResponse copyWith(
          void Function(CreateInvoiceResponse) updates) =>
      super.copyWith((message) => updates(message as CreateInvoiceResponse))
          as CreateInvoiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use CreateInvoiceResponse() / CreateInvoiceResponse.new instead')
  static CreateInvoiceResponse create() => CreateInvoiceResponse._();
  static $pb.GeneratedMessage $_createMessage() => CreateInvoiceResponse._();
  @$core.override
  CreateInvoiceResponse createEmptyInstance() => CreateInvoiceResponse._();
  @$core.pragma('dart2js:noInline')
  static CreateInvoiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CreateInvoiceResponse>(
          CreateInvoiceResponse.$_createMessage);
  static CreateInvoiceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Invoice get invoice => $_getN(0);
  @$pb.TagNumber(1)
  set invoice(Invoice value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasInvoice() => $_has(0);
  @$pb.TagNumber(1)
  void clearInvoice() => $_clearField(1);
  @$pb.TagNumber(1)
  Invoice ensureInvoice() => $_ensure(0);
}

class UpdateInvoiceRequest extends $pb.GeneratedMessage {
  factory UpdateInvoiceRequest({
    $core.String? id,
    $core.String? deliveryDate,
    $core.String? dueDate,
    $core.String? placeOfIssue,
    $core.String? note,
    $core.Iterable<InvoiceLine>? lines,
  }) {
    final result = UpdateInvoiceRequest._();
    if (id != null) result.id = id;
    if (deliveryDate != null) result.deliveryDate = deliveryDate;
    if (dueDate != null) result.dueDate = dueDate;
    if (placeOfIssue != null) result.placeOfIssue = placeOfIssue;
    if (note != null) result.note = note;
    if (lines != null) result.lines.addAll(lines);
    return result;
  }

  UpdateInvoiceRequest._();

  factory UpdateInvoiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpdateInvoiceRequest()..mergeFromBuffer(data, registry);
  factory UpdateInvoiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpdateInvoiceRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateInvoiceRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpdateInvoiceRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'deliveryDate')
    ..aOS(3, _omitFieldNames ? '' : 'dueDate')
    ..aOS(4, _omitFieldNames ? '' : 'placeOfIssue')
    ..aOS(5, _omitFieldNames ? '' : 'note')
    ..pPM<InvoiceLine>(6, _omitFieldNames ? '' : 'lines',
        subBuilder: InvoiceLine.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateInvoiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateInvoiceRequest copyWith(void Function(UpdateInvoiceRequest) updates) =>
      super.copyWith((message) => updates(message as UpdateInvoiceRequest))
          as UpdateInvoiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpdateInvoiceRequest() / UpdateInvoiceRequest.new instead')
  static UpdateInvoiceRequest create() => UpdateInvoiceRequest._();
  static $pb.GeneratedMessage $_createMessage() => UpdateInvoiceRequest._();
  @$core.override
  UpdateInvoiceRequest createEmptyInstance() => UpdateInvoiceRequest._();
  @$core.pragma('dart2js:noInline')
  static UpdateInvoiceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateInvoiceRequest>(
          UpdateInvoiceRequest.$_createMessage);
  static UpdateInvoiceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get deliveryDate => $_getSZ(1);
  @$pb.TagNumber(2)
  set deliveryDate($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasDeliveryDate() => $_has(1);
  @$pb.TagNumber(2)
  void clearDeliveryDate() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get dueDate => $_getSZ(2);
  @$pb.TagNumber(3)
  set dueDate($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDueDate() => $_has(2);
  @$pb.TagNumber(3)
  void clearDueDate() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get placeOfIssue => $_getSZ(3);
  @$pb.TagNumber(4)
  set placeOfIssue($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPlaceOfIssue() => $_has(3);
  @$pb.TagNumber(4)
  void clearPlaceOfIssue() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get note => $_getSZ(4);
  @$pb.TagNumber(5)
  set note($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasNote() => $_has(4);
  @$pb.TagNumber(5)
  void clearNote() => $_clearField(5);

  @$pb.TagNumber(6)
  $pb.PbList<InvoiceLine> get lines => $_getList(5);
}

class UpdateInvoiceResponse extends $pb.GeneratedMessage {
  factory UpdateInvoiceResponse({
    Invoice? invoice,
  }) {
    final result = UpdateInvoiceResponse._();
    if (invoice != null) result.invoice = invoice;
    return result;
  }

  UpdateInvoiceResponse._();

  factory UpdateInvoiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpdateInvoiceResponse()..mergeFromBuffer(data, registry);
  factory UpdateInvoiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpdateInvoiceResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpdateInvoiceResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpdateInvoiceResponse.$_createMessage)
    ..aOM<Invoice>(1, _omitFieldNames ? '' : 'invoice',
        subBuilder: Invoice.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateInvoiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpdateInvoiceResponse copyWith(
          void Function(UpdateInvoiceResponse) updates) =>
      super.copyWith((message) => updates(message as UpdateInvoiceResponse))
          as UpdateInvoiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpdateInvoiceResponse() / UpdateInvoiceResponse.new instead')
  static UpdateInvoiceResponse create() => UpdateInvoiceResponse._();
  static $pb.GeneratedMessage $_createMessage() => UpdateInvoiceResponse._();
  @$core.override
  UpdateInvoiceResponse createEmptyInstance() => UpdateInvoiceResponse._();
  @$core.pragma('dart2js:noInline')
  static UpdateInvoiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpdateInvoiceResponse>(
          UpdateInvoiceResponse.$_createMessage);
  static UpdateInvoiceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Invoice get invoice => $_getN(0);
  @$pb.TagNumber(1)
  set invoice(Invoice value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasInvoice() => $_has(0);
  @$pb.TagNumber(1)
  void clearInvoice() => $_clearField(1);
  @$pb.TagNumber(1)
  Invoice ensureInvoice() => $_ensure(0);
}

class PreviewInvoiceRequest extends $pb.GeneratedMessage {
  factory PreviewInvoiceRequest({
    $core.String? id,
  }) {
    final result = PreviewInvoiceRequest._();
    if (id != null) result.id = id;
    return result;
  }

  PreviewInvoiceRequest._();

  factory PreviewInvoiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PreviewInvoiceRequest()..mergeFromBuffer(data, registry);
  factory PreviewInvoiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PreviewInvoiceRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PreviewInvoiceRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: PreviewInvoiceRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PreviewInvoiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PreviewInvoiceRequest copyWith(
          void Function(PreviewInvoiceRequest) updates) =>
      super.copyWith((message) => updates(message as PreviewInvoiceRequest))
          as PreviewInvoiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use PreviewInvoiceRequest() / PreviewInvoiceRequest.new instead')
  static PreviewInvoiceRequest create() => PreviewInvoiceRequest._();
  static $pb.GeneratedMessage $_createMessage() => PreviewInvoiceRequest._();
  @$core.override
  PreviewInvoiceRequest createEmptyInstance() => PreviewInvoiceRequest._();
  @$core.pragma('dart2js:noInline')
  static PreviewInvoiceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PreviewInvoiceRequest>(
          PreviewInvoiceRequest.$_createMessage);
  static PreviewInvoiceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class PreviewInvoiceResponse extends $pb.GeneratedMessage {
  factory PreviewInvoiceResponse({
    $core.String? contentHash,
    $core.String? number,
    $core.List<$core.int>? pdf,
  }) {
    final result = PreviewInvoiceResponse._();
    if (contentHash != null) result.contentHash = contentHash;
    if (number != null) result.number = number;
    if (pdf != null) result.pdf = pdf;
    return result;
  }

  PreviewInvoiceResponse._();

  factory PreviewInvoiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PreviewInvoiceResponse()..mergeFromBuffer(data, registry);
  factory PreviewInvoiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      PreviewInvoiceResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'PreviewInvoiceResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: PreviewInvoiceResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'contentHash')
    ..aOS(2, _omitFieldNames ? '' : 'number')
    ..a<$core.List<$core.int>>(
        3, _omitFieldNames ? '' : 'pdf', $pb.PbFieldType.OY)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PreviewInvoiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  PreviewInvoiceResponse copyWith(
          void Function(PreviewInvoiceResponse) updates) =>
      super.copyWith((message) => updates(message as PreviewInvoiceResponse))
          as PreviewInvoiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use PreviewInvoiceResponse() / PreviewInvoiceResponse.new instead')
  static PreviewInvoiceResponse create() => PreviewInvoiceResponse._();
  static $pb.GeneratedMessage $_createMessage() => PreviewInvoiceResponse._();
  @$core.override
  PreviewInvoiceResponse createEmptyInstance() => PreviewInvoiceResponse._();
  @$core.pragma('dart2js:noInline')
  static PreviewInvoiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<PreviewInvoiceResponse>(
          PreviewInvoiceResponse.$_createMessage);
  static PreviewInvoiceResponse? _defaultInstance;

  /// What ApproveInvoice must be given.
  @$pb.TagNumber(1)
  $core.String get contentHash => $_getSZ(0);
  @$pb.TagNumber(1)
  set contentHash($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasContentHash() => $_has(0);
  @$pb.TagNumber(1)
  void clearContentHash() => $_clearField(1);

  /// The number the approval would take.
  @$pb.TagNumber(2)
  $core.String get number => $_getSZ(1);
  @$pb.TagNumber(2)
  set number($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasNumber() => $_has(1);
  @$pb.TagNumber(2)
  void clearNumber() => $_clearField(2);

  /// Watermarked PDF.
  @$pb.TagNumber(3)
  $core.List<$core.int> get pdf => $_getN(2);
  @$pb.TagNumber(3)
  set pdf($core.List<$core.int> value) => $_setBytes(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPdf() => $_has(2);
  @$pb.TagNumber(3)
  void clearPdf() => $_clearField(3);
}

class ApproveInvoiceRequest extends $pb.GeneratedMessage {
  factory ApproveInvoiceRequest({
    $core.String? id,
    $core.String? contentHash,
  }) {
    final result = ApproveInvoiceRequest._();
    if (id != null) result.id = id;
    if (contentHash != null) result.contentHash = contentHash;
    return result;
  }

  ApproveInvoiceRequest._();

  factory ApproveInvoiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ApproveInvoiceRequest()..mergeFromBuffer(data, registry);
  factory ApproveInvoiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ApproveInvoiceRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ApproveInvoiceRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ApproveInvoiceRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'contentHash')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApproveInvoiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApproveInvoiceRequest copyWith(
          void Function(ApproveInvoiceRequest) updates) =>
      super.copyWith((message) => updates(message as ApproveInvoiceRequest))
          as ApproveInvoiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ApproveInvoiceRequest() / ApproveInvoiceRequest.new instead')
  static ApproveInvoiceRequest create() => ApproveInvoiceRequest._();
  static $pb.GeneratedMessage $_createMessage() => ApproveInvoiceRequest._();
  @$core.override
  ApproveInvoiceRequest createEmptyInstance() => ApproveInvoiceRequest._();
  @$core.pragma('dart2js:noInline')
  static ApproveInvoiceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ApproveInvoiceRequest>(
          ApproveInvoiceRequest.$_createMessage);
  static ApproveInvoiceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get contentHash => $_getSZ(1);
  @$pb.TagNumber(2)
  set contentHash($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasContentHash() => $_has(1);
  @$pb.TagNumber(2)
  void clearContentHash() => $_clearField(2);
}

class ApproveInvoiceResponse extends $pb.GeneratedMessage {
  factory ApproveInvoiceResponse({
    Invoice? invoice,
  }) {
    final result = ApproveInvoiceResponse._();
    if (invoice != null) result.invoice = invoice;
    return result;
  }

  ApproveInvoiceResponse._();

  factory ApproveInvoiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ApproveInvoiceResponse()..mergeFromBuffer(data, registry);
  factory ApproveInvoiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ApproveInvoiceResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ApproveInvoiceResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ApproveInvoiceResponse.$_createMessage)
    ..aOM<Invoice>(1, _omitFieldNames ? '' : 'invoice',
        subBuilder: Invoice.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApproveInvoiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ApproveInvoiceResponse copyWith(
          void Function(ApproveInvoiceResponse) updates) =>
      super.copyWith((message) => updates(message as ApproveInvoiceResponse))
          as ApproveInvoiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ApproveInvoiceResponse() / ApproveInvoiceResponse.new instead')
  static ApproveInvoiceResponse create() => ApproveInvoiceResponse._();
  static $pb.GeneratedMessage $_createMessage() => ApproveInvoiceResponse._();
  @$core.override
  ApproveInvoiceResponse createEmptyInstance() => ApproveInvoiceResponse._();
  @$core.pragma('dart2js:noInline')
  static ApproveInvoiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ApproveInvoiceResponse>(
          ApproveInvoiceResponse.$_createMessage);
  static ApproveInvoiceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Invoice get invoice => $_getN(0);
  @$pb.TagNumber(1)
  set invoice(Invoice value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasInvoice() => $_has(0);
  @$pb.TagNumber(1)
  void clearInvoice() => $_clearField(1);
  @$pb.TagNumber(1)
  Invoice ensureInvoice() => $_ensure(0);
}

class CancelInvoiceRequest extends $pb.GeneratedMessage {
  factory CancelInvoiceRequest({
    $core.String? id,
    $core.String? reason,
  }) {
    final result = CancelInvoiceRequest._();
    if (id != null) result.id = id;
    if (reason != null) result.reason = reason;
    return result;
  }

  CancelInvoiceRequest._();

  factory CancelInvoiceRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CancelInvoiceRequest()..mergeFromBuffer(data, registry);
  factory CancelInvoiceRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CancelInvoiceRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CancelInvoiceRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: CancelInvoiceRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'reason')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelInvoiceRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelInvoiceRequest copyWith(void Function(CancelInvoiceRequest) updates) =>
      super.copyWith((message) => updates(message as CancelInvoiceRequest))
          as CancelInvoiceRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use CancelInvoiceRequest() / CancelInvoiceRequest.new instead')
  static CancelInvoiceRequest create() => CancelInvoiceRequest._();
  static $pb.GeneratedMessage $_createMessage() => CancelInvoiceRequest._();
  @$core.override
  CancelInvoiceRequest createEmptyInstance() => CancelInvoiceRequest._();
  @$core.pragma('dart2js:noInline')
  static CancelInvoiceRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CancelInvoiceRequest>(
          CancelInvoiceRequest.$_createMessage);
  static CancelInvoiceRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get reason => $_getSZ(1);
  @$pb.TagNumber(2)
  set reason($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasReason() => $_has(1);
  @$pb.TagNumber(2)
  void clearReason() => $_clearField(2);
}

class CancelInvoiceResponse extends $pb.GeneratedMessage {
  factory CancelInvoiceResponse({
    Invoice? invoice,
  }) {
    final result = CancelInvoiceResponse._();
    if (invoice != null) result.invoice = invoice;
    return result;
  }

  CancelInvoiceResponse._();

  factory CancelInvoiceResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CancelInvoiceResponse()..mergeFromBuffer(data, registry);
  factory CancelInvoiceResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CancelInvoiceResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CancelInvoiceResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: CancelInvoiceResponse.$_createMessage)
    ..aOM<Invoice>(1, _omitFieldNames ? '' : 'invoice',
        subBuilder: Invoice.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelInvoiceResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CancelInvoiceResponse copyWith(
          void Function(CancelInvoiceResponse) updates) =>
      super.copyWith((message) => updates(message as CancelInvoiceResponse))
          as CancelInvoiceResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use CancelInvoiceResponse() / CancelInvoiceResponse.new instead')
  static CancelInvoiceResponse create() => CancelInvoiceResponse._();
  static $pb.GeneratedMessage $_createMessage() => CancelInvoiceResponse._();
  @$core.override
  CancelInvoiceResponse createEmptyInstance() => CancelInvoiceResponse._();
  @$core.pragma('dart2js:noInline')
  static CancelInvoiceResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CancelInvoiceResponse>(
          CancelInvoiceResponse.$_createMessage);
  static CancelInvoiceResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Invoice get invoice => $_getN(0);
  @$pb.TagNumber(1)
  set invoice(Invoice value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasInvoice() => $_has(0);
  @$pb.TagNumber(1)
  void clearInvoice() => $_clearField(1);
  @$pb.TagNumber(1)
  Invoice ensureInvoice() => $_ensure(0);
}

class GetInvoiceDocumentRequest extends $pb.GeneratedMessage {
  factory GetInvoiceDocumentRequest({
    $core.String? id,
  }) {
    final result = GetInvoiceDocumentRequest._();
    if (id != null) result.id = id;
    return result;
  }

  GetInvoiceDocumentRequest._();

  factory GetInvoiceDocumentRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetInvoiceDocumentRequest()..mergeFromBuffer(data, registry);
  factory GetInvoiceDocumentRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetInvoiceDocumentRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetInvoiceDocumentRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetInvoiceDocumentRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInvoiceDocumentRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInvoiceDocumentRequest copyWith(
          void Function(GetInvoiceDocumentRequest) updates) =>
      super.copyWith((message) => updates(message as GetInvoiceDocumentRequest))
          as GetInvoiceDocumentRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use GetInvoiceDocumentRequest() / GetInvoiceDocumentRequest.new instead')
  static GetInvoiceDocumentRequest create() => GetInvoiceDocumentRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      GetInvoiceDocumentRequest._();
  @$core.override
  GetInvoiceDocumentRequest createEmptyInstance() =>
      GetInvoiceDocumentRequest._();
  @$core.pragma('dart2js:noInline')
  static GetInvoiceDocumentRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetInvoiceDocumentRequest>(
          GetInvoiceDocumentRequest.$_createMessage);
  static GetInvoiceDocumentRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetInvoiceDocumentResponse extends $pb.GeneratedMessage {
  factory GetInvoiceDocumentResponse({
    $core.String? contentType,
    $core.List<$core.int>? pdf,
    $core.String? number,
  }) {
    final result = GetInvoiceDocumentResponse._();
    if (contentType != null) result.contentType = contentType;
    if (pdf != null) result.pdf = pdf;
    if (number != null) result.number = number;
    return result;
  }

  GetInvoiceDocumentResponse._();

  factory GetInvoiceDocumentResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetInvoiceDocumentResponse()..mergeFromBuffer(data, registry);
  factory GetInvoiceDocumentResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetInvoiceDocumentResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetInvoiceDocumentResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetInvoiceDocumentResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'contentType')
    ..a<$core.List<$core.int>>(
        2, _omitFieldNames ? '' : 'pdf', $pb.PbFieldType.OY)
    ..aOS(3, _omitFieldNames ? '' : 'number')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInvoiceDocumentResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetInvoiceDocumentResponse copyWith(
          void Function(GetInvoiceDocumentResponse) updates) =>
      super.copyWith(
              (message) => updates(message as GetInvoiceDocumentResponse))
          as GetInvoiceDocumentResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use GetInvoiceDocumentResponse() / GetInvoiceDocumentResponse.new instead')
  static GetInvoiceDocumentResponse create() => GetInvoiceDocumentResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      GetInvoiceDocumentResponse._();
  @$core.override
  GetInvoiceDocumentResponse createEmptyInstance() =>
      GetInvoiceDocumentResponse._();
  @$core.pragma('dart2js:noInline')
  static GetInvoiceDocumentResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetInvoiceDocumentResponse>(
          GetInvoiceDocumentResponse.$_createMessage);
  static GetInvoiceDocumentResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get contentType => $_getSZ(0);
  @$pb.TagNumber(1)
  set contentType($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasContentType() => $_has(0);
  @$pb.TagNumber(1)
  void clearContentType() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.List<$core.int> get pdf => $_getN(1);
  @$pb.TagNumber(2)
  set pdf($core.List<$core.int> value) => $_setBytes(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPdf() => $_has(1);
  @$pb.TagNumber(2)
  void clearPdf() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get number => $_getSZ(2);
  @$pb.TagNumber(3)
  set number($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasNumber() => $_has(2);
  @$pb.TagNumber(3)
  void clearNumber() => $_clearField(3);
}

class MonthlySummaryRequest extends $pb.GeneratedMessage {
  factory MonthlySummaryRequest({
    $core.Iterable<$core.String>? partyIds,
    $core.bool? includeInternal,
    $core.String? fromMonth,
  }) {
    final result = MonthlySummaryRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    if (includeInternal != null) result.includeInternal = includeInternal;
    if (fromMonth != null) result.fromMonth = fromMonth;
    return result;
  }

  MonthlySummaryRequest._();

  factory MonthlySummaryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MonthlySummaryRequest()..mergeFromBuffer(data, registry);
  factory MonthlySummaryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MonthlySummaryRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MonthlySummaryRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: MonthlySummaryRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..aOB(2, _omitFieldNames ? '' : 'includeInternal')
    ..aOS(3, _omitFieldNames ? '' : 'fromMonth')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonthlySummaryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonthlySummaryRequest copyWith(
          void Function(MonthlySummaryRequest) updates) =>
      super.copyWith((message) => updates(message as MonthlySummaryRequest))
          as MonthlySummaryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use MonthlySummaryRequest() / MonthlySummaryRequest.new instead')
  static MonthlySummaryRequest create() => MonthlySummaryRequest._();
  static $pb.GeneratedMessage $_createMessage() => MonthlySummaryRequest._();
  @$core.override
  MonthlySummaryRequest createEmptyInstance() => MonthlySummaryRequest._();
  @$core.pragma('dart2js:noInline')
  static MonthlySummaryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MonthlySummaryRequest>(
          MonthlySummaryRequest.$_createMessage);
  static MonthlySummaryRequest? _defaultInstance;

  /// Narrow to these parties; same rule as ListTransactions.
  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);

  /// Count transfers between the caller's own accounts. Off by default.
  @$pb.TagNumber(2)
  $core.bool get includeInternal => $_getBF(1);
  @$pb.TagNumber(2)
  set includeInternal($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasIncludeInternal() => $_has(1);
  @$pb.TagNumber(2)
  void clearIncludeInternal() => $_clearField(2);

  /// Earliest month, YYYY-MM. Empty means everything.
  @$pb.TagNumber(3)
  $core.String get fromMonth => $_getSZ(2);
  @$pb.TagNumber(3)
  set fromMonth($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasFromMonth() => $_has(2);
  @$pb.TagNumber(3)
  void clearFromMonth() => $_clearField(3);
}

class MonthlySummaryResponse extends $pb.GeneratedMessage {
  factory MonthlySummaryResponse({
    $core.Iterable<SummaryRow>? rows,
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = MonthlySummaryResponse._();
    if (rows != null) result.rows.addAll(rows);
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  MonthlySummaryResponse._();

  factory MonthlySummaryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MonthlySummaryResponse()..mergeFromBuffer(data, registry);
  factory MonthlySummaryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      MonthlySummaryResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'MonthlySummaryResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: MonthlySummaryResponse.$_createMessage)
    ..pPM<SummaryRow>(1, _omitFieldNames ? '' : 'rows',
        subBuilder: SummaryRow.$_createMessage)
    ..pPS(2, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonthlySummaryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  MonthlySummaryResponse copyWith(
          void Function(MonthlySummaryResponse) updates) =>
      super.copyWith((message) => updates(message as MonthlySummaryResponse))
          as MonthlySummaryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use MonthlySummaryResponse() / MonthlySummaryResponse.new instead')
  static MonthlySummaryResponse create() => MonthlySummaryResponse._();
  static $pb.GeneratedMessage $_createMessage() => MonthlySummaryResponse._();
  @$core.override
  MonthlySummaryResponse createEmptyInstance() => MonthlySummaryResponse._();
  @$core.pragma('dart2js:noInline')
  static MonthlySummaryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<MonthlySummaryResponse>(
          MonthlySummaryResponse.$_createMessage);
  static MonthlySummaryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<SummaryRow> get rows => $_getList(0);

  @$pb.TagNumber(2)
  $pb.PbList<$core.String> get partyIds => $_getList(1);
}

class SummaryRow extends $pb.GeneratedMessage {
  factory SummaryRow({
    $core.String? month,
    $core.String? partyId,
    $core.String? categoryId,
    $core.String? category,
    $core.String? kind,
    $core.String? currency,
    $fixnum.Int64? totalMinor,
    $core.int? count,
    $core.bool? internal,
  }) {
    final result = SummaryRow._();
    if (month != null) result.month = month;
    if (partyId != null) result.partyId = partyId;
    if (categoryId != null) result.categoryId = categoryId;
    if (category != null) result.category = category;
    if (kind != null) result.kind = kind;
    if (currency != null) result.currency = currency;
    if (totalMinor != null) result.totalMinor = totalMinor;
    if (count != null) result.count = count;
    if (internal != null) result.internal = internal;
    return result;
  }

  SummaryRow._();

  factory SummaryRow.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SummaryRow()..mergeFromBuffer(data, registry);
  factory SummaryRow.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SummaryRow()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SummaryRow',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SummaryRow.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'month')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'categoryId')
    ..aOS(4, _omitFieldNames ? '' : 'category')
    ..aOS(5, _omitFieldNames ? '' : 'kind')
    ..aOS(6, _omitFieldNames ? '' : 'currency')
    ..aInt64(7, _omitFieldNames ? '' : 'totalMinor')
    ..aI(8, _omitFieldNames ? '' : 'count', fieldType: $pb.PbFieldType.OU3)
    ..aOB(9, _omitFieldNames ? '' : 'internal')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SummaryRow clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SummaryRow copyWith(void Function(SummaryRow) updates) =>
      super.copyWith((message) => updates(message as SummaryRow)) as SummaryRow;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use SummaryRow() / SummaryRow.new instead')
  static SummaryRow create() => SummaryRow._();
  static $pb.GeneratedMessage $_createMessage() => SummaryRow._();
  @$core.override
  SummaryRow createEmptyInstance() => SummaryRow._();
  @$core.pragma('dart2js:noInline')
  static SummaryRow getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SummaryRow>(SummaryRow.$_createMessage);
  static SummaryRow? _defaultInstance;

  /// YYYY-MM.
  @$pb.TagNumber(1)
  $core.String get month => $_getSZ(0);
  @$pb.TagNumber(1)
  set month($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasMonth() => $_has(0);
  @$pb.TagNumber(1)
  void clearMonth() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  /// Empty when nothing has claimed the rows yet.
  @$pb.TagNumber(3)
  $core.String get categoryId => $_getSZ(2);
  @$pb.TagNumber(3)
  set categoryId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCategoryId() => $_has(2);
  @$pb.TagNumber(3)
  void clearCategoryId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get category => $_getSZ(3);
  @$pb.TagNumber(4)
  set category($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCategory() => $_has(3);
  @$pb.TagNumber(4)
  void clearCategory() => $_clearField(4);

  /// expense, income, transfer, tax, capital; empty when uncategorised.
  @$pb.TagNumber(5)
  $core.String get kind => $_getSZ(4);
  @$pb.TagNumber(5)
  set kind($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasKind() => $_has(4);
  @$pb.TagNumber(5)
  void clearKind() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get currency => $_getSZ(5);
  @$pb.TagNumber(6)
  set currency($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCurrency() => $_has(5);
  @$pb.TagNumber(6)
  void clearCurrency() => $_clearField(6);

  /// Signed minor units; JSON string.
  @$pb.TagNumber(7)
  $fixnum.Int64 get totalMinor => $_getI64(6);
  @$pb.TagNumber(7)
  set totalMinor($fixnum.Int64 value) => $_setInt64(6, value);
  @$pb.TagNumber(7)
  $core.bool hasTotalMinor() => $_has(6);
  @$pb.TagNumber(7)
  void clearTotalMinor() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.int get count => $_getIZ(7);
  @$pb.TagNumber(8)
  set count($core.int value) => $_setUnsignedInt32(7, value);
  @$pb.TagNumber(8)
  $core.bool hasCount() => $_has(7);
  @$pb.TagNumber(8)
  void clearCount() => $_clearField(8);

  /// Rows that are transfers between the caller's own accounts. Only present
  /// when the request asked to include them; a single party's view shows an
  /// owner draw as money in, a combined view leaves it out.
  @$pb.TagNumber(9)
  $core.bool get internal => $_getBF(8);
  @$pb.TagNumber(9)
  set internal($core.bool value) => $_setBool(8, value);
  @$pb.TagNumber(9)
  $core.bool hasInternal() => $_has(8);
  @$pb.TagNumber(9)
  void clearInternal() => $_clearField(9);
}

class ListPartiesRequest extends $pb.GeneratedMessage {
  factory ListPartiesRequest() => ListPartiesRequest._();

  ListPartiesRequest._();

  factory ListPartiesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListPartiesRequest()..mergeFromBuffer(data, registry);
  factory ListPartiesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListPartiesRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListPartiesRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListPartiesRequest.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListPartiesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListPartiesRequest copyWith(void Function(ListPartiesRequest) updates) =>
      super.copyWith((message) => updates(message as ListPartiesRequest))
          as ListPartiesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ListPartiesRequest() / ListPartiesRequest.new instead')
  static ListPartiesRequest create() => ListPartiesRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListPartiesRequest._();
  @$core.override
  ListPartiesRequest createEmptyInstance() => ListPartiesRequest._();
  @$core.pragma('dart2js:noInline')
  static ListPartiesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListPartiesRequest>(
          ListPartiesRequest.$_createMessage);
  static ListPartiesRequest? _defaultInstance;
}

class ListPartiesResponse extends $pb.GeneratedMessage {
  factory ListPartiesResponse({
    $core.Iterable<Party>? parties,
  }) {
    final result = ListPartiesResponse._();
    if (parties != null) result.parties.addAll(parties);
    return result;
  }

  ListPartiesResponse._();

  factory ListPartiesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListPartiesResponse()..mergeFromBuffer(data, registry);
  factory ListPartiesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListPartiesResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListPartiesResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListPartiesResponse.$_createMessage)
    ..pPM<Party>(1, _omitFieldNames ? '' : 'parties',
        subBuilder: Party.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListPartiesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListPartiesResponse copyWith(void Function(ListPartiesResponse) updates) =>
      super.copyWith((message) => updates(message as ListPartiesResponse))
          as ListPartiesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use ListPartiesResponse() / ListPartiesResponse.new instead')
  static ListPartiesResponse create() => ListPartiesResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListPartiesResponse._();
  @$core.override
  ListPartiesResponse createEmptyInstance() => ListPartiesResponse._();
  @$core.pragma('dart2js:noInline')
  static ListPartiesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListPartiesResponse>(
          ListPartiesResponse.$_createMessage);
  static ListPartiesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Party> get parties => $_getList(0);
}

class Party extends $pb.GeneratedMessage {
  factory Party({
    $core.String? id,
    $core.String? kind,
    $core.String? displayName,
    $core.String? capability,
  }) {
    final result = Party._();
    if (id != null) result.id = id;
    if (kind != null) result.kind = kind;
    if (displayName != null) result.displayName = displayName;
    if (capability != null) result.capability = capability;
    return result;
  }

  Party._();

  factory Party.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Party()..mergeFromBuffer(data, registry);
  factory Party.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Party()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Party',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Party.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'kind')
    ..aOS(3, _omitFieldNames ? '' : 'displayName')
    ..aOS(4, _omitFieldNames ? '' : 'capability')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Party clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Party copyWith(void Function(Party) updates) =>
      super.copyWith((message) => updates(message as Party)) as Party;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Party() / Party.new instead')
  static Party create() => Party._();
  static $pb.GeneratedMessage $_createMessage() => Party._();
  @$core.override
  Party createEmptyInstance() => Party._();
  @$core.pragma('dart2js:noInline')
  static Party getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Party>(Party.$_createMessage);
  static Party? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  /// person or org.
  @$pb.TagNumber(2)
  $core.String get kind => $_getSZ(1);
  @$pb.TagNumber(2)
  set kind($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasKind() => $_has(1);
  @$pb.TagNumber(2)
  void clearKind() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get displayName => $_getSZ(2);
  @$pb.TagNumber(3)
  set displayName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasDisplayName() => $_has(2);
  @$pb.TagNumber(3)
  void clearDisplayName() => $_clearField(3);

  /// own or read.
  @$pb.TagNumber(4)
  $core.String get capability => $_getSZ(3);
  @$pb.TagNumber(4)
  set capability($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCapability() => $_has(3);
  @$pb.TagNumber(4)
  void clearCapability() => $_clearField(4);
}

class ListAccountsRequest extends $pb.GeneratedMessage {
  factory ListAccountsRequest({
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = ListAccountsRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  ListAccountsRequest._();

  factory ListAccountsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListAccountsRequest()..mergeFromBuffer(data, registry);
  factory ListAccountsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListAccountsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListAccountsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListAccountsRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListAccountsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListAccountsRequest copyWith(void Function(ListAccountsRequest) updates) =>
      super.copyWith((message) => updates(message as ListAccountsRequest))
          as ListAccountsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core
      .Deprecated('Use ListAccountsRequest() / ListAccountsRequest.new instead')
  static ListAccountsRequest create() => ListAccountsRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListAccountsRequest._();
  @$core.override
  ListAccountsRequest createEmptyInstance() => ListAccountsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListAccountsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListAccountsRequest>(
          ListAccountsRequest.$_createMessage);
  static ListAccountsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);
}

class ListAccountsResponse extends $pb.GeneratedMessage {
  factory ListAccountsResponse({
    $core.Iterable<Account>? accounts,
  }) {
    final result = ListAccountsResponse._();
    if (accounts != null) result.accounts.addAll(accounts);
    return result;
  }

  ListAccountsResponse._();

  factory ListAccountsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListAccountsResponse()..mergeFromBuffer(data, registry);
  factory ListAccountsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListAccountsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListAccountsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListAccountsResponse.$_createMessage)
    ..pPM<Account>(1, _omitFieldNames ? '' : 'accounts',
        subBuilder: Account.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListAccountsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListAccountsResponse copyWith(void Function(ListAccountsResponse) updates) =>
      super.copyWith((message) => updates(message as ListAccountsResponse))
          as ListAccountsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListAccountsResponse() / ListAccountsResponse.new instead')
  static ListAccountsResponse create() => ListAccountsResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListAccountsResponse._();
  @$core.override
  ListAccountsResponse createEmptyInstance() => ListAccountsResponse._();
  @$core.pragma('dart2js:noInline')
  static ListAccountsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListAccountsResponse>(
          ListAccountsResponse.$_createMessage);
  static ListAccountsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Account> get accounts => $_getList(0);
}

class Account extends $pb.GeneratedMessage {
  factory Account({
    $core.String? id,
    $core.String? partyId,
    $core.String? connectionId,
    $core.String? provider,
    $core.String? iban,
    $core.String? currency,
    $core.String? name,
    $core.bool? syncEnabled,
    $core.String? lastSyncedAt,
    $core.String? lastSyncStatus,
    $core.String? lastSyncError,
    $core.String? lastBookedThrough,
    $core.String? syncBackoffUntil,
    $core.int? syncBudgetUsed,
    $core.Iterable<Balance>? balances,
  }) {
    final result = Account._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (connectionId != null) result.connectionId = connectionId;
    if (provider != null) result.provider = provider;
    if (iban != null) result.iban = iban;
    if (currency != null) result.currency = currency;
    if (name != null) result.name = name;
    if (syncEnabled != null) result.syncEnabled = syncEnabled;
    if (lastSyncedAt != null) result.lastSyncedAt = lastSyncedAt;
    if (lastSyncStatus != null) result.lastSyncStatus = lastSyncStatus;
    if (lastSyncError != null) result.lastSyncError = lastSyncError;
    if (lastBookedThrough != null) result.lastBookedThrough = lastBookedThrough;
    if (syncBackoffUntil != null) result.syncBackoffUntil = syncBackoffUntil;
    if (syncBudgetUsed != null) result.syncBudgetUsed = syncBudgetUsed;
    if (balances != null) result.balances.addAll(balances);
    return result;
  }

  Account._();

  factory Account.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Account()..mergeFromBuffer(data, registry);
  factory Account.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Account()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Account',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Account.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'connectionId')
    ..aOS(4, _omitFieldNames ? '' : 'provider')
    ..aOS(5, _omitFieldNames ? '' : 'iban')
    ..aOS(6, _omitFieldNames ? '' : 'currency')
    ..aOS(7, _omitFieldNames ? '' : 'name')
    ..aOB(8, _omitFieldNames ? '' : 'syncEnabled')
    ..aOS(9, _omitFieldNames ? '' : 'lastSyncedAt')
    ..aOS(10, _omitFieldNames ? '' : 'lastSyncStatus')
    ..aOS(11, _omitFieldNames ? '' : 'lastSyncError')
    ..aOS(12, _omitFieldNames ? '' : 'lastBookedThrough')
    ..aOS(13, _omitFieldNames ? '' : 'syncBackoffUntil')
    ..aI(14, _omitFieldNames ? '' : 'syncBudgetUsed',
        fieldType: $pb.PbFieldType.OU3)
    ..pPM<Balance>(15, _omitFieldNames ? '' : 'balances',
        subBuilder: Balance.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Account clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Account copyWith(void Function(Account) updates) =>
      super.copyWith((message) => updates(message as Account)) as Account;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Account() / Account.new instead')
  static Account create() => Account._();
  static $pb.GeneratedMessage $_createMessage() => Account._();
  @$core.override
  Account createEmptyInstance() => Account._();
  @$core.pragma('dart2js:noInline')
  static Account getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Account>(Account.$_createMessage);
  static Account? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get connectionId => $_getSZ(2);
  @$pb.TagNumber(3)
  set connectionId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasConnectionId() => $_has(2);
  @$pb.TagNumber(3)
  void clearConnectionId() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get provider => $_getSZ(3);
  @$pb.TagNumber(4)
  set provider($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasProvider() => $_has(3);
  @$pb.TagNumber(4)
  void clearProvider() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get iban => $_getSZ(4);
  @$pb.TagNumber(5)
  set iban($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasIban() => $_has(4);
  @$pb.TagNumber(5)
  void clearIban() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get currency => $_getSZ(5);
  @$pb.TagNumber(6)
  set currency($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCurrency() => $_has(5);
  @$pb.TagNumber(6)
  void clearCurrency() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get name => $_getSZ(6);
  @$pb.TagNumber(7)
  set name($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasName() => $_has(6);
  @$pb.TagNumber(7)
  void clearName() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.bool get syncEnabled => $_getBF(7);
  @$pb.TagNumber(8)
  set syncEnabled($core.bool value) => $_setBool(7, value);
  @$pb.TagNumber(8)
  $core.bool hasSyncEnabled() => $_has(7);
  @$pb.TagNumber(8)
  void clearSyncEnabled() => $_clearField(8);

  /// RFC 3339, empty when never.
  @$pb.TagNumber(9)
  $core.String get lastSyncedAt => $_getSZ(8);
  @$pb.TagNumber(9)
  set lastSyncedAt($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasLastSyncedAt() => $_has(8);
  @$pb.TagNumber(9)
  void clearLastSyncedAt() => $_clearField(9);

  /// ok, rate_limited, consent_invalid, transport, error; empty when never.
  @$pb.TagNumber(10)
  $core.String get lastSyncStatus => $_getSZ(9);
  @$pb.TagNumber(10)
  set lastSyncStatus($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasLastSyncStatus() => $_has(9);
  @$pb.TagNumber(10)
  void clearLastSyncStatus() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.String get lastSyncError => $_getSZ(10);
  @$pb.TagNumber(11)
  set lastSyncError($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasLastSyncError() => $_has(10);
  @$pb.TagNumber(11)
  void clearLastSyncError() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get lastBookedThrough => $_getSZ(11);
  @$pb.TagNumber(12)
  set lastBookedThrough($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasLastBookedThrough() => $_has(11);
  @$pb.TagNumber(12)
  void clearLastBookedThrough() => $_clearField(12);

  /// RFC 3339, empty when not backing off.
  @$pb.TagNumber(13)
  $core.String get syncBackoffUntil => $_getSZ(12);
  @$pb.TagNumber(13)
  set syncBackoffUntil($core.String value) => $_setString(12, value);
  @$pb.TagNumber(13)
  $core.bool hasSyncBackoffUntil() => $_has(12);
  @$pb.TagNumber(13)
  void clearSyncBackoffUntil() => $_clearField(13);

  @$pb.TagNumber(14)
  $core.int get syncBudgetUsed => $_getIZ(13);
  @$pb.TagNumber(14)
  set syncBudgetUsed($core.int value) => $_setUnsignedInt32(13, value);
  @$pb.TagNumber(14)
  $core.bool hasSyncBudgetUsed() => $_has(13);
  @$pb.TagNumber(14)
  void clearSyncBudgetUsed() => $_clearField(14);

  /// Latest balance the bank reported, minor units, per balance type.
  @$pb.TagNumber(15)
  $pb.PbList<Balance> get balances => $_getList(14);
}

class Balance extends $pb.GeneratedMessage {
  factory Balance({
    $core.String? balanceType,
    $fixnum.Int64? amountMinor,
    $core.String? currency,
    $core.String? observedAt,
  }) {
    final result = Balance._();
    if (balanceType != null) result.balanceType = balanceType;
    if (amountMinor != null) result.amountMinor = amountMinor;
    if (currency != null) result.currency = currency;
    if (observedAt != null) result.observedAt = observedAt;
    return result;
  }

  Balance._();

  factory Balance.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Balance()..mergeFromBuffer(data, registry);
  factory Balance.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Balance()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Balance',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Balance.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'balanceType')
    ..aInt64(2, _omitFieldNames ? '' : 'amountMinor')
    ..aOS(3, _omitFieldNames ? '' : 'currency')
    ..aOS(4, _omitFieldNames ? '' : 'observedAt')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Balance clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Balance copyWith(void Function(Balance) updates) =>
      super.copyWith((message) => updates(message as Balance)) as Balance;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Balance() / Balance.new instead')
  static Balance create() => Balance._();
  static $pb.GeneratedMessage $_createMessage() => Balance._();
  @$core.override
  Balance createEmptyInstance() => Balance._();
  @$core.pragma('dart2js:noInline')
  static Balance getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Balance>(Balance.$_createMessage);
  static Balance? _defaultInstance;

  /// The bank's type: CLBD, ITAV, ...
  @$pb.TagNumber(1)
  $core.String get balanceType => $_getSZ(0);
  @$pb.TagNumber(1)
  set balanceType($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasBalanceType() => $_has(0);
  @$pb.TagNumber(1)
  void clearBalanceType() => $_clearField(1);

  @$pb.TagNumber(2)
  $fixnum.Int64 get amountMinor => $_getI64(1);
  @$pb.TagNumber(2)
  set amountMinor($fixnum.Int64 value) => $_setInt64(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAmountMinor() => $_has(1);
  @$pb.TagNumber(2)
  void clearAmountMinor() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get currency => $_getSZ(2);
  @$pb.TagNumber(3)
  set currency($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasCurrency() => $_has(2);
  @$pb.TagNumber(3)
  void clearCurrency() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get observedAt => $_getSZ(3);
  @$pb.TagNumber(4)
  set observedAt($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasObservedAt() => $_has(3);
  @$pb.TagNumber(4)
  void clearObservedAt() => $_clearField(4);
}

class RefreshAccountRequest extends $pb.GeneratedMessage {
  factory RefreshAccountRequest({
    $core.String? accountId,
  }) {
    final result = RefreshAccountRequest._();
    if (accountId != null) result.accountId = accountId;
    return result;
  }

  RefreshAccountRequest._();

  factory RefreshAccountRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RefreshAccountRequest()..mergeFromBuffer(data, registry);
  factory RefreshAccountRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RefreshAccountRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RefreshAccountRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: RefreshAccountRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'accountId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RefreshAccountRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RefreshAccountRequest copyWith(
          void Function(RefreshAccountRequest) updates) =>
      super.copyWith((message) => updates(message as RefreshAccountRequest))
          as RefreshAccountRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use RefreshAccountRequest() / RefreshAccountRequest.new instead')
  static RefreshAccountRequest create() => RefreshAccountRequest._();
  static $pb.GeneratedMessage $_createMessage() => RefreshAccountRequest._();
  @$core.override
  RefreshAccountRequest createEmptyInstance() => RefreshAccountRequest._();
  @$core.pragma('dart2js:noInline')
  static RefreshAccountRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RefreshAccountRequest>(
          RefreshAccountRequest.$_createMessage);
  static RefreshAccountRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get accountId => $_getSZ(0);
  @$pb.TagNumber(1)
  set accountId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAccountId() => $_has(0);
  @$pb.TagNumber(1)
  void clearAccountId() => $_clearField(1);
}

class RefreshAccountResponse extends $pb.GeneratedMessage {
  factory RefreshAccountResponse({
    $core.String? outcome,
    $core.String? skipped,
    $core.int? inserted,
    $core.int? booked,
    $core.int? duplicates,
    $core.bool? attended,
  }) {
    final result = RefreshAccountResponse._();
    if (outcome != null) result.outcome = outcome;
    if (skipped != null) result.skipped = skipped;
    if (inserted != null) result.inserted = inserted;
    if (booked != null) result.booked = booked;
    if (duplicates != null) result.duplicates = duplicates;
    if (attended != null) result.attended = attended;
    return result;
  }

  RefreshAccountResponse._();

  factory RefreshAccountResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RefreshAccountResponse()..mergeFromBuffer(data, registry);
  factory RefreshAccountResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      RefreshAccountResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'RefreshAccountResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: RefreshAccountResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'outcome')
    ..aOS(2, _omitFieldNames ? '' : 'skipped')
    ..aI(3, _omitFieldNames ? '' : 'inserted', fieldType: $pb.PbFieldType.OU3)
    ..aI(4, _omitFieldNames ? '' : 'booked', fieldType: $pb.PbFieldType.OU3)
    ..aI(5, _omitFieldNames ? '' : 'duplicates', fieldType: $pb.PbFieldType.OU3)
    ..aOB(6, _omitFieldNames ? '' : 'attended')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RefreshAccountResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  RefreshAccountResponse copyWith(
          void Function(RefreshAccountResponse) updates) =>
      super.copyWith((message) => updates(message as RefreshAccountResponse))
          as RefreshAccountResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use RefreshAccountResponse() / RefreshAccountResponse.new instead')
  static RefreshAccountResponse create() => RefreshAccountResponse._();
  static $pb.GeneratedMessage $_createMessage() => RefreshAccountResponse._();
  @$core.override
  RefreshAccountResponse createEmptyInstance() => RefreshAccountResponse._();
  @$core.pragma('dart2js:noInline')
  static RefreshAccountResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<RefreshAccountResponse>(
          RefreshAccountResponse.$_createMessage);
  static RefreshAccountResponse? _defaultInstance;

  /// ok, rate_limited, consent_invalid, transport, error; or skipped.
  @$pb.TagNumber(1)
  $core.String get outcome => $_getSZ(0);
  @$pb.TagNumber(1)
  set outcome($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasOutcome() => $_has(0);
  @$pb.TagNumber(1)
  void clearOutcome() => $_clearField(1);

  /// When skipped: budget_spent, backing_off, no_consent, busy.
  @$pb.TagNumber(2)
  $core.String get skipped => $_getSZ(1);
  @$pb.TagNumber(2)
  set skipped($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasSkipped() => $_has(1);
  @$pb.TagNumber(2)
  void clearSkipped() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get inserted => $_getIZ(2);
  @$pb.TagNumber(3)
  set inserted($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasInserted() => $_has(2);
  @$pb.TagNumber(3)
  void clearInserted() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.int get booked => $_getIZ(3);
  @$pb.TagNumber(4)
  set booked($core.int value) => $_setUnsignedInt32(3, value);
  @$pb.TagNumber(4)
  $core.bool hasBooked() => $_has(3);
  @$pb.TagNumber(4)
  void clearBooked() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.int get duplicates => $_getIZ(4);
  @$pb.TagNumber(5)
  set duplicates($core.int value) => $_setUnsignedInt32(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDuplicates() => $_has(4);
  @$pb.TagNumber(5)
  void clearDuplicates() => $_clearField(5);

  /// The call carried the person's address, so the bank did not count it
  /// against the unattended allowance.
  @$pb.TagNumber(6)
  $core.bool get attended => $_getBF(5);
  @$pb.TagNumber(6)
  set attended($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasAttended() => $_has(5);
  @$pb.TagNumber(6)
  void clearAttended() => $_clearField(6);
}

class SetAccountSyncRequest extends $pb.GeneratedMessage {
  factory SetAccountSyncRequest({
    $core.String? accountId,
    $core.bool? enabled,
  }) {
    final result = SetAccountSyncRequest._();
    if (accountId != null) result.accountId = accountId;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  SetAccountSyncRequest._();

  factory SetAccountSyncRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetAccountSyncRequest()..mergeFromBuffer(data, registry);
  factory SetAccountSyncRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetAccountSyncRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SetAccountSyncRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SetAccountSyncRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'accountId')
    ..aOB(2, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetAccountSyncRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetAccountSyncRequest copyWith(
          void Function(SetAccountSyncRequest) updates) =>
      super.copyWith((message) => updates(message as SetAccountSyncRequest))
          as SetAccountSyncRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use SetAccountSyncRequest() / SetAccountSyncRequest.new instead')
  static SetAccountSyncRequest create() => SetAccountSyncRequest._();
  static $pb.GeneratedMessage $_createMessage() => SetAccountSyncRequest._();
  @$core.override
  SetAccountSyncRequest createEmptyInstance() => SetAccountSyncRequest._();
  @$core.pragma('dart2js:noInline')
  static SetAccountSyncRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetAccountSyncRequest>(
          SetAccountSyncRequest.$_createMessage);
  static SetAccountSyncRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get accountId => $_getSZ(0);
  @$pb.TagNumber(1)
  set accountId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasAccountId() => $_has(0);
  @$pb.TagNumber(1)
  void clearAccountId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.bool get enabled => $_getBF(1);
  @$pb.TagNumber(2)
  set enabled($core.bool value) => $_setBool(1, value);
  @$pb.TagNumber(2)
  $core.bool hasEnabled() => $_has(1);
  @$pb.TagNumber(2)
  void clearEnabled() => $_clearField(2);
}

class SetAccountSyncResponse extends $pb.GeneratedMessage {
  factory SetAccountSyncResponse({
    Account? account,
  }) {
    final result = SetAccountSyncResponse._();
    if (account != null) result.account = account;
    return result;
  }

  SetAccountSyncResponse._();

  factory SetAccountSyncResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetAccountSyncResponse()..mergeFromBuffer(data, registry);
  factory SetAccountSyncResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      SetAccountSyncResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'SetAccountSyncResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: SetAccountSyncResponse.$_createMessage)
    ..aOM<Account>(1, _omitFieldNames ? '' : 'account',
        subBuilder: Account.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetAccountSyncResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  SetAccountSyncResponse copyWith(
          void Function(SetAccountSyncResponse) updates) =>
      super.copyWith((message) => updates(message as SetAccountSyncResponse))
          as SetAccountSyncResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use SetAccountSyncResponse() / SetAccountSyncResponse.new instead')
  static SetAccountSyncResponse create() => SetAccountSyncResponse._();
  static $pb.GeneratedMessage $_createMessage() => SetAccountSyncResponse._();
  @$core.override
  SetAccountSyncResponse createEmptyInstance() => SetAccountSyncResponse._();
  @$core.pragma('dart2js:noInline')
  static SetAccountSyncResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<SetAccountSyncResponse>(
          SetAccountSyncResponse.$_createMessage);
  static SetAccountSyncResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Account get account => $_getN(0);
  @$pb.TagNumber(1)
  set account(Account value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasAccount() => $_has(0);
  @$pb.TagNumber(1)
  void clearAccount() => $_clearField(1);
  @$pb.TagNumber(1)
  Account ensureAccount() => $_ensure(0);
}

class ListCategoriesRequest extends $pb.GeneratedMessage {
  factory ListCategoriesRequest({
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = ListCategoriesRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  ListCategoriesRequest._();

  factory ListCategoriesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListCategoriesRequest()..mergeFromBuffer(data, registry);
  factory ListCategoriesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListCategoriesRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListCategoriesRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListCategoriesRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCategoriesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCategoriesRequest copyWith(
          void Function(ListCategoriesRequest) updates) =>
      super.copyWith((message) => updates(message as ListCategoriesRequest))
          as ListCategoriesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListCategoriesRequest() / ListCategoriesRequest.new instead')
  static ListCategoriesRequest create() => ListCategoriesRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListCategoriesRequest._();
  @$core.override
  ListCategoriesRequest createEmptyInstance() => ListCategoriesRequest._();
  @$core.pragma('dart2js:noInline')
  static ListCategoriesRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListCategoriesRequest>(
          ListCategoriesRequest.$_createMessage);
  static ListCategoriesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);
}

class ListCategoriesResponse extends $pb.GeneratedMessage {
  factory ListCategoriesResponse({
    $core.Iterable<Category>? categories,
  }) {
    final result = ListCategoriesResponse._();
    if (categories != null) result.categories.addAll(categories);
    return result;
  }

  ListCategoriesResponse._();

  factory ListCategoriesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListCategoriesResponse()..mergeFromBuffer(data, registry);
  factory ListCategoriesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListCategoriesResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListCategoriesResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListCategoriesResponse.$_createMessage)
    ..pPM<Category>(1, _omitFieldNames ? '' : 'categories',
        subBuilder: Category.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCategoriesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListCategoriesResponse copyWith(
          void Function(ListCategoriesResponse) updates) =>
      super.copyWith((message) => updates(message as ListCategoriesResponse))
          as ListCategoriesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListCategoriesResponse() / ListCategoriesResponse.new instead')
  static ListCategoriesResponse create() => ListCategoriesResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListCategoriesResponse._();
  @$core.override
  ListCategoriesResponse createEmptyInstance() => ListCategoriesResponse._();
  @$core.pragma('dart2js:noInline')
  static ListCategoriesResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListCategoriesResponse>(
          ListCategoriesResponse.$_createMessage);
  static ListCategoriesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Category> get categories => $_getList(0);
}

class Category extends $pb.GeneratedMessage {
  factory Category({
    $core.String? id,
    $core.String? partyId,
    $core.String? slug,
    $core.String? name,
    $core.String? kind,
    $core.bool? deductible,
    $core.bool? archived,
  }) {
    final result = Category._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (slug != null) result.slug = slug;
    if (name != null) result.name = name;
    if (kind != null) result.kind = kind;
    if (deductible != null) result.deductible = deductible;
    if (archived != null) result.archived = archived;
    return result;
  }

  Category._();

  factory Category.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Category()..mergeFromBuffer(data, registry);
  factory Category.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Category()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Category',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Category.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'slug')
    ..aOS(4, _omitFieldNames ? '' : 'name')
    ..aOS(5, _omitFieldNames ? '' : 'kind')
    ..aOB(6, _omitFieldNames ? '' : 'deductible')
    ..aOB(7, _omitFieldNames ? '' : 'archived')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Category clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Category copyWith(void Function(Category) updates) =>
      super.copyWith((message) => updates(message as Category)) as Category;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Category() / Category.new instead')
  static Category create() => Category._();
  static $pb.GeneratedMessage $_createMessage() => Category._();
  @$core.override
  Category createEmptyInstance() => Category._();
  @$core.pragma('dart2js:noInline')
  static Category getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Category>(Category.$_createMessage);
  static Category? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get slug => $_getSZ(2);
  @$pb.TagNumber(3)
  set slug($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasSlug() => $_has(2);
  @$pb.TagNumber(3)
  void clearSlug() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get name => $_getSZ(3);
  @$pb.TagNumber(4)
  set name($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasName() => $_has(3);
  @$pb.TagNumber(4)
  void clearName() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get kind => $_getSZ(4);
  @$pb.TagNumber(5)
  set kind($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasKind() => $_has(4);
  @$pb.TagNumber(5)
  void clearKind() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get deductible => $_getBF(5);
  @$pb.TagNumber(6)
  set deductible($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasDeductible() => $_has(5);
  @$pb.TagNumber(6)
  void clearDeductible() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.bool get archived => $_getBF(6);
  @$pb.TagNumber(7)
  set archived($core.bool value) => $_setBool(6, value);
  @$pb.TagNumber(7)
  $core.bool hasArchived() => $_has(6);
  @$pb.TagNumber(7)
  void clearArchived() => $_clearField(7);
}

class DeclareCategoryRequest extends $pb.GeneratedMessage {
  factory DeclareCategoryRequest({
    $core.String? transactionId,
    $core.String? categoryId,
  }) {
    final result = DeclareCategoryRequest._();
    if (transactionId != null) result.transactionId = transactionId;
    if (categoryId != null) result.categoryId = categoryId;
    return result;
  }

  DeclareCategoryRequest._();

  factory DeclareCategoryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeclareCategoryRequest()..mergeFromBuffer(data, registry);
  factory DeclareCategoryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeclareCategoryRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeclareCategoryRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeclareCategoryRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'transactionId')
    ..aOS(2, _omitFieldNames ? '' : 'categoryId')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeclareCategoryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeclareCategoryRequest copyWith(
          void Function(DeclareCategoryRequest) updates) =>
      super.copyWith((message) => updates(message as DeclareCategoryRequest))
          as DeclareCategoryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeclareCategoryRequest() / DeclareCategoryRequest.new instead')
  static DeclareCategoryRequest create() => DeclareCategoryRequest._();
  static $pb.GeneratedMessage $_createMessage() => DeclareCategoryRequest._();
  @$core.override
  DeclareCategoryRequest createEmptyInstance() => DeclareCategoryRequest._();
  @$core.pragma('dart2js:noInline')
  static DeclareCategoryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeclareCategoryRequest>(
          DeclareCategoryRequest.$_createMessage);
  static DeclareCategoryRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get transactionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set transactionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasTransactionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearTransactionId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get categoryId => $_getSZ(1);
  @$pb.TagNumber(2)
  set categoryId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCategoryId() => $_has(1);
  @$pb.TagNumber(2)
  void clearCategoryId() => $_clearField(2);
}

class DeclareCategoryResponse extends $pb.GeneratedMessage {
  factory DeclareCategoryResponse({
    Transaction? transaction,
  }) {
    final result = DeclareCategoryResponse._();
    if (transaction != null) result.transaction = transaction;
    return result;
  }

  DeclareCategoryResponse._();

  factory DeclareCategoryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeclareCategoryResponse()..mergeFromBuffer(data, registry);
  factory DeclareCategoryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      DeclareCategoryResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'DeclareCategoryResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: DeclareCategoryResponse.$_createMessage)
    ..aOM<Transaction>(1, _omitFieldNames ? '' : 'transaction',
        subBuilder: Transaction.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeclareCategoryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  DeclareCategoryResponse copyWith(
          void Function(DeclareCategoryResponse) updates) =>
      super.copyWith((message) => updates(message as DeclareCategoryResponse))
          as DeclareCategoryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use DeclareCategoryResponse() / DeclareCategoryResponse.new instead')
  static DeclareCategoryResponse create() => DeclareCategoryResponse._();
  static $pb.GeneratedMessage $_createMessage() => DeclareCategoryResponse._();
  @$core.override
  DeclareCategoryResponse createEmptyInstance() => DeclareCategoryResponse._();
  @$core.pragma('dart2js:noInline')
  static DeclareCategoryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<DeclareCategoryResponse>(
          DeclareCategoryResponse.$_createMessage);
  static DeclareCategoryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Transaction get transaction => $_getN(0);
  @$pb.TagNumber(1)
  set transaction(Transaction value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTransaction() => $_has(0);
  @$pb.TagNumber(1)
  void clearTransaction() => $_clearField(1);
  @$pb.TagNumber(1)
  Transaction ensureTransaction() => $_ensure(0);
}

class UpsertCategoryRequest extends $pb.GeneratedMessage {
  factory UpsertCategoryRequest({
    $core.String? id,
    $core.String? partyId,
    $core.String? name,
    $core.String? kind,
    $core.bool? deductible,
    $core.bool? archived,
  }) {
    final result = UpsertCategoryRequest._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (name != null) result.name = name;
    if (kind != null) result.kind = kind;
    if (deductible != null) result.deductible = deductible;
    if (archived != null) result.archived = archived;
    return result;
  }

  UpsertCategoryRequest._();

  factory UpsertCategoryRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertCategoryRequest()..mergeFromBuffer(data, registry);
  factory UpsertCategoryRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertCategoryRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertCategoryRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertCategoryRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'name')
    ..aOS(4, _omitFieldNames ? '' : 'kind')
    ..aOB(5, _omitFieldNames ? '' : 'deductible')
    ..aOB(6, _omitFieldNames ? '' : 'archived')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertCategoryRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertCategoryRequest copyWith(
          void Function(UpsertCategoryRequest) updates) =>
      super.copyWith((message) => updates(message as UpsertCategoryRequest))
          as UpsertCategoryRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpsertCategoryRequest() / UpsertCategoryRequest.new instead')
  static UpsertCategoryRequest create() => UpsertCategoryRequest._();
  static $pb.GeneratedMessage $_createMessage() => UpsertCategoryRequest._();
  @$core.override
  UpsertCategoryRequest createEmptyInstance() => UpsertCategoryRequest._();
  @$core.pragma('dart2js:noInline')
  static UpsertCategoryRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertCategoryRequest>(
          UpsertCategoryRequest.$_createMessage);
  static UpsertCategoryRequest? _defaultInstance;

  /// Empty creates; otherwise the category to change.
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get name => $_getSZ(2);
  @$pb.TagNumber(3)
  set name($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasName() => $_has(2);
  @$pb.TagNumber(3)
  void clearName() => $_clearField(3);

  /// expense, income, transfer, tax or capital.
  @$pb.TagNumber(4)
  $core.String get kind => $_getSZ(3);
  @$pb.TagNumber(4)
  set kind($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasKind() => $_has(3);
  @$pb.TagNumber(4)
  void clearKind() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.bool get deductible => $_getBF(4);
  @$pb.TagNumber(5)
  set deductible($core.bool value) => $_setBool(4, value);
  @$pb.TagNumber(5)
  $core.bool hasDeductible() => $_has(4);
  @$pb.TagNumber(5)
  void clearDeductible() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.bool get archived => $_getBF(5);
  @$pb.TagNumber(6)
  set archived($core.bool value) => $_setBool(5, value);
  @$pb.TagNumber(6)
  $core.bool hasArchived() => $_has(5);
  @$pb.TagNumber(6)
  void clearArchived() => $_clearField(6);
}

class UpsertCategoryResponse extends $pb.GeneratedMessage {
  factory UpsertCategoryResponse({
    Category? category,
    $core.int? categorised,
    $core.int? unmatched,
  }) {
    final result = UpsertCategoryResponse._();
    if (category != null) result.category = category;
    if (categorised != null) result.categorised = categorised;
    if (unmatched != null) result.unmatched = unmatched;
    return result;
  }

  UpsertCategoryResponse._();

  factory UpsertCategoryResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertCategoryResponse()..mergeFromBuffer(data, registry);
  factory UpsertCategoryResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertCategoryResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertCategoryResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertCategoryResponse.$_createMessage)
    ..aOM<Category>(1, _omitFieldNames ? '' : 'category',
        subBuilder: Category.$_createMessage)
    ..aI(2, _omitFieldNames ? '' : 'categorised',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'unmatched', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertCategoryResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertCategoryResponse copyWith(
          void Function(UpsertCategoryResponse) updates) =>
      super.copyWith((message) => updates(message as UpsertCategoryResponse))
          as UpsertCategoryResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use UpsertCategoryResponse() / UpsertCategoryResponse.new instead')
  static UpsertCategoryResponse create() => UpsertCategoryResponse._();
  static $pb.GeneratedMessage $_createMessage() => UpsertCategoryResponse._();
  @$core.override
  UpsertCategoryResponse createEmptyInstance() => UpsertCategoryResponse._();
  @$core.pragma('dart2js:noInline')
  static UpsertCategoryResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertCategoryResponse>(
          UpsertCategoryResponse.$_createMessage);
  static UpsertCategoryResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Category get category => $_getN(0);
  @$pb.TagNumber(1)
  set category(Category value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasCategory() => $_has(0);
  @$pb.TagNumber(1)
  void clearCategory() => $_clearField(1);
  @$pb.TagNumber(1)
  Category ensureCategory() => $_ensure(0);

  /// What the pass after the change did, for the whole party.
  @$pb.TagNumber(2)
  $core.int get categorised => $_getIZ(1);
  @$pb.TagNumber(2)
  set categorised($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCategorised() => $_has(1);
  @$pb.TagNumber(2)
  void clearCategorised() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get unmatched => $_getIZ(2);
  @$pb.TagNumber(3)
  set unmatched($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasUnmatched() => $_has(2);
  @$pb.TagNumber(3)
  void clearUnmatched() => $_clearField(3);
}

class ListRulesRequest extends $pb.GeneratedMessage {
  factory ListRulesRequest({
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = ListRulesRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  ListRulesRequest._();

  factory ListRulesRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListRulesRequest()..mergeFromBuffer(data, registry);
  factory ListRulesRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListRulesRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRulesRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListRulesRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesRequest copyWith(void Function(ListRulesRequest) updates) =>
      super.copyWith((message) => updates(message as ListRulesRequest))
          as ListRulesRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ListRulesRequest() / ListRulesRequest.new instead')
  static ListRulesRequest create() => ListRulesRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListRulesRequest._();
  @$core.override
  ListRulesRequest createEmptyInstance() => ListRulesRequest._();
  @$core.pragma('dart2js:noInline')
  static ListRulesRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ListRulesRequest>(
          ListRulesRequest.$_createMessage);
  static ListRulesRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);
}

class ListRulesResponse extends $pb.GeneratedMessage {
  factory ListRulesResponse({
    $core.Iterable<Rule>? rules,
  }) {
    final result = ListRulesResponse._();
    if (rules != null) result.rules.addAll(rules);
    return result;
  }

  ListRulesResponse._();

  factory ListRulesResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListRulesResponse()..mergeFromBuffer(data, registry);
  factory ListRulesResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListRulesResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListRulesResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListRulesResponse.$_createMessage)
    ..pPM<Rule>(1, _omitFieldNames ? '' : 'rules',
        subBuilder: Rule.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListRulesResponse copyWith(void Function(ListRulesResponse) updates) =>
      super.copyWith((message) => updates(message as ListRulesResponse))
          as ListRulesResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use ListRulesResponse() / ListRulesResponse.new instead')
  static ListRulesResponse create() => ListRulesResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListRulesResponse._();
  @$core.override
  ListRulesResponse createEmptyInstance() => ListRulesResponse._();
  @$core.pragma('dart2js:noInline')
  static ListRulesResponse getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<ListRulesResponse>(
          ListRulesResponse.$_createMessage);
  static ListRulesResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Rule> get rules => $_getList(0);
}

class Rule extends $pb.GeneratedMessage {
  factory Rule({
    $core.String? id,
    $core.String? partyId,
    $core.int? priority,
    $core.String? name,
    $core.String? categoryId,
    $core.String? matchCounterpartyLike,
    $core.String? matchCounterpartyIban,
    $core.String? matchRemittanceLike,
    $core.String? matchCurrency,
    $core.String? matchCreditDebit,
    $core.bool? enabled,
    $fixnum.Int64? hits,
  }) {
    final result = Rule._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (priority != null) result.priority = priority;
    if (name != null) result.name = name;
    if (categoryId != null) result.categoryId = categoryId;
    if (matchCounterpartyLike != null)
      result.matchCounterpartyLike = matchCounterpartyLike;
    if (matchCounterpartyIban != null)
      result.matchCounterpartyIban = matchCounterpartyIban;
    if (matchRemittanceLike != null)
      result.matchRemittanceLike = matchRemittanceLike;
    if (matchCurrency != null) result.matchCurrency = matchCurrency;
    if (matchCreditDebit != null) result.matchCreditDebit = matchCreditDebit;
    if (enabled != null) result.enabled = enabled;
    if (hits != null) result.hits = hits;
    return result;
  }

  Rule._();

  factory Rule.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Rule()..mergeFromBuffer(data, registry);
  factory Rule.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Rule()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Rule',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Rule.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aI(3, _omitFieldNames ? '' : 'priority')
    ..aOS(4, _omitFieldNames ? '' : 'name')
    ..aOS(5, _omitFieldNames ? '' : 'categoryId')
    ..aOS(6, _omitFieldNames ? '' : 'matchCounterpartyLike')
    ..aOS(7, _omitFieldNames ? '' : 'matchCounterpartyIban')
    ..aOS(8, _omitFieldNames ? '' : 'matchRemittanceLike')
    ..aOS(9, _omitFieldNames ? '' : 'matchCurrency')
    ..aOS(10, _omitFieldNames ? '' : 'matchCreditDebit')
    ..aOB(11, _omitFieldNames ? '' : 'enabled')
    ..aInt64(12, _omitFieldNames ? '' : 'hits')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Rule clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Rule copyWith(void Function(Rule) updates) =>
      super.copyWith((message) => updates(message as Rule)) as Rule;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Rule() / Rule.new instead')
  static Rule create() => Rule._();
  static $pb.GeneratedMessage $_createMessage() => Rule._();
  @$core.override
  Rule createEmptyInstance() => Rule._();
  @$core.pragma('dart2js:noInline')
  static Rule getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Rule>(Rule.$_createMessage);
  static Rule? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get priority => $_getIZ(2);
  @$pb.TagNumber(3)
  set priority($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPriority() => $_has(2);
  @$pb.TagNumber(3)
  void clearPriority() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get name => $_getSZ(3);
  @$pb.TagNumber(4)
  set name($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasName() => $_has(3);
  @$pb.TagNumber(4)
  void clearName() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get categoryId => $_getSZ(4);
  @$pb.TagNumber(5)
  set categoryId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCategoryId() => $_has(4);
  @$pb.TagNumber(5)
  void clearCategoryId() => $_clearField(5);

  /// Every condition optional; those set must all match. Normalised: upper
  /// case, Croatian diacritics stripped.
  @$pb.TagNumber(6)
  $core.String get matchCounterpartyLike => $_getSZ(5);
  @$pb.TagNumber(6)
  set matchCounterpartyLike($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasMatchCounterpartyLike() => $_has(5);
  @$pb.TagNumber(6)
  void clearMatchCounterpartyLike() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get matchCounterpartyIban => $_getSZ(6);
  @$pb.TagNumber(7)
  set matchCounterpartyIban($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasMatchCounterpartyIban() => $_has(6);
  @$pb.TagNumber(7)
  void clearMatchCounterpartyIban() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get matchRemittanceLike => $_getSZ(7);
  @$pb.TagNumber(8)
  set matchRemittanceLike($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasMatchRemittanceLike() => $_has(7);
  @$pb.TagNumber(8)
  void clearMatchRemittanceLike() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get matchCurrency => $_getSZ(8);
  @$pb.TagNumber(9)
  set matchCurrency($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasMatchCurrency() => $_has(8);
  @$pb.TagNumber(9)
  void clearMatchCurrency() => $_clearField(9);

  /// CRDT, DBIT or empty.
  @$pb.TagNumber(10)
  $core.String get matchCreditDebit => $_getSZ(9);
  @$pb.TagNumber(10)
  set matchCreditDebit($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasMatchCreditDebit() => $_has(9);
  @$pb.TagNumber(10)
  void clearMatchCreditDebit() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.bool get enabled => $_getBF(10);
  @$pb.TagNumber(11)
  set enabled($core.bool value) => $_setBool(10, value);
  @$pb.TagNumber(11)
  $core.bool hasEnabled() => $_has(10);
  @$pb.TagNumber(11)
  void clearEnabled() => $_clearField(11);

  /// Rows this rule owns after the last pass.
  @$pb.TagNumber(12)
  $fixnum.Int64 get hits => $_getI64(11);
  @$pb.TagNumber(12)
  set hits($fixnum.Int64 value) => $_setInt64(11, value);
  @$pb.TagNumber(12)
  $core.bool hasHits() => $_has(11);
  @$pb.TagNumber(12)
  void clearHits() => $_clearField(12);
}

class UpsertRuleRequest extends $pb.GeneratedMessage {
  factory UpsertRuleRequest({
    $core.String? id,
    $core.String? partyId,
    $core.int? priority,
    $core.String? name,
    $core.String? categoryId,
    $core.String? matchCounterpartyLike,
    $core.String? matchCounterpartyIban,
    $core.String? matchRemittanceLike,
    $core.String? matchCurrency,
    $core.String? matchCreditDebit,
    $core.bool? enabled,
  }) {
    final result = UpsertRuleRequest._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (priority != null) result.priority = priority;
    if (name != null) result.name = name;
    if (categoryId != null) result.categoryId = categoryId;
    if (matchCounterpartyLike != null)
      result.matchCounterpartyLike = matchCounterpartyLike;
    if (matchCounterpartyIban != null)
      result.matchCounterpartyIban = matchCounterpartyIban;
    if (matchRemittanceLike != null)
      result.matchRemittanceLike = matchRemittanceLike;
    if (matchCurrency != null) result.matchCurrency = matchCurrency;
    if (matchCreditDebit != null) result.matchCreditDebit = matchCreditDebit;
    if (enabled != null) result.enabled = enabled;
    return result;
  }

  UpsertRuleRequest._();

  factory UpsertRuleRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertRuleRequest()..mergeFromBuffer(data, registry);
  factory UpsertRuleRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertRuleRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertRuleRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertRuleRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aI(3, _omitFieldNames ? '' : 'priority')
    ..aOS(4, _omitFieldNames ? '' : 'name')
    ..aOS(5, _omitFieldNames ? '' : 'categoryId')
    ..aOS(6, _omitFieldNames ? '' : 'matchCounterpartyLike')
    ..aOS(7, _omitFieldNames ? '' : 'matchCounterpartyIban')
    ..aOS(8, _omitFieldNames ? '' : 'matchRemittanceLike')
    ..aOS(9, _omitFieldNames ? '' : 'matchCurrency')
    ..aOS(10, _omitFieldNames ? '' : 'matchCreditDebit')
    ..aOB(11, _omitFieldNames ? '' : 'enabled')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertRuleRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertRuleRequest copyWith(void Function(UpsertRuleRequest) updates) =>
      super.copyWith((message) => updates(message as UpsertRuleRequest))
          as UpsertRuleRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use UpsertRuleRequest() / UpsertRuleRequest.new instead')
  static UpsertRuleRequest create() => UpsertRuleRequest._();
  static $pb.GeneratedMessage $_createMessage() => UpsertRuleRequest._();
  @$core.override
  UpsertRuleRequest createEmptyInstance() => UpsertRuleRequest._();
  @$core.pragma('dart2js:noInline')
  static UpsertRuleRequest getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<UpsertRuleRequest>(
          UpsertRuleRequest.$_createMessage);
  static UpsertRuleRequest? _defaultInstance;

  /// Empty creates; otherwise the rule to change.
  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get priority => $_getIZ(2);
  @$pb.TagNumber(3)
  set priority($core.int value) => $_setSignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPriority() => $_has(2);
  @$pb.TagNumber(3)
  void clearPriority() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get name => $_getSZ(3);
  @$pb.TagNumber(4)
  set name($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasName() => $_has(3);
  @$pb.TagNumber(4)
  void clearName() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get categoryId => $_getSZ(4);
  @$pb.TagNumber(5)
  set categoryId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasCategoryId() => $_has(4);
  @$pb.TagNumber(5)
  void clearCategoryId() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get matchCounterpartyLike => $_getSZ(5);
  @$pb.TagNumber(6)
  set matchCounterpartyLike($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasMatchCounterpartyLike() => $_has(5);
  @$pb.TagNumber(6)
  void clearMatchCounterpartyLike() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get matchCounterpartyIban => $_getSZ(6);
  @$pb.TagNumber(7)
  set matchCounterpartyIban($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasMatchCounterpartyIban() => $_has(6);
  @$pb.TagNumber(7)
  void clearMatchCounterpartyIban() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get matchRemittanceLike => $_getSZ(7);
  @$pb.TagNumber(8)
  set matchRemittanceLike($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasMatchRemittanceLike() => $_has(7);
  @$pb.TagNumber(8)
  void clearMatchRemittanceLike() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get matchCurrency => $_getSZ(8);
  @$pb.TagNumber(9)
  set matchCurrency($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasMatchCurrency() => $_has(8);
  @$pb.TagNumber(9)
  void clearMatchCurrency() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get matchCreditDebit => $_getSZ(9);
  @$pb.TagNumber(10)
  set matchCreditDebit($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasMatchCreditDebit() => $_has(9);
  @$pb.TagNumber(10)
  void clearMatchCreditDebit() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.bool get enabled => $_getBF(10);
  @$pb.TagNumber(11)
  set enabled($core.bool value) => $_setBool(10, value);
  @$pb.TagNumber(11)
  $core.bool hasEnabled() => $_has(10);
  @$pb.TagNumber(11)
  void clearEnabled() => $_clearField(11);
}

class UpsertRuleResponse extends $pb.GeneratedMessage {
  factory UpsertRuleResponse({
    Rule? rule,
    $core.int? categorised,
    $core.int? unmatched,
  }) {
    final result = UpsertRuleResponse._();
    if (rule != null) result.rule = rule;
    if (categorised != null) result.categorised = categorised;
    if (unmatched != null) result.unmatched = unmatched;
    return result;
  }

  UpsertRuleResponse._();

  factory UpsertRuleResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertRuleResponse()..mergeFromBuffer(data, registry);
  factory UpsertRuleResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      UpsertRuleResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'UpsertRuleResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: UpsertRuleResponse.$_createMessage)
    ..aOM<Rule>(1, _omitFieldNames ? '' : 'rule',
        subBuilder: Rule.$_createMessage)
    ..aI(2, _omitFieldNames ? '' : 'categorised',
        fieldType: $pb.PbFieldType.OU3)
    ..aI(3, _omitFieldNames ? '' : 'unmatched', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertRuleResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  UpsertRuleResponse copyWith(void Function(UpsertRuleResponse) updates) =>
      super.copyWith((message) => updates(message as UpsertRuleResponse))
          as UpsertRuleResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use UpsertRuleResponse() / UpsertRuleResponse.new instead')
  static UpsertRuleResponse create() => UpsertRuleResponse._();
  static $pb.GeneratedMessage $_createMessage() => UpsertRuleResponse._();
  @$core.override
  UpsertRuleResponse createEmptyInstance() => UpsertRuleResponse._();
  @$core.pragma('dart2js:noInline')
  static UpsertRuleResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<UpsertRuleResponse>(
          UpsertRuleResponse.$_createMessage);
  static UpsertRuleResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Rule get rule => $_getN(0);
  @$pb.TagNumber(1)
  set rule(Rule value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasRule() => $_has(0);
  @$pb.TagNumber(1)
  void clearRule() => $_clearField(1);
  @$pb.TagNumber(1)
  Rule ensureRule() => $_ensure(0);

  /// What the pass after the change did, for the whole party.
  @$pb.TagNumber(2)
  $core.int get categorised => $_getIZ(1);
  @$pb.TagNumber(2)
  set categorised($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCategorised() => $_has(1);
  @$pb.TagNumber(2)
  void clearCategorised() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.int get unmatched => $_getIZ(2);
  @$pb.TagNumber(3)
  set unmatched($core.int value) => $_setUnsignedInt32(2, value);
  @$pb.TagNumber(3)
  $core.bool hasUnmatched() => $_has(2);
  @$pb.TagNumber(3)
  void clearUnmatched() => $_clearField(3);
}

class StartConnectionRequest extends $pb.GeneratedMessage {
  factory StartConnectionRequest({
    $core.String? partyId,
    $core.String? psuType,
    $core.String? aspspName,
    $core.String? aspspCountry,
  }) {
    final result = StartConnectionRequest._();
    if (partyId != null) result.partyId = partyId;
    if (psuType != null) result.psuType = psuType;
    if (aspspName != null) result.aspspName = aspspName;
    if (aspspCountry != null) result.aspspCountry = aspspCountry;
    return result;
  }

  StartConnectionRequest._();

  factory StartConnectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartConnectionRequest()..mergeFromBuffer(data, registry);
  factory StartConnectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartConnectionRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StartConnectionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: StartConnectionRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'partyId')
    ..aOS(2, _omitFieldNames ? '' : 'psuType')
    ..aOS(3, _omitFieldNames ? '' : 'aspspName')
    ..aOS(4, _omitFieldNames ? '' : 'aspspCountry')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartConnectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartConnectionRequest copyWith(
          void Function(StartConnectionRequest) updates) =>
      super.copyWith((message) => updates(message as StartConnectionRequest))
          as StartConnectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use StartConnectionRequest() / StartConnectionRequest.new instead')
  static StartConnectionRequest create() => StartConnectionRequest._();
  static $pb.GeneratedMessage $_createMessage() => StartConnectionRequest._();
  @$core.override
  StartConnectionRequest createEmptyInstance() => StartConnectionRequest._();
  @$core.pragma('dart2js:noInline')
  static StartConnectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StartConnectionRequest>(
          StartConnectionRequest.$_createMessage);
  static StartConnectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get partyId => $_getSZ(0);
  @$pb.TagNumber(1)
  set partyId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasPartyId() => $_has(0);
  @$pb.TagNumber(1)
  void clearPartyId() => $_clearField(1);

  /// business or personal.
  @$pb.TagNumber(2)
  $core.String get psuType => $_getSZ(1);
  @$pb.TagNumber(2)
  set psuType($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPsuType() => $_has(1);
  @$pb.TagNumber(2)
  void clearPsuType() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get aspspName => $_getSZ(2);
  @$pb.TagNumber(3)
  set aspspName($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasAspspName() => $_has(2);
  @$pb.TagNumber(3)
  void clearAspspName() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get aspspCountry => $_getSZ(3);
  @$pb.TagNumber(4)
  set aspspCountry($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasAspspCountry() => $_has(3);
  @$pb.TagNumber(4)
  void clearAspspCountry() => $_clearField(4);
}

class StartConnectionResponse extends $pb.GeneratedMessage {
  factory StartConnectionResponse({
    $core.String? connectionId,
    $core.String? url,
  }) {
    final result = StartConnectionResponse._();
    if (connectionId != null) result.connectionId = connectionId;
    if (url != null) result.url = url;
    return result;
  }

  StartConnectionResponse._();

  factory StartConnectionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartConnectionResponse()..mergeFromBuffer(data, registry);
  factory StartConnectionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      StartConnectionResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'StartConnectionResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: StartConnectionResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'connectionId')
    ..aOS(2, _omitFieldNames ? '' : 'url')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartConnectionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  StartConnectionResponse copyWith(
          void Function(StartConnectionResponse) updates) =>
      super.copyWith((message) => updates(message as StartConnectionResponse))
          as StartConnectionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use StartConnectionResponse() / StartConnectionResponse.new instead')
  static StartConnectionResponse create() => StartConnectionResponse._();
  static $pb.GeneratedMessage $_createMessage() => StartConnectionResponse._();
  @$core.override
  StartConnectionResponse createEmptyInstance() => StartConnectionResponse._();
  @$core.pragma('dart2js:noInline')
  static StartConnectionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<StartConnectionResponse>(
          StartConnectionResponse.$_createMessage);
  static StartConnectionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get connectionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set connectionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasConnectionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnectionId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get url => $_getSZ(1);
  @$pb.TagNumber(2)
  set url($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasUrl() => $_has(1);
  @$pb.TagNumber(2)
  void clearUrl() => $_clearField(2);
}

class CompleteConnectionRequest extends $pb.GeneratedMessage {
  factory CompleteConnectionRequest({
    $core.String? state,
    $core.String? code,
  }) {
    final result = CompleteConnectionRequest._();
    if (state != null) result.state = state;
    if (code != null) result.code = code;
    return result;
  }

  CompleteConnectionRequest._();

  factory CompleteConnectionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteConnectionRequest()..mergeFromBuffer(data, registry);
  factory CompleteConnectionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteConnectionRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteConnectionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: CompleteConnectionRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'state')
    ..aOS(2, _omitFieldNames ? '' : 'code')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteConnectionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteConnectionRequest copyWith(
          void Function(CompleteConnectionRequest) updates) =>
      super.copyWith((message) => updates(message as CompleteConnectionRequest))
          as CompleteConnectionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use CompleteConnectionRequest() / CompleteConnectionRequest.new instead')
  static CompleteConnectionRequest create() => CompleteConnectionRequest._();
  static $pb.GeneratedMessage $_createMessage() =>
      CompleteConnectionRequest._();
  @$core.override
  CompleteConnectionRequest createEmptyInstance() =>
      CompleteConnectionRequest._();
  @$core.pragma('dart2js:noInline')
  static CompleteConnectionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteConnectionRequest>(
          CompleteConnectionRequest.$_createMessage);
  static CompleteConnectionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get state => $_getSZ(0);
  @$pb.TagNumber(1)
  set state($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasState() => $_has(0);
  @$pb.TagNumber(1)
  void clearState() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get code => $_getSZ(1);
  @$pb.TagNumber(2)
  set code($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasCode() => $_has(1);
  @$pb.TagNumber(2)
  void clearCode() => $_clearField(2);
}

class CompleteConnectionResponse extends $pb.GeneratedMessage {
  factory CompleteConnectionResponse({
    $core.String? connectionId,
    $core.Iterable<$core.String>? accountIds,
  }) {
    final result = CompleteConnectionResponse._();
    if (connectionId != null) result.connectionId = connectionId;
    if (accountIds != null) result.accountIds.addAll(accountIds);
    return result;
  }

  CompleteConnectionResponse._();

  factory CompleteConnectionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteConnectionResponse()..mergeFromBuffer(data, registry);
  factory CompleteConnectionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      CompleteConnectionResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'CompleteConnectionResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: CompleteConnectionResponse.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'connectionId')
    ..pPS(2, _omitFieldNames ? '' : 'accountIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteConnectionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  CompleteConnectionResponse copyWith(
          void Function(CompleteConnectionResponse) updates) =>
      super.copyWith(
              (message) => updates(message as CompleteConnectionResponse))
          as CompleteConnectionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use CompleteConnectionResponse() / CompleteConnectionResponse.new instead')
  static CompleteConnectionResponse create() => CompleteConnectionResponse._();
  static $pb.GeneratedMessage $_createMessage() =>
      CompleteConnectionResponse._();
  @$core.override
  CompleteConnectionResponse createEmptyInstance() =>
      CompleteConnectionResponse._();
  @$core.pragma('dart2js:noInline')
  static CompleteConnectionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<CompleteConnectionResponse>(
          CompleteConnectionResponse.$_createMessage);
  static CompleteConnectionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get connectionId => $_getSZ(0);
  @$pb.TagNumber(1)
  set connectionId($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasConnectionId() => $_has(0);
  @$pb.TagNumber(1)
  void clearConnectionId() => $_clearField(1);

  @$pb.TagNumber(2)
  $pb.PbList<$core.String> get accountIds => $_getList(1);
}

class ListConnectionsRequest extends $pb.GeneratedMessage {
  factory ListConnectionsRequest({
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = ListConnectionsRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  ListConnectionsRequest._();

  factory ListConnectionsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectionsRequest()..mergeFromBuffer(data, registry);
  factory ListConnectionsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectionsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConnectionsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListConnectionsRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectionsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectionsRequest copyWith(
          void Function(ListConnectionsRequest) updates) =>
      super.copyWith((message) => updates(message as ListConnectionsRequest))
          as ListConnectionsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListConnectionsRequest() / ListConnectionsRequest.new instead')
  static ListConnectionsRequest create() => ListConnectionsRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListConnectionsRequest._();
  @$core.override
  ListConnectionsRequest createEmptyInstance() => ListConnectionsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListConnectionsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConnectionsRequest>(
          ListConnectionsRequest.$_createMessage);
  static ListConnectionsRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);
}

class ListConnectionsResponse extends $pb.GeneratedMessage {
  factory ListConnectionsResponse({
    $core.Iterable<Connection>? connections,
  }) {
    final result = ListConnectionsResponse._();
    if (connections != null) result.connections.addAll(connections);
    return result;
  }

  ListConnectionsResponse._();

  factory ListConnectionsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectionsResponse()..mergeFromBuffer(data, registry);
  factory ListConnectionsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListConnectionsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListConnectionsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListConnectionsResponse.$_createMessage)
    ..pPM<Connection>(1, _omitFieldNames ? '' : 'connections',
        subBuilder: Connection.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectionsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListConnectionsResponse copyWith(
          void Function(ListConnectionsResponse) updates) =>
      super.copyWith((message) => updates(message as ListConnectionsResponse))
          as ListConnectionsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListConnectionsResponse() / ListConnectionsResponse.new instead')
  static ListConnectionsResponse create() => ListConnectionsResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListConnectionsResponse._();
  @$core.override
  ListConnectionsResponse createEmptyInstance() => ListConnectionsResponse._();
  @$core.pragma('dart2js:noInline')
  static ListConnectionsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListConnectionsResponse>(
          ListConnectionsResponse.$_createMessage);
  static ListConnectionsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Connection> get connections => $_getList(0);
}

class Connection extends $pb.GeneratedMessage {
  factory Connection({
    $core.String? id,
    $core.String? partyId,
    $core.String? provider,
    $core.String? psuType,
    $core.String? aspspName,
    $core.String? status,
    $core.String? validUntil,
    $core.String? authorizedAt,
    $core.int? accounts,
  }) {
    final result = Connection._();
    if (id != null) result.id = id;
    if (partyId != null) result.partyId = partyId;
    if (provider != null) result.provider = provider;
    if (psuType != null) result.psuType = psuType;
    if (aspspName != null) result.aspspName = aspspName;
    if (status != null) result.status = status;
    if (validUntil != null) result.validUntil = validUntil;
    if (authorizedAt != null) result.authorizedAt = authorizedAt;
    if (accounts != null) result.accounts = accounts;
    return result;
  }

  Connection._();

  factory Connection.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Connection()..mergeFromBuffer(data, registry);
  factory Connection.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Connection()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Connection',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Connection.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'partyId')
    ..aOS(3, _omitFieldNames ? '' : 'provider')
    ..aOS(4, _omitFieldNames ? '' : 'psuType')
    ..aOS(5, _omitFieldNames ? '' : 'aspspName')
    ..aOS(6, _omitFieldNames ? '' : 'status')
    ..aOS(7, _omitFieldNames ? '' : 'validUntil')
    ..aOS(8, _omitFieldNames ? '' : 'authorizedAt')
    ..aI(9, _omitFieldNames ? '' : 'accounts', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Connection clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Connection copyWith(void Function(Connection) updates) =>
      super.copyWith((message) => updates(message as Connection)) as Connection;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Connection() / Connection.new instead')
  static Connection create() => Connection._();
  static $pb.GeneratedMessage $_createMessage() => Connection._();
  @$core.override
  Connection createEmptyInstance() => Connection._();
  @$core.pragma('dart2js:noInline')
  static Connection getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<Connection>(Connection.$_createMessage);
  static Connection? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get partyId => $_getSZ(1);
  @$pb.TagNumber(2)
  set partyId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasPartyId() => $_has(1);
  @$pb.TagNumber(2)
  void clearPartyId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get provider => $_getSZ(2);
  @$pb.TagNumber(3)
  set provider($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasProvider() => $_has(2);
  @$pb.TagNumber(3)
  void clearProvider() => $_clearField(3);

  @$pb.TagNumber(4)
  $core.String get psuType => $_getSZ(3);
  @$pb.TagNumber(4)
  set psuType($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasPsuType() => $_has(3);
  @$pb.TagNumber(4)
  void clearPsuType() => $_clearField(4);

  @$pb.TagNumber(5)
  $core.String get aspspName => $_getSZ(4);
  @$pb.TagNumber(5)
  set aspspName($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasAspspName() => $_has(4);
  @$pb.TagNumber(5)
  void clearAspspName() => $_clearField(5);

  /// pending, authorized, expired, revoked, failed.
  @$pb.TagNumber(6)
  $core.String get status => $_getSZ(5);
  @$pb.TagNumber(6)
  set status($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasStatus() => $_has(5);
  @$pb.TagNumber(6)
  void clearStatus() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.String get validUntil => $_getSZ(6);
  @$pb.TagNumber(7)
  set validUntil($core.String value) => $_setString(6, value);
  @$pb.TagNumber(7)
  $core.bool hasValidUntil() => $_has(6);
  @$pb.TagNumber(7)
  void clearValidUntil() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get authorizedAt => $_getSZ(7);
  @$pb.TagNumber(8)
  set authorizedAt($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasAuthorizedAt() => $_has(7);
  @$pb.TagNumber(8)
  void clearAuthorizedAt() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.int get accounts => $_getIZ(8);
  @$pb.TagNumber(9)
  set accounts($core.int value) => $_setUnsignedInt32(8, value);
  @$pb.TagNumber(9)
  $core.bool hasAccounts() => $_has(8);
  @$pb.TagNumber(9)
  void clearAccounts() => $_clearField(9);
}

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
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
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
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
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

class ListTransactionsRequest extends $pb.GeneratedMessage {
  factory ListTransactionsRequest({
    $core.Iterable<$core.String>? partyIds,
    $core.int? limit,
    $core.String? month,
    $core.String? categoryId,
    $core.String? accountId,
    $core.String? search,
    $core.int? offset,
  }) {
    final result = ListTransactionsRequest._();
    if (partyIds != null) result.partyIds.addAll(partyIds);
    if (limit != null) result.limit = limit;
    if (month != null) result.month = month;
    if (categoryId != null) result.categoryId = categoryId;
    if (accountId != null) result.accountId = accountId;
    if (search != null) result.search = search;
    if (offset != null) result.offset = offset;
    return result;
  }

  ListTransactionsRequest._();

  factory ListTransactionsRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListTransactionsRequest()..mergeFromBuffer(data, registry);
  factory ListTransactionsRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListTransactionsRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTransactionsRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListTransactionsRequest.$_createMessage)
    ..pPS(1, _omitFieldNames ? '' : 'partyIds')
    ..aI(2, _omitFieldNames ? '' : 'limit', fieldType: $pb.PbFieldType.OU3)
    ..aOS(3, _omitFieldNames ? '' : 'month')
    ..aOS(4, _omitFieldNames ? '' : 'categoryId')
    ..aOS(5, _omitFieldNames ? '' : 'accountId')
    ..aOS(6, _omitFieldNames ? '' : 'search')
    ..aI(7, _omitFieldNames ? '' : 'offset', fieldType: $pb.PbFieldType.OU3)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTransactionsRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTransactionsRequest copyWith(
          void Function(ListTransactionsRequest) updates) =>
      super.copyWith((message) => updates(message as ListTransactionsRequest))
          as ListTransactionsRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListTransactionsRequest() / ListTransactionsRequest.new instead')
  static ListTransactionsRequest create() => ListTransactionsRequest._();
  static $pb.GeneratedMessage $_createMessage() => ListTransactionsRequest._();
  @$core.override
  ListTransactionsRequest createEmptyInstance() => ListTransactionsRequest._();
  @$core.pragma('dart2js:noInline')
  static ListTransactionsRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTransactionsRequest>(
          ListTransactionsRequest.$_createMessage);
  static ListTransactionsRequest? _defaultInstance;

  /// Narrow to these parties. Ids the caller has no grant for are dropped, not
  /// refused: refusing would confirm which of them exist. Empty means every
  /// party the caller may read.
  @$pb.TagNumber(1)
  $pb.PbList<$core.String> get partyIds => $_getList(0);

  /// Page size, 1..=500. Zero means the default.
  @$pb.TagNumber(2)
  $core.int get limit => $_getIZ(1);
  @$pb.TagNumber(2)
  set limit($core.int value) => $_setUnsignedInt32(1, value);
  @$pb.TagNumber(2)
  $core.bool hasLimit() => $_has(1);
  @$pb.TagNumber(2)
  void clearLimit() => $_clearField(2);

  /// YYYY-MM: only that month. Empty means newest first, any month.
  @$pb.TagNumber(3)
  $core.String get month => $_getSZ(2);
  @$pb.TagNumber(3)
  set month($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasMonth() => $_has(2);
  @$pb.TagNumber(3)
  void clearMonth() => $_clearField(3);

  /// Only this category; "none" for uncategorised rows.
  @$pb.TagNumber(4)
  $core.String get categoryId => $_getSZ(3);
  @$pb.TagNumber(4)
  set categoryId($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasCategoryId() => $_has(3);
  @$pb.TagNumber(4)
  void clearCategoryId() => $_clearField(4);

  /// Only this account.
  @$pb.TagNumber(5)
  $core.String get accountId => $_getSZ(4);
  @$pb.TagNumber(5)
  set accountId($core.String value) => $_setString(4, value);
  @$pb.TagNumber(5)
  $core.bool hasAccountId() => $_has(4);
  @$pb.TagNumber(5)
  void clearAccountId() => $_clearField(5);

  /// Case-insensitive substring of counterparty or remittance.
  @$pb.TagNumber(6)
  $core.String get search => $_getSZ(5);
  @$pb.TagNumber(6)
  set search($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasSearch() => $_has(5);
  @$pb.TagNumber(6)
  void clearSearch() => $_clearField(6);

  /// Rows before this many are skipped, for paging.
  @$pb.TagNumber(7)
  $core.int get offset => $_getIZ(6);
  @$pb.TagNumber(7)
  set offset($core.int value) => $_setUnsignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasOffset() => $_has(6);
  @$pb.TagNumber(7)
  void clearOffset() => $_clearField(7);
}

class ListTransactionsResponse extends $pb.GeneratedMessage {
  factory ListTransactionsResponse({
    $core.Iterable<Transaction>? transactions,
    $core.Iterable<$core.String>? partyIds,
  }) {
    final result = ListTransactionsResponse._();
    if (transactions != null) result.transactions.addAll(transactions);
    if (partyIds != null) result.partyIds.addAll(partyIds);
    return result;
  }

  ListTransactionsResponse._();

  factory ListTransactionsResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListTransactionsResponse()..mergeFromBuffer(data, registry);
  factory ListTransactionsResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      ListTransactionsResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'ListTransactionsResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: ListTransactionsResponse.$_createMessage)
    ..pPM<Transaction>(1, _omitFieldNames ? '' : 'transactions',
        subBuilder: Transaction.$_createMessage)
    ..pPS(2, _omitFieldNames ? '' : 'partyIds')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTransactionsResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  ListTransactionsResponse copyWith(
          void Function(ListTransactionsResponse) updates) =>
      super.copyWith((message) => updates(message as ListTransactionsResponse))
          as ListTransactionsResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use ListTransactionsResponse() / ListTransactionsResponse.new instead')
  static ListTransactionsResponse create() => ListTransactionsResponse._();
  static $pb.GeneratedMessage $_createMessage() => ListTransactionsResponse._();
  @$core.override
  ListTransactionsResponse createEmptyInstance() =>
      ListTransactionsResponse._();
  @$core.pragma('dart2js:noInline')
  static ListTransactionsResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<ListTransactionsResponse>(
          ListTransactionsResponse.$_createMessage);
  static ListTransactionsResponse? _defaultInstance;

  @$pb.TagNumber(1)
  $pb.PbList<Transaction> get transactions => $_getList(0);

  /// Parties this answer was drawn from: what the caller may read, after
  /// narrowing. Empty when the caller may read nothing.
  @$pb.TagNumber(2)
  $pb.PbList<$core.String> get partyIds => $_getList(1);
}

class Transaction extends $pb.GeneratedMessage {
  factory Transaction({
    $core.String? id,
    $core.String? accountId,
    $core.String? partyId,
    $core.String? status,
    $fixnum.Int64? amountMinor,
    $core.String? currency,
    $core.int? scale,
    $core.String? bookingDate,
    $core.String? counterpartyName,
    $core.String? remittance,
    $core.String? valueDate,
    $core.String? counterpartyIban,
    $core.String? categoryId,
    $core.String? category,
    $core.String? categorySource,
    $core.bool? internal,
    $core.String? referenceNumber,
    $core.String? entryReference,
    $core.String? categoryRuleId,
    $core.String? categorisedAt,
    $core.String? accountName,
    $core.String? raw,
  }) {
    final result = Transaction._();
    if (id != null) result.id = id;
    if (accountId != null) result.accountId = accountId;
    if (partyId != null) result.partyId = partyId;
    if (status != null) result.status = status;
    if (amountMinor != null) result.amountMinor = amountMinor;
    if (currency != null) result.currency = currency;
    if (scale != null) result.scale = scale;
    if (bookingDate != null) result.bookingDate = bookingDate;
    if (counterpartyName != null) result.counterpartyName = counterpartyName;
    if (remittance != null) result.remittance = remittance;
    if (valueDate != null) result.valueDate = valueDate;
    if (counterpartyIban != null) result.counterpartyIban = counterpartyIban;
    if (categoryId != null) result.categoryId = categoryId;
    if (category != null) result.category = category;
    if (categorySource != null) result.categorySource = categorySource;
    if (internal != null) result.internal = internal;
    if (referenceNumber != null) result.referenceNumber = referenceNumber;
    if (entryReference != null) result.entryReference = entryReference;
    if (categoryRuleId != null) result.categoryRuleId = categoryRuleId;
    if (categorisedAt != null) result.categorisedAt = categorisedAt;
    if (accountName != null) result.accountName = accountName;
    if (raw != null) result.raw = raw;
    return result;
  }

  Transaction._();

  factory Transaction.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Transaction()..mergeFromBuffer(data, registry);
  factory Transaction.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      Transaction()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'Transaction',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: Transaction.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..aOS(2, _omitFieldNames ? '' : 'accountId')
    ..aOS(3, _omitFieldNames ? '' : 'partyId')
    ..aOS(4, _omitFieldNames ? '' : 'status')
    ..aInt64(5, _omitFieldNames ? '' : 'amountMinor')
    ..aOS(6, _omitFieldNames ? '' : 'currency')
    ..aI(7, _omitFieldNames ? '' : 'scale', fieldType: $pb.PbFieldType.OU3)
    ..aOS(8, _omitFieldNames ? '' : 'bookingDate')
    ..aOS(9, _omitFieldNames ? '' : 'counterpartyName')
    ..aOS(10, _omitFieldNames ? '' : 'remittance')
    ..aOS(11, _omitFieldNames ? '' : 'valueDate')
    ..aOS(12, _omitFieldNames ? '' : 'counterpartyIban')
    ..aOS(13, _omitFieldNames ? '' : 'categoryId')
    ..aOS(14, _omitFieldNames ? '' : 'category')
    ..aOS(15, _omitFieldNames ? '' : 'categorySource')
    ..aOB(16, _omitFieldNames ? '' : 'internal')
    ..aOS(17, _omitFieldNames ? '' : 'referenceNumber')
    ..aOS(18, _omitFieldNames ? '' : 'entryReference')
    ..aOS(19, _omitFieldNames ? '' : 'categoryRuleId')
    ..aOS(20, _omitFieldNames ? '' : 'categorisedAt')
    ..aOS(21, _omitFieldNames ? '' : 'accountName')
    ..aOS(22, _omitFieldNames ? '' : 'raw')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Transaction clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  Transaction copyWith(void Function(Transaction) updates) =>
      super.copyWith((message) => updates(message as Transaction))
          as Transaction;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated('Use Transaction() / Transaction.new instead')
  static Transaction create() => Transaction._();
  static $pb.GeneratedMessage $_createMessage() => Transaction._();
  @$core.override
  Transaction createEmptyInstance() => Transaction._();
  @$core.pragma('dart2js:noInline')
  static Transaction getDefault() =>
      _defaultInstance ??= $pb.GeneratedMessage.$_defaultFor<Transaction>(
          Transaction.$_createMessage);
  static Transaction? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);

  @$pb.TagNumber(2)
  $core.String get accountId => $_getSZ(1);
  @$pb.TagNumber(2)
  set accountId($core.String value) => $_setString(1, value);
  @$pb.TagNumber(2)
  $core.bool hasAccountId() => $_has(1);
  @$pb.TagNumber(2)
  void clearAccountId() => $_clearField(2);

  @$pb.TagNumber(3)
  $core.String get partyId => $_getSZ(2);
  @$pb.TagNumber(3)
  set partyId($core.String value) => $_setString(2, value);
  @$pb.TagNumber(3)
  $core.bool hasPartyId() => $_has(2);
  @$pb.TagNumber(3)
  void clearPartyId() => $_clearField(3);

  /// BOOKED or PENDING.
  @$pb.TagNumber(4)
  $core.String get status => $_getSZ(3);
  @$pb.TagNumber(4)
  set status($core.String value) => $_setString(3, value);
  @$pb.TagNumber(4)
  $core.bool hasStatus() => $_has(3);
  @$pb.TagNumber(4)
  void clearStatus() => $_clearField(4);

  /// Minor units, signed: negative is money out. Rendered as a JSON string by
  /// the transcoder, which is what money wants in a browser.
  @$pb.TagNumber(5)
  $fixnum.Int64 get amountMinor => $_getI64(4);
  @$pb.TagNumber(5)
  set amountMinor($fixnum.Int64 value) => $_setInt64(4, value);
  @$pb.TagNumber(5)
  $core.bool hasAmountMinor() => $_has(4);
  @$pb.TagNumber(5)
  void clearAmountMinor() => $_clearField(5);

  @$pb.TagNumber(6)
  $core.String get currency => $_getSZ(5);
  @$pb.TagNumber(6)
  set currency($core.String value) => $_setString(5, value);
  @$pb.TagNumber(6)
  $core.bool hasCurrency() => $_has(5);
  @$pb.TagNumber(6)
  void clearCurrency() => $_clearField(6);

  @$pb.TagNumber(7)
  $core.int get scale => $_getIZ(6);
  @$pb.TagNumber(7)
  set scale($core.int value) => $_setUnsignedInt32(6, value);
  @$pb.TagNumber(7)
  $core.bool hasScale() => $_has(6);
  @$pb.TagNumber(7)
  void clearScale() => $_clearField(7);

  @$pb.TagNumber(8)
  $core.String get bookingDate => $_getSZ(7);
  @$pb.TagNumber(8)
  set bookingDate($core.String value) => $_setString(7, value);
  @$pb.TagNumber(8)
  $core.bool hasBookingDate() => $_has(7);
  @$pb.TagNumber(8)
  void clearBookingDate() => $_clearField(8);

  @$pb.TagNumber(9)
  $core.String get counterpartyName => $_getSZ(8);
  @$pb.TagNumber(9)
  set counterpartyName($core.String value) => $_setString(8, value);
  @$pb.TagNumber(9)
  $core.bool hasCounterpartyName() => $_has(8);
  @$pb.TagNumber(9)
  void clearCounterpartyName() => $_clearField(9);

  @$pb.TagNumber(10)
  $core.String get remittance => $_getSZ(9);
  @$pb.TagNumber(10)
  set remittance($core.String value) => $_setString(9, value);
  @$pb.TagNumber(10)
  $core.bool hasRemittance() => $_has(9);
  @$pb.TagNumber(10)
  void clearRemittance() => $_clearField(10);

  @$pb.TagNumber(11)
  $core.String get valueDate => $_getSZ(10);
  @$pb.TagNumber(11)
  set valueDate($core.String value) => $_setString(10, value);
  @$pb.TagNumber(11)
  $core.bool hasValueDate() => $_has(10);
  @$pb.TagNumber(11)
  void clearValueDate() => $_clearField(11);

  @$pb.TagNumber(12)
  $core.String get counterpartyIban => $_getSZ(11);
  @$pb.TagNumber(12)
  set counterpartyIban($core.String value) => $_setString(11, value);
  @$pb.TagNumber(12)
  $core.bool hasCounterpartyIban() => $_has(11);
  @$pb.TagNumber(12)
  void clearCounterpartyIban() => $_clearField(12);

  /// Empty when nothing has claimed it.
  @$pb.TagNumber(13)
  $core.String get categoryId => $_getSZ(12);
  @$pb.TagNumber(13)
  set categoryId($core.String value) => $_setString(12, value);
  @$pb.TagNumber(13)
  $core.bool hasCategoryId() => $_has(12);
  @$pb.TagNumber(13)
  void clearCategoryId() => $_clearField(13);

  @$pb.TagNumber(14)
  $core.String get category => $_getSZ(13);
  @$pb.TagNumber(14)
  set category($core.String value) => $_setString(13, value);
  @$pb.TagNumber(14)
  $core.bool hasCategory() => $_has(13);
  @$pb.TagNumber(14)
  void clearCategory() => $_clearField(14);

  /// declared (a person) or inferred (a rule); empty when uncategorised.
  @$pb.TagNumber(15)
  $core.String get categorySource => $_getSZ(14);
  @$pb.TagNumber(15)
  set categorySource($core.String value) => $_setString(14, value);
  @$pb.TagNumber(15)
  $core.bool hasCategorySource() => $_has(14);
  @$pb.TagNumber(15)
  void clearCategorySource() => $_clearField(15);

  /// A transfer between the caller's own accounts.
  @$pb.TagNumber(16)
  $core.bool get internal => $_getBF(15);
  @$pb.TagNumber(16)
  set internal($core.bool value) => $_setBool(15, value);
  @$pb.TagNumber(16)
  $core.bool hasInternal() => $_has(15);
  @$pb.TagNumber(16)
  void clearInternal() => $_clearField(16);

  /// The structured reference (Croatian "poziv na broj"), when the bank gave one.
  @$pb.TagNumber(17)
  $core.String get referenceNumber => $_getSZ(16);
  @$pb.TagNumber(17)
  set referenceNumber($core.String value) => $_setString(16, value);
  @$pb.TagNumber(17)
  $core.bool hasReferenceNumber() => $_has(16);
  @$pb.TagNumber(17)
  void clearReferenceNumber() => $_clearField(17);

  /// The bank's own id for the entry.
  @$pb.TagNumber(18)
  $core.String get entryReference => $_getSZ(17);
  @$pb.TagNumber(18)
  set entryReference($core.String value) => $_setString(17, value);
  @$pb.TagNumber(18)
  $core.bool hasEntryReference() => $_has(17);
  @$pb.TagNumber(18)
  void clearEntryReference() => $_clearField(18);

  /// The rule that claimed it, when `category_source` is inferred.
  @$pb.TagNumber(19)
  $core.String get categoryRuleId => $_getSZ(18);
  @$pb.TagNumber(19)
  set categoryRuleId($core.String value) => $_setString(18, value);
  @$pb.TagNumber(19)
  $core.bool hasCategoryRuleId() => $_has(18);
  @$pb.TagNumber(19)
  void clearCategoryRuleId() => $_clearField(19);

  /// RFC 3339, when it was last categorised.
  @$pb.TagNumber(20)
  $core.String get categorisedAt => $_getSZ(19);
  @$pb.TagNumber(20)
  set categorisedAt($core.String value) => $_setString(19, value);
  @$pb.TagNumber(20)
  $core.bool hasCategorisedAt() => $_has(19);
  @$pb.TagNumber(20)
  void clearCategorisedAt() => $_clearField(20);

  @$pb.TagNumber(21)
  $core.String get accountName => $_getSZ(20);
  @$pb.TagNumber(21)
  set accountName($core.String value) => $_setString(20, value);
  @$pb.TagNumber(21)
  $core.bool hasAccountName() => $_has(20);
  @$pb.TagNumber(21)
  void clearAccountName() => $_clearField(21);

  /// The bank's record as JSON. Only GetTransaction fills it.
  @$pb.TagNumber(22)
  $core.String get raw => $_getSZ(21);
  @$pb.TagNumber(22)
  set raw($core.String value) => $_setString(21, value);
  @$pb.TagNumber(22)
  $core.bool hasRaw() => $_has(21);
  @$pb.TagNumber(22)
  void clearRaw() => $_clearField(22);
}

class GetTransactionRequest extends $pb.GeneratedMessage {
  factory GetTransactionRequest({
    $core.String? id,
  }) {
    final result = GetTransactionRequest._();
    if (id != null) result.id = id;
    return result;
  }

  GetTransactionRequest._();

  factory GetTransactionRequest.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetTransactionRequest()..mergeFromBuffer(data, registry);
  factory GetTransactionRequest.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetTransactionRequest()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetTransactionRequest',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetTransactionRequest.$_createMessage)
    ..aOS(1, _omitFieldNames ? '' : 'id')
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTransactionRequest clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTransactionRequest copyWith(
          void Function(GetTransactionRequest) updates) =>
      super.copyWith((message) => updates(message as GetTransactionRequest))
          as GetTransactionRequest;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use GetTransactionRequest() / GetTransactionRequest.new instead')
  static GetTransactionRequest create() => GetTransactionRequest._();
  static $pb.GeneratedMessage $_createMessage() => GetTransactionRequest._();
  @$core.override
  GetTransactionRequest createEmptyInstance() => GetTransactionRequest._();
  @$core.pragma('dart2js:noInline')
  static GetTransactionRequest getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetTransactionRequest>(
          GetTransactionRequest.$_createMessage);
  static GetTransactionRequest? _defaultInstance;

  @$pb.TagNumber(1)
  $core.String get id => $_getSZ(0);
  @$pb.TagNumber(1)
  set id($core.String value) => $_setString(0, value);
  @$pb.TagNumber(1)
  $core.bool hasId() => $_has(0);
  @$pb.TagNumber(1)
  void clearId() => $_clearField(1);
}

class GetTransactionResponse extends $pb.GeneratedMessage {
  factory GetTransactionResponse({
    Transaction? transaction,
  }) {
    final result = GetTransactionResponse._();
    if (transaction != null) result.transaction = transaction;
    return result;
  }

  GetTransactionResponse._();

  factory GetTransactionResponse.fromBuffer($core.List<$core.int> data,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetTransactionResponse()..mergeFromBuffer(data, registry);
  factory GetTransactionResponse.fromJson($core.String json,
          [$pb.ExtensionRegistry registry = $pb.ExtensionRegistry.EMPTY]) =>
      GetTransactionResponse()..mergeFromJson(json, registry);

  static final $pb.BuilderInfo _i = $pb.BuilderInfo(
      _omitMessageNames ? '' : 'GetTransactionResponse',
      package: const $pb.PackageName(_omitMessageNames ? '' : 'tbd.finance.v1'),
      createEmptyInstance: GetTransactionResponse.$_createMessage)
    ..aOM<Transaction>(1, _omitFieldNames ? '' : 'transaction',
        subBuilder: Transaction.$_createMessage)
    ..hasRequiredFields = false;

  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTransactionResponse clone() => deepCopy();
  @$core.Deprecated('See https://github.com/google/protobuf.dart/issues/998.')
  GetTransactionResponse copyWith(
          void Function(GetTransactionResponse) updates) =>
      super.copyWith((message) => updates(message as GetTransactionResponse))
          as GetTransactionResponse;

  @$core.override
  $pb.BuilderInfo get info_ => _i;

  @$core.pragma('dart2js:noInline')
  @$core.Deprecated(
      'Use GetTransactionResponse() / GetTransactionResponse.new instead')
  static GetTransactionResponse create() => GetTransactionResponse._();
  static $pb.GeneratedMessage $_createMessage() => GetTransactionResponse._();
  @$core.override
  GetTransactionResponse createEmptyInstance() => GetTransactionResponse._();
  @$core.pragma('dart2js:noInline')
  static GetTransactionResponse getDefault() => _defaultInstance ??=
      $pb.GeneratedMessage.$_defaultFor<GetTransactionResponse>(
          GetTransactionResponse.$_createMessage);
  static GetTransactionResponse? _defaultInstance;

  @$pb.TagNumber(1)
  Transaction get transaction => $_getN(0);
  @$pb.TagNumber(1)
  set transaction(Transaction value) => $_setField(1, value);
  @$pb.TagNumber(1)
  $core.bool hasTransaction() => $_has(0);
  @$pb.TagNumber(1)
  void clearTransaction() => $_clearField(1);
  @$pb.TagNumber(1)
  Transaction ensureTransaction() => $_ensure(0);
}

const $core.bool _omitFieldNames =
    $core.bool.fromEnvironment('protobuf.omit_field_names');
const $core.bool _omitMessageNames =
    $core.bool.fromEnvironment('protobuf.omit_message_names');
