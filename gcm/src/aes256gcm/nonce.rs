use aes_gcm::aead::OsRng;
use aes_gcm::{AeadCore, Aes256Gcm, aead::Nonce};

pub fn generate_nonce() -> Nonce<Aes256Gcm> {
    Aes256Gcm::generate_nonce(OsRng)
}
