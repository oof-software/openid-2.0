use std::collections::HashSet;
use std::sync::Mutex;

use rand::RngCore;

const NONCE_BYTES: usize = 36;
const NONCE_BASE64_LEN: usize = (NONCE_BYTES * 4) / 3;

#[derive(Debug)]
pub(crate) struct NonceSet {
    inner: Mutex<HashSet<String>>,
}
impl NonceSet {
    pub(crate) async fn validate_and_remove(&self, nonce: &str) -> bool {
        self.inner.lock().unwrap().remove(nonce)
    }
    pub(crate) async fn insert_new(&self) -> String {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD as Base64;
        use base64::Engine;

        let mut nonce_bytes = [0u8; NONCE_BYTES];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        let mut nonce_base64 = String::with_capacity(NONCE_BASE64_LEN);
        Base64.encode_string(nonce_bytes, &mut nonce_base64);
        let nonce_base64_copy = nonce_base64.clone();

        self.inner.lock().unwrap().insert(nonce_base64);

        nonce_base64_copy
    }
    pub(crate) fn new() -> NonceSet {
        NonceSet {
            inner: Mutex::new(HashSet::with_capacity(128)),
        }
    }
}
