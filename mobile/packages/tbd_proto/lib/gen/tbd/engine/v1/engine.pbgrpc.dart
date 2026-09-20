// This is a generated file - do not edit.
//
// Generated from tbd/engine/v1/engine.proto.

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

import 'engine.pb.dart' as $0;

export 'engine.pb.dart';

/// The engine is the streaming compute service. Every call is scoped to a
/// subject: the entity the engine is reasoning about.
@$pb.GrpcServiceName('tbd.engine.v1.EngineService')
class EngineServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  EngineServiceClient(super.channel, {super.options, super.interceptors});

  /// Score a single subject once. Unary.
  $grpc.ResponseFuture<$0.EvaluateResponse> evaluate(
    $0.EvaluateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$evaluate, request, options: options);
  }

  /// Subscribe to engine events for a subject. Server streaming.
  $grpc.ResponseStream<$0.SubscribeResponse> subscribe(
    $0.SubscribeRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$subscribe, $async.Stream.fromIterable([request]),
        options: options);
  }

  /// Long-lived bidirectional session. Client and server both stream frames.
  $grpc.ResponseStream<$0.SessionResponse> session(
    $async.Stream<$0.SessionRequest> request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(_$session, request, options: options);
  }

  // method descriptors

  static final _$evaluate =
      $grpc.ClientMethod<$0.EvaluateRequest, $0.EvaluateResponse>(
          '/tbd.engine.v1.EngineService/Evaluate',
          ($0.EvaluateRequest value) => value.writeToBuffer(),
          $0.EvaluateResponse.fromBuffer);
  static final _$subscribe =
      $grpc.ClientMethod<$0.SubscribeRequest, $0.SubscribeResponse>(
          '/tbd.engine.v1.EngineService/Subscribe',
          ($0.SubscribeRequest value) => value.writeToBuffer(),
          $0.SubscribeResponse.fromBuffer);
  static final _$session =
      $grpc.ClientMethod<$0.SessionRequest, $0.SessionResponse>(
          '/tbd.engine.v1.EngineService/Session',
          ($0.SessionRequest value) => value.writeToBuffer(),
          $0.SessionResponse.fromBuffer);
}

@$pb.GrpcServiceName('tbd.engine.v1.EngineService')
abstract class EngineServiceBase extends $grpc.Service {
  $core.String get $name => 'tbd.engine.v1.EngineService';

  EngineServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.EvaluateRequest, $0.EvaluateResponse>(
        'Evaluate',
        evaluate_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.EvaluateRequest.fromBuffer(value),
        ($0.EvaluateResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SubscribeRequest, $0.SubscribeResponse>(
        'Subscribe',
        subscribe_Pre,
        false,
        true,
        ($core.List<$core.int> value) => $0.SubscribeRequest.fromBuffer(value),
        ($0.SubscribeResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SessionRequest, $0.SessionResponse>(
        'Session',
        session,
        true,
        true,
        ($core.List<$core.int> value) => $0.SessionRequest.fromBuffer(value),
        ($0.SessionResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.EvaluateResponse> evaluate_Pre($grpc.ServiceCall $call,
      $async.Future<$0.EvaluateRequest> $request) async {
    return evaluate($call, await $request);
  }

  $async.Future<$0.EvaluateResponse> evaluate(
      $grpc.ServiceCall call, $0.EvaluateRequest request);

  $async.Stream<$0.SubscribeResponse> subscribe_Pre($grpc.ServiceCall $call,
      $async.Future<$0.SubscribeRequest> $request) async* {
    yield* subscribe($call, await $request);
  }

  $async.Stream<$0.SubscribeResponse> subscribe(
      $grpc.ServiceCall call, $0.SubscribeRequest request);

  $async.Stream<$0.SessionResponse> session(
      $grpc.ServiceCall call, $async.Stream<$0.SessionRequest> request);
}
