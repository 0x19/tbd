//! Credentials at rest.
//!
//! ChaCha20-Poly1305 with a random nonce per seal, the key from configuration
//! and never from the database. A backup of the database is then a backup of
//! ciphertext; the key is a Kubernetes Secret and lives only in the service's
//! environment.

use base64::{Engine, engine::general_purpose::STANDARD};
use ring::{
    aead::{Aad, CHACHA20_POLY1305, LessSafeKey, NONCE_LEN, Nonce, UnboundKey},
    rand::{SecureRandom, SystemRandom},
};

/// Why sealing or opening failed.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// The configured key is not 32 bytes of base64.
    #[error("connector key: {0}")]
    Key(String),
    /// The ciphertext is not ours, or was tampered with.
    #[error("credentials could not be opened")]
    Open,
    /// The RNG failed, which ring reserves for a broken system.
    #[error("rng")]
    Rng,
}

/// The sealing key. `Debug` prints nothing of it.
pub struct Sealer {
    key: LessSafeKey,
    rng: SystemRandom,
}

impl std::fmt::Debug for Sealer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Sealer(<key>)")
    }
}

impl Sealer {
    /// From the base64 of 32 random bytes (`openssl rand -base64 32`).
    ///
    /// # Errors
    /// Not base64, or not 32 bytes.
    pub fn from_base64(key: &str) -> Result<Self, CryptoError> {
        let bytes = STANDARD
            .decode(key.trim())
            .map_err(|e| CryptoError::Key(format!("not base64: {e}")))?;
        if bytes.len() != 32 {
            return Err(CryptoError::Key(format!(
                "want 32 bytes, got {}",
                bytes.len()
            )));
        }
        let unbound = UnboundKey::new(&CHACHA20_POLY1305, &bytes)
            .map_err(|_| CryptoError::Key("unusable".into()))?;
        Ok(Self {
            key: LessSafeKey::new(unbound),
            rng: SystemRandom::new(),
        })
    }

    /// Encrypt. The output is `nonce || ciphertext || tag`; `aad` binds the
    /// blob to its row so a ciphertext moved to another connector's row does
    /// not open.
    ///
    /// # Errors
    /// The RNG.
    pub fn seal(&self, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce = [0u8; NONCE_LEN];
        self.rng.fill(&mut nonce).map_err(|_| CryptoError::Rng)?;
        let mut out = plaintext.to_vec();
        self.key
            .seal_in_place_append_tag(
                Nonce::assume_unique_for_key(nonce),
                Aad::from(aad),
                &mut out,
            )
            .map_err(|_| CryptoError::Rng)?;
        let mut blob = nonce.to_vec();
        blob.extend_from_slice(&out);
        Ok(blob)
    }

    /// Decrypt what [`Sealer::seal`] produced, under the same `aad`.
    ///
    /// # Errors
    /// Not ours, wrong row, or tampered with.
    pub fn open(&self, blob: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if blob.len() < NONCE_LEN {
            return Err(CryptoError::Open);
        }
        let (nonce, rest) = blob.split_at(NONCE_LEN);
        let nonce: [u8; NONCE_LEN] = nonce.try_into().map_err(|_| CryptoError::Open)?;
        let mut buf = rest.to_vec();
        let opened = self
            .key
            .open_in_place(
                Nonce::assume_unique_for_key(nonce),
                Aad::from(aad),
                &mut buf,
            )
            .map_err(|_| CryptoError::Open)?;
        Ok(opened.to_vec())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn sealer() -> Sealer {
        Sealer::from_base64(&STANDARD.encode([7u8; 32])).unwrap()
    }

    #[test]
    fn round_trips_and_binds_to_the_row() {
        let s = sealer();
        let blob = s.seal(b"refresh-token", b"row-1").unwrap();
        assert_eq!(s.open(&blob, b"row-1").unwrap(), b"refresh-token");
        assert!(s.open(&blob, b"row-2").is_err(), "moved to another row");
        let mut tampered = blob.clone();
        *tampered.last_mut().unwrap() ^= 1;
        assert!(s.open(&tampered, b"row-1").is_err());
    }

    #[test]
    fn two_seals_of_one_secret_differ() {
        let s = sealer();
        assert_ne!(s.seal(b"x", b"r").unwrap(), s.seal(b"x", b"r").unwrap());
    }

    #[test]
    fn a_short_or_wrong_key_is_refused_by_name() {
        assert!(Sealer::from_base64("nope").is_err());
        assert!(Sealer::from_base64(&STANDARD.encode([1u8; 16])).is_err());
    }
}
