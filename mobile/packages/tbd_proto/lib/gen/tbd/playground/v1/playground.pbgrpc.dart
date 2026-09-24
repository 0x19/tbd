// This is a generated file - do not edit.
//
// Generated from tbd/playground/v1/playground.proto.

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

import 'playground.pb.dart' as $0;

export 'playground.pb.dart';

/// The playground service: one shared sandbox that anyone may try to break.
///
/// A small stack runs under constant synthetic load with an objective ticking
/// along. Callers spend from a global, refilling budget to inject faults — the
/// same faults the chaos tool injects in CI — and the world heals itself as
/// each one expires. Every RPC is public and unauthenticated by design, so the
/// service clamps every parameter it accepts and never takes a target address
/// from the caller.
@$pb.GrpcServiceName('tbd.playground.v1.PlaygroundService')
class PlaygroundServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  PlaygroundServiceClient(super.channel, {super.options, super.interceptors});

  /// Echo a message. Proves configuration, telemetry and fault injection reach
  /// an RPC.
  $grpc.ResponseFuture<$0.PingResponse> ping(
    $0.PingRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$ping, request, options: options);
  }

  /// The world as it stands right now.
  $grpc.ResponseFuture<$0.GetWorldResponse> getWorld(
    $0.GetWorldRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getWorld, request, options: options);
  }

  /// The world on every tick, until the caller goes away. Served as server-sent
  /// events over REST and as a subscription over the multiplexed socket.
  $grpc.ResponseStream<$0.WatchResponse> watch(
    $0.WatchRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(_$watch, $async.Stream.fromIterable([request]),
        options: options);
  }

  /// Spend budget on one fault. Refused, with a reason, when the budget is dry
  /// or the move would leave nothing healthy.
  $grpc.ResponseFuture<$0.InjectFaultResponse> injectFault(
    $0.InjectFaultRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$injectFault, request, options: options);
  }

  /// The fastest breaches recorded so far.
  $grpc.ResponseFuture<$0.ScoresResponse> scores(
    $0.ScoresRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$scores, request, options: options);
  }

  // method descriptors

  static final _$ping = $grpc.ClientMethod<$0.PingRequest, $0.PingResponse>(
      '/tbd.playground.v1.PlaygroundService/Ping',
      ($0.PingRequest value) => value.writeToBuffer(),
      $0.PingResponse.fromBuffer);
  static final _$getWorld =
      $grpc.ClientMethod<$0.GetWorldRequest, $0.GetWorldResponse>(
          '/tbd.playground.v1.PlaygroundService/GetWorld',
          ($0.GetWorldRequest value) => value.writeToBuffer(),
          $0.GetWorldResponse.fromBuffer);
  static final _$watch = $grpc.ClientMethod<$0.WatchRequest, $0.WatchResponse>(
      '/tbd.playground.v1.PlaygroundService/Watch',
      ($0.WatchRequest value) => value.writeToBuffer(),
      $0.WatchResponse.fromBuffer);
  static final _$injectFault =
      $grpc.ClientMethod<$0.InjectFaultRequest, $0.InjectFaultResponse>(
          '/tbd.playground.v1.PlaygroundService/InjectFault',
          ($0.InjectFaultRequest value) => value.writeToBuffer(),
          $0.InjectFaultResponse.fromBuffer);
  static final _$scores =
      $grpc.ClientMethod<$0.ScoresRequest, $0.ScoresResponse>(
          '/tbd.playground.v1.PlaygroundService/Scores',
          ($0.ScoresRequest value) => value.writeToBuffer(),
          $0.ScoresResponse.fromBuffer);
}

@$pb.GrpcServiceName('tbd.playground.v1.PlaygroundService')
abstract class PlaygroundServiceBase extends $grpc.Service {
  $core.String get $name => 'tbd.playground.v1.PlaygroundService';

  PlaygroundServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.PingRequest, $0.PingResponse>(
        'Ping',
        ping_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.PingRequest.fromBuffer(value),
        ($0.PingResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetWorldRequest, $0.GetWorldResponse>(
        'GetWorld',
        getWorld_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetWorldRequest.fromBuffer(value),
        ($0.GetWorldResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.WatchRequest, $0.WatchResponse>(
        'Watch',
        watch_Pre,
        false,
        true,
        ($core.List<$core.int> value) => $0.WatchRequest.fromBuffer(value),
        ($0.WatchResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.InjectFaultRequest, $0.InjectFaultResponse>(
            'InjectFault',
            injectFault_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.InjectFaultRequest.fromBuffer(value),
            ($0.InjectFaultResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ScoresRequest, $0.ScoresResponse>(
        'Scores',
        scores_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ScoresRequest.fromBuffer(value),
        ($0.ScoresResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.PingResponse> ping_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.PingRequest> $request) async {
    return ping($call, await $request);
  }

  $async.Future<$0.PingResponse> ping(
      $grpc.ServiceCall call, $0.PingRequest request);

  $async.Future<$0.GetWorldResponse> getWorld_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetWorldRequest> $request) async {
    return getWorld($call, await $request);
  }

  $async.Future<$0.GetWorldResponse> getWorld(
      $grpc.ServiceCall call, $0.GetWorldRequest request);

  $async.Stream<$0.WatchResponse> watch_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.WatchRequest> $request) async* {
    yield* watch($call, await $request);
  }

  $async.Stream<$0.WatchResponse> watch(
      $grpc.ServiceCall call, $0.WatchRequest request);

  $async.Future<$0.InjectFaultResponse> injectFault_Pre($grpc.ServiceCall $call,
      $async.Future<$0.InjectFaultRequest> $request) async {
    return injectFault($call, await $request);
  }

  $async.Future<$0.InjectFaultResponse> injectFault(
      $grpc.ServiceCall call, $0.InjectFaultRequest request);

  $async.Future<$0.ScoresResponse> scores_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.ScoresRequest> $request) async {
    return scores($call, await $request);
  }

  $async.Future<$0.ScoresResponse> scores(
      $grpc.ServiceCall call, $0.ScoresRequest request);
}
