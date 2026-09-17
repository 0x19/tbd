//! The RS256 JWT that authorizes every Enable Banking call.
//!
//! Signed locally against the application's RSA key. There is no token
//! endpoint and no round trip: a bad key shows up as a 401 on the first real
//! call, not as a login failure. The prototype (`prototype/bank/eb/token.py`)
//! proved the exact claims Enable Banking accepts; this is that, in Rust.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ring::{rand::SystemRandom, signature::RsaKeyPair};

use super::error::ProviderError;

const ISSUER: &str = "enablebanking.com";
const AUDIENCE: &str = "api.enablebanking.com";
/// The documented maximum is 86400s. An hour bounds the damage if a token
/// leaks into a log somewhere.
const TTL: Duration = Duration::from_secs(3600);
/// `iat` is backdated so a few seconds of clock skew against their servers
/// is harmless.
const SKEW: Duration = Duration::from_secs(30);

/// A loaded signing key plus the application it belongs to.
///
/// Holds the key pair, not the PEM: the private key bytes are parsed once and
/// never kept as text. `Debug` prints the application id only.
pub struct Signer {
    application_id: String,
    key: RsaKeyPair,
    rng: SystemRandom,
}

impl std::fmt::Debug for Signer {
    // The key is redacted on purpose, which the lint cannot know.
    #[allow(clippy::missing_fields_in_debug)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Signer")
            .field("application_id", &self.application_id)
            .field("key", &"<rsa private key>")
            .field("rng", &"SystemRandom")
            .finish()
    }
}

impl Signer {
    /// Build from a PKCS#8 PEM (`-----BEGIN PRIVATE KEY-----`).
    ///
    /// That is what `openssl genpkey` and Enable Banking's registration page
    /// produce. A PKCS#1 key (`BEGIN RSA PRIVATE KEY`) is refused by name so
    /// the fix is one `openssl pkcs8 -topk8` away rather than a mystery.
    ///
    /// # Errors
    /// The PEM is not a PKCS#8 RSA private key, or the application id is empty.
    pub fn from_pem(application_id: &str, pem: &str) -> Result<Self, ProviderError> {
        if application_id.trim().is_empty() {
            return Err(ProviderError::Config {
                field: "application_id",
                reason: "empty".into(),
            });
        }
        if pem.contains("BEGIN RSA PRIVATE KEY") {
            return Err(ProviderError::Config {
                field: "private_key",
                reason: "PKCS#1 key; convert with `openssl pkcs8 -topk8 -nocrypt`".into(),
            });
        }
        let der = pem_to_der(pem)?;
        let key = RsaKeyPair::from_pkcs8(&der).map_err(|e| ProviderError::Config {
            field: "private_key",
            reason: format!("not a PKCS#8 RSA key: {e}"),
        })?;
        Ok(Self {
            application_id: application_id.to_owned(),
            key,
            rng: SystemRandom::new(),
        })
    }

    /// The application id, which is also the token's `kid`.
    #[must_use]
    pub fn application_id(&self) -> &str {
        &self.application_id
    }

    /// Mint a token valid from now.
    ///
    /// # Errors
    /// The clock is before 1970, or signing fails (which ring reserves for a
    /// broken RNG).
    pub fn mint(&self) -> Result<String, ProviderError> {
        let now =
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| ProviderError::Config {
                    field: "clock",
                    reason: e.to_string(),
                })?;
        self.mint_at(now)
    }

    /// Mint for a given "now". Split out so a test can pin the clock.
    ///
    /// # Errors
    /// Signing fails.
    pub fn mint_at(&self, now: Duration) -> Result<String, ProviderError> {
        let header = serde_json::json!({
            "typ": "JWT",
            "alg": "RS256",
            "kid": self.application_id,
        });
        let claims = serde_json::json!({
            "iss": ISSUER,
            "aud": AUDIENCE,
            "iat": now.saturating_sub(SKEW).as_secs(),
            "exp": (now + TTL).as_secs(),
        });
        let signing_input = format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(header.to_string()),
            URL_SAFE_NO_PAD.encode(claims.to_string())
        );
        let mut signature = vec![0; self.key.public().modulus_len()];
        self.key
            .sign(
                &ring::signature::RSA_PKCS1_SHA256,
                &self.rng,
                signing_input.as_bytes(),
                &mut signature,
            )
            .map_err(|e| ProviderError::Config {
                field: "private_key",
                reason: format!("signing failed: {e}"),
            })?;
        Ok(format!(
            "{signing_input}.{}",
            URL_SAFE_NO_PAD.encode(&signature)
        ))
    }
}

/// Strip the PEM armour and decode the body.
///
/// Deliberately not a PEM crate: the format is two marker lines around
/// base64, and pulling in a parser for that is one more thing `cargo deny`
/// has to vet.
fn pem_to_der(pem: &str) -> Result<Vec<u8>, ProviderError> {
    let body: String = pem
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("-----"))
        .collect();
    if body.is_empty() {
        return Err(ProviderError::Config {
            field: "private_key",
            reason: "no PEM body".into(),
        });
    }
    base64::engine::general_purpose::STANDARD
        .decode(body)
        .map_err(|e| ProviderError::Config {
            field: "private_key",
            reason: format!("PEM body is not base64: {e}"),
        })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// A throwaway 2048-bit key generated for these tests (`.pkcs8`, because
    /// `*.pem` is gitignored to keep real keys out). It authorizes
    /// nothing anywhere.
    const TEST_KEY: &str = include_str!("../../tests/fixtures/test-signing-key.pkcs8");

    fn decode_segment(segment: &str) -> serde_json::Value {
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(segment).unwrap()).unwrap()
    }

    #[test]
    fn the_token_has_the_claims_enable_banking_expects() {
        let signer = Signer::from_pem("app-1234", TEST_KEY).unwrap();
        let token = signer.mint_at(Duration::from_secs(1_700_000_000)).unwrap();
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3, "a JWT is three segments");

        let header = decode_segment(parts[0]);
        assert_eq!(header["alg"], "RS256");
        assert_eq!(header["typ"], "JWT");
        assert_eq!(header["kid"], "app-1234");

        let claims = decode_segment(parts[1]);
        assert_eq!(claims["iss"], "enablebanking.com");
        assert_eq!(claims["aud"], "api.enablebanking.com");
        assert_eq!(claims["iat"], 1_700_000_000 - 30);
        assert_eq!(claims["exp"], 1_700_000_000 + 3600);
    }

    #[test]
    fn the_signature_verifies_against_the_public_key() {
        let signer = Signer::from_pem("app", TEST_KEY).unwrap();
        let token = signer.mint().unwrap();
        let (input, sig) = token.rsplit_once('.').unwrap();
        let sig = URL_SAFE_NO_PAD.decode(sig).unwrap();
        let public = ring::signature::UnparsedPublicKey::new(
            &ring::signature::RSA_PKCS1_2048_8192_SHA256,
            signer.key.public().as_ref(),
        );
        public.verify(input.as_bytes(), &sig).unwrap();
    }

    #[test]
    fn a_pkcs1_key_is_refused_with_the_fix_named() {
        let e = Signer::from_pem(
            "app",
            "-----BEGIN RSA PRIVATE KEY-----\nAAAA\n-----END RSA PRIVATE KEY-----",
        )
        .unwrap_err();
        assert!(e.to_string().contains("pkcs8"), "{e}");
    }

    #[test]
    fn garbage_is_refused_before_ring_sees_it() {
        assert!(Signer::from_pem("app", "not a key").is_err());
        assert!(Signer::from_pem("", TEST_KEY).is_err());
    }

    #[test]
    fn debug_never_prints_the_key() {
        let signer = Signer::from_pem("app", TEST_KEY).unwrap();
        let shown = format!("{signer:?}");
        assert!(!shown.contains("BEGIN"));
        assert!(!shown.contains(TEST_KEY.lines().nth(1).unwrap()));
    }
}
