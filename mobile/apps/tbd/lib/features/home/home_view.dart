import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_core/tbd_core.dart';
import 'package:tbd_mobile/app/errors.dart';
import 'package:tbd_mobile/app/providers.dart';
import 'package:tbd_mobile/app/router.dart';
import 'package:tbd_mobile/features/home/home_view_model.dart';
import 'package:tbd_mobile/l10n/app_localizations.dart';
import 'package:tbd_ui/tbd_ui.dart';

/// Hello, by the name the ID token carries, then the backend's own hello:
/// `/v1/me` over REST and `Ping` over gRPC, each with its round-trip time.
/// The whole chain on one screen: proto, Rust, Envoy, Dart, one token.
class HomeView extends ConsumerWidget {
  const HomeView({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l = AppLocalizations.of(context);
    final c = TbdColors.of(context);
    final state = ref.watch(sessionStateProvider);
    final who = state is SignedIn ? state.principal : null;
    final hello = ref.watch(homeViewModelProvider);
    return Scaffold(
      appBar: AppBar(
        title: Text(l.helloTitle),
        actions: [
          IconButton(
            tooltip: l.openSettings,
            icon: const Icon(Icons.settings_outlined),
            onPressed: () => context.push(Routes.settings),
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: ref.read(homeViewModelProvider.notifier).refresh,
        child: ListView(
          padding: const EdgeInsets.all(24),
          children: [
            Text(
              l.helloName(who?.displayName ?? ''),
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            const SizedBox(height: 4),
            Text(l.helloBody, style: TextStyle(color: c.mutedForeground)),
            const SizedBox(height: 24),
            switch (hello) {
              AsyncData(:final value) => _Hellos(value),
              AsyncError() => const _Hellos(null),
              _ => _Loading(l.helloLoading),
            },
            const SizedBox(height: 24),
            OutlinedButton.icon(
              onPressed: hello.isLoading
                  ? null
                  : ref.read(homeViewModelProvider.notifier).refresh,
              icon: const Icon(Icons.refresh),
              label: Text(l.helloRefresh),
            ),
          ],
        ),
      ),
    );
  }
}

class _Loading extends StatelessWidget {
  const _Loading(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Row(
    children: [
      const SizedBox.square(
        dimension: 16,
        child: CircularProgressIndicator(strokeWidth: 2),
      ),
      const SizedBox(width: 12),
      Text(text),
    ],
  );
}

class _Hellos extends StatelessWidget {
  const _Hellos(this.state);
  final HelloState? state;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final s = state;
    if (s == null) return const SizedBox.shrink();
    return Column(
      children: [
        _Card(
          title: l.helloRest,
          elapsed: s.me.elapsed,
          child: s.me.result.when(
            ok: (me) => Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(me.subject, style: mono(context)),
                Text(
                  l.helloRole(me.role ?? me.kind),
                  style: TextStyle(
                    color: TbdColors.of(context).mutedForeground,
                  ),
                ),
              ],
            ),
            err: _Error.new,
          ),
        ),
        const SizedBox(height: 12),
        _Card(
          title: l.helloGrpc,
          elapsed: s.ping.elapsed,
          child: s.ping.result.when(
            ok: (p) => Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(p.message, style: mono(context)),
                Text(
                  l.helloVersion(p.protocolVersion),
                  style: TextStyle(
                    color: TbdColors.of(context).mutedForeground,
                  ),
                ),
              ],
            ),
            err: _Error.new,
          ),
        ),
      ],
    );
  }
}

class _Card extends StatelessWidget {
  const _Card({
    required this.title,
    required this.elapsed,
    required this.child,
  });
  final String title;
  final Duration elapsed;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final c = TbdColors.of(context);
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    title,
                    style: Theme.of(context).textTheme.titleSmall,
                  ),
                ),
                Text(
                  l.helloMillis(elapsed.inMilliseconds),
                  style: mono(context, size: 12, color: c.mutedForeground),
                ),
              ],
            ),
            const SizedBox(height: 8),
            child,
          ],
        ),
      ),
    );
  }
}

class _Error extends StatelessWidget {
  const _Error(this.error);
  final AppError error;

  @override
  Widget build(BuildContext context) => Text(
    describe(AppLocalizations.of(context), error),
    style: TextStyle(color: TbdColors.of(context).destructive),
  );
}
