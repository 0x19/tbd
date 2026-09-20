// This is a generated file - do not edit.
//
// Generated from tbd/ledger/v1/ledger.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:async' as $async;
import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'package:protobuf/protobuf.dart' as $pb;

import 'ledger.pb.dart' as $0;

export 'ledger.pb.dart';

/// The facts ledger: append-only facts about an opaque subject, with
/// provenance, tombstones and erasure (docs/ledger/README.md). The subject id
/// is the ledger's own opaque uuid, never a person identifier.
@$pb.GrpcServiceName('tbd.ledger.v1.LedgerService')
class LedgerServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  LedgerServiceClient(super.channel, {super.options, super.interceptors});

  /// Echo a message. Proves configuration, telemetry and fault injection reach
  /// an RPC; reports which store backs the service.
  $grpc.ResponseFuture<$0.PingResponse> ping(
    $0.PingRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$ping, request, options: options);
  }

  /// Append one fact. The subject is created on its first write. A matching
  /// idempotency_key replays the earlier fact (`replayed = true`); reused with
  /// different content it is ABORTED.
  $grpc.ResponseFuture<$0.AppendResponse> append(
    $0.AppendRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$append, request, options: options);
  }

  /// The latest valued, unexpired fact per (path, source), under the given
  /// scopes. Pages by cursor.
  $grpc.ResponseFuture<$0.CurrentResponse> current(
    $0.CurrentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$current, request, options: options);
  }

  /// Everything the ledger still holds, tombstones included; `at` cuts history
  /// at an instant (a retracted value is absent from every cut).
  $grpc.ResponseFuture<$0.HistoryResponse> history(
    $0.HistoryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$history, request, options: options);
  }

  /// Delete every valued fact of (path, source) and append a tombstone.
  $grpc.ResponseFuture<$0.RetractResponse> retract(
    $0.RetractRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$retract, request, options: options);
  }

  /// Request erasure: every read and write answers FAILED_PRECONDITION from
  /// now; the cascade runs after the grace window.
  $grpc.ResponseFuture<$0.EraseResponse> erase(
    $0.EraseRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$erase, request, options: options);
  }

  /// Cancel a pending erasure and reopen the subject. NOT_FOUND once executed.
  $grpc.ResponseFuture<$0.RestoreResponse> restore(
    $0.RestoreRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$restore, request, options: options);
  }

  // method descriptors

  static final _$ping = $grpc.ClientMethod<$0.PingRequest, $0.PingResponse>(
      '/tbd.ledger.v1.LedgerService/Ping',
      ($0.PingRequest value) => value.writeToBuffer(),
      $0.PingResponse.fromBuffer);
  static final _$append =
      $grpc.ClientMethod<$0.AppendRequest, $0.AppendResponse>(
          '/tbd.ledger.v1.LedgerService/Append',
          ($0.AppendRequest value) => value.writeToBuffer(),
          $0.AppendResponse.fromBuffer);
  static final _$current =
      $grpc.ClientMethod<$0.CurrentRequest, $0.CurrentResponse>(
          '/tbd.ledger.v1.LedgerService/Current',
          ($0.CurrentRequest value) => value.writeToBuffer(),
          $0.CurrentResponse.fromBuffer);
  static final _$history =
      $grpc.ClientMethod<$0.HistoryRequest, $0.HistoryResponse>(
          '/tbd.ledger.v1.LedgerService/History',
          ($0.HistoryRequest value) => value.writeToBuffer(),
          $0.HistoryResponse.fromBuffer);
  static final _$retract =
      $grpc.ClientMethod<$0.RetractRequest, $0.RetractResponse>(
          '/tbd.ledger.v1.LedgerService/Retract',
          ($0.RetractRequest value) => value.writeToBuffer(),
          $0.RetractResponse.fromBuffer);
  static final _$erase = $grpc.ClientMethod<$0.EraseRequest, $0.EraseResponse>(
      '/tbd.ledger.v1.LedgerService/Erase',
      ($0.EraseRequest value) => value.writeToBuffer(),
      $0.EraseResponse.fromBuffer);
  static final _$restore =
      $grpc.ClientMethod<$0.RestoreRequest, $0.RestoreResponse>(
          '/tbd.ledger.v1.LedgerService/Restore',
          ($0.RestoreRequest value) => value.writeToBuffer(),
          $0.RestoreResponse.fromBuffer);
}

@$pb.GrpcServiceName('tbd.ledger.v1.LedgerService')
abstract class LedgerServiceBase extends $grpc.Service {
  $core.String get $name => 'tbd.ledger.v1.LedgerService';

  LedgerServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.PingRequest, $0.PingResponse>(
        'Ping',
        ping_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.PingRequest.fromBuffer(value),
        ($0.PingResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.AppendRequest, $0.AppendResponse>(
        'Append',
        append_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.AppendRequest.fromBuffer(value),
        ($0.AppendResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CurrentRequest, $0.CurrentResponse>(
        'Current',
        current_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.CurrentRequest.fromBuffer(value),
        ($0.CurrentResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.HistoryRequest, $0.HistoryResponse>(
        'History',
        history_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.HistoryRequest.fromBuffer(value),
        ($0.HistoryResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RetractRequest, $0.RetractResponse>(
        'Retract',
        retract_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RetractRequest.fromBuffer(value),
        ($0.RetractResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.EraseRequest, $0.EraseResponse>(
        'Erase',
        erase_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.EraseRequest.fromBuffer(value),
        ($0.EraseResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RestoreRequest, $0.RestoreResponse>(
        'Restore',
        restore_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.RestoreRequest.fromBuffer(value),
        ($0.RestoreResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.PingResponse> ping_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.PingRequest> $request) async {
    return ping($call, await $request);
  }

  $async.Future<$0.PingResponse> ping(
      $grpc.ServiceCall call, $0.PingRequest request);

  $async.Future<$0.AppendResponse> append_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.AppendRequest> $request) async {
    return append($call, await $request);
  }

  $async.Future<$0.AppendResponse> append(
      $grpc.ServiceCall call, $0.AppendRequest request);

  $async.Future<$0.CurrentResponse> current_Pre($grpc.ServiceCall $call,
      $async.Future<$0.CurrentRequest> $request) async {
    return current($call, await $request);
  }

  $async.Future<$0.CurrentResponse> current(
      $grpc.ServiceCall call, $0.CurrentRequest request);

  $async.Future<$0.HistoryResponse> history_Pre($grpc.ServiceCall $call,
      $async.Future<$0.HistoryRequest> $request) async {
    return history($call, await $request);
  }

  $async.Future<$0.HistoryResponse> history(
      $grpc.ServiceCall call, $0.HistoryRequest request);

  $async.Future<$0.RetractResponse> retract_Pre($grpc.ServiceCall $call,
      $async.Future<$0.RetractRequest> $request) async {
    return retract($call, await $request);
  }

  $async.Future<$0.RetractResponse> retract(
      $grpc.ServiceCall call, $0.RetractRequest request);

  $async.Future<$0.EraseResponse> erase_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.EraseRequest> $request) async {
    return erase($call, await $request);
  }

  $async.Future<$0.EraseResponse> erase(
      $grpc.ServiceCall call, $0.EraseRequest request);

  $async.Future<$0.RestoreResponse> restore_Pre($grpc.ServiceCall $call,
      $async.Future<$0.RestoreRequest> $request) async {
    return restore($call, await $request);
  }

  $async.Future<$0.RestoreResponse> restore(
      $grpc.ServiceCall call, $0.RestoreRequest request);
}
