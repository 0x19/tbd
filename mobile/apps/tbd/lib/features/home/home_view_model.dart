import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tbd_api/tbd_api.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:tbd_mobile/app/providers.dart';
import 'package:tbd_proto/tbd_proto.dart';

/// One round-trip's outcome and how long it took, as the screen shows it.
final class RoundTrip<T> {
  const RoundTrip(this.result, this.elapsed);

  final Result<T> result;
  final Duration elapsed;
}

/// The two hellos: who the edge says we are (REST) and the protocol's ping
/// (gRPC). Each fails on its own; neither throws.
final class HelloState {
  const HelloState({required this.me, required this.ping});

  final RoundTrip<Me> me;
  final RoundTrip<PingResponse> ping;
}

/// Loads both on first watch; `refresh` asks again.
final homeViewModelProvider = AsyncNotifierProvider<HomeViewModel, HelloState>(
  HomeViewModel.new,
);

class HomeViewModel extends AsyncNotifier<HelloState> {
  @override
  Future<HelloState> build() async {
    final api = ref.watch(protocolApiProvider);
    final (me, ping) = await (
      _timed(api.me),
      _timed(() => api.ping('hello from the phone')),
    ).wait;
    return HelloState(me: me, ping: ping);
  }

  Future<void> refresh() async {
    state = const AsyncLoading();
    ref.invalidateSelf();
    await future;
  }

  static Future<RoundTrip<T>> _timed<T>(
    Future<Result<T>> Function() call,
  ) async {
    final sw = Stopwatch()..start();
    final r = await call();
    return RoundTrip(r, sw.elapsed);
  }
}
