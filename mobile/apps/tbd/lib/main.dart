import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tbd_auth/tbd_auth.dart';
import 'package:tbd_mobile/app/app.dart';
import 'package:tbd_mobile/app/env.dart';
import 'package:tbd_mobile/app/providers.dart';
import 'package:tbd_mobile/app/router.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  // A build without its configuration fails here, naming the key, not on the
  // first screen that needs a URL.
  final env = readEnv();
  final session = Session(
    repository: AppAuthRepository(env),
    store: SecureTokenStore(),
  );
  // Not awaited: the router shows the splash until the store has been read.
  unawaited(session.restore());
  runApp(
    ProviderScope(
      overrides: [
        envProvider.overrideWithValue(env),
        sessionProvider.overrideWithValue(session),
      ],
      child: App(router: buildRouter(session)),
    ),
  );
}
