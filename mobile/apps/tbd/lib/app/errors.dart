import 'package:tbd_core/tbd_core.dart';
import 'package:tbd_mobile/l10n/app_localizations.dart';

/// The closed error set as a sentence in the person's language. The
/// server's own words stay in the log; a person sees what to do.
String describe(AppLocalizations l, AppError e) => switch (e) {
  NetworkError() => l.errorNetwork,
  UnauthenticatedError() => l.errorUnauthenticated,
  ForbiddenError() => l.errorForbidden,
  ServerError(:final status) => l.errorServer(status),
  CancelledError() => l.errorCancelled,
  ConfigError() => l.errorConfig,
};
