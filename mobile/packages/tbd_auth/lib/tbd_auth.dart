/// Signing in through the SSO and staying signed in: authorization code
/// with PKCE in the system browser, tokens in the platform's secure store,
/// refresh ahead of expiry and on a 401, sign-out at the issuer; all behind
/// `AuthRepository`, which a test replaces with a fake.
library;

export 'src/auth_repository.dart';
export 'src/session.dart';
export 'src/token_store.dart';
export 'src/tokens.dart';
