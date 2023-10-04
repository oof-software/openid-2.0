//! A set of nonces
//!
//! # Birtday problem approximation
//!
//! <https://planetmath.org/approximatingthebirthdayproblem>
//!
//! <https://brilliant.org/wiki/birthday-paradox/>
//!
//! Given a nonce length of 36 bytes (324 bits) and assuming a truly random uniform distribution.
//!
//! At least 2^144 nonces must be generated to have a collision probability of >50% on average.
//!
//! Which would require generating more than 2^89 nonces every millisecond for 1'000'000 years.

use std::borrow::Borrow;
use std::collections::HashMap;

use chrono::Utc;
use parking_lot::Mutex;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;

const NONCE_BYTES: usize = 36;
const NONCE_BASE64_LEN: usize = (NONCE_BYTES * 4) / 3;

/// 5 Minutes between us redirecting the user to steam
/// and him getting redirected to the callback function
/// seems reasonable.
const NONCE_MAX_AGE_MS: i64 = 5_000_000;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub(crate) struct Nonce {
    inner: String,
}
impl Borrow<str> for Nonce {
    fn borrow(&self) -> &str {
        &self.inner
    }
}

#[derive(Debug)]
struct Metadata {
    time: i64,
}

impl Metadata {
    fn new(_nonce: &Nonce) -> Metadata {
        let now = Utc::now().timestamp_millis();
        Metadata { time: now }
    }
    const fn is_expired(&self, now: i64) -> bool {
        now - self.time > NONCE_MAX_AGE_MS
    }
}

impl Nonce {
    fn random() -> Nonce {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD as Base64;
        use base64::Engine;

        let mut nonce_bytes = [0u8; NONCE_BYTES];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        let mut nonce_base64 = String::with_capacity(NONCE_BASE64_LEN);
        Base64.encode_string(nonce_bytes, &mut nonce_base64);

        Nonce {
            inner: nonce_base64,
        }
    }
    pub(crate) fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

#[derive(Error, Debug)]
pub(crate) enum NonceError {
    #[error("the nonce is invalid")]
    Invalid,
    #[error("the nonce has expired")]
    Expired,
}

#[derive(Debug)]
pub(crate) struct NonceSet {
    inner: Mutex<HashMap<Nonce, Metadata>>,
}
impl NonceSet {
    /// Remove all expired nonces
    pub(crate) fn remove_expired_nonces(&self) {
        let now = Utc::now().timestamp_millis();
        self.inner.lock().retain(|_, meta| !meta.is_expired(now));
    }

    /// Validate the nonce and remove it, if it is valid
    pub(crate) fn validate_and_remove(&self, nonce: &str) -> Result<(), NonceError> {
        let Some(nonce) = self.inner.lock().remove(nonce) else {
            return Err(NonceError::Invalid);
        };
        if nonce.is_expired(Utc::now().timestamp_millis()) {
            return Err(NonceError::Expired);
        }
        Ok(())
    }

    /// Check if the nonce is valid (as in not expired)
    pub(crate) fn validate(&self, nonce: &str) -> Result<(), NonceError> {
        if self.inner.lock().contains_key(nonce) {
            Ok(())
        } else {
            Err(NonceError::Expired)
        }
    }

    /// Look for the given nonce and replace it
    pub(crate) fn replace(&self, old: &str) -> Result<Nonce, NonceError> {
        let new_nonce = Nonce::random();
        let new_meta = Metadata::new(&new_nonce);
        let new_nonce_copy = new_nonce.clone();

        {
            let mut lock = self.inner.lock();
            if lock.remove(old).is_none() {
                return Err(NonceError::Invalid);
            }
            let _ = lock.insert(new_nonce, new_meta);
        }

        Ok(new_nonce_copy)
    }

    /// Insert a new nonce
    pub(crate) fn insert_new(&self) -> Nonce {
        let nonce = Nonce::random();
        let meta = Metadata::new(&nonce);
        let nonce_copy = nonce.clone();

        let _ = self.inner.lock().insert(nonce, meta);

        nonce_copy
    }

    /// Create a new thingy
    pub(crate) fn new() -> NonceSet {
        NonceSet {
            inner: Mutex::new(HashMap::with_capacity(128)),
        }
    }
}
