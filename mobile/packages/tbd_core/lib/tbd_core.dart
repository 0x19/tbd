/// The layer below every feature: configuration read from the build, a
/// `Result` and the errors it carries, telemetry primitives, and time.
///
/// Pure Dart on purpose. Nothing here imports Flutter or a platform plugin,
/// so it runs in a unit test, a CLI tool and every app alike; what needs a
/// platform (secure storage, a browser) lives in `tbd_auth` and `tbd_api`.
library;

export 'src/clock.dart';
export 'src/env.dart';
export 'src/error.dart';
export 'src/result.dart';
export 'src/telemetry.dart';
