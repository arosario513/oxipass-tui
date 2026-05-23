#[cfg(test)]
extern crate color_eyre;
extern crate gcm;

use aes_gcm::{AeadCore, Aes256Gcm, KeyInit, aead::OsRng};
use gcm::prelude::*;

#[test]
fn gcm_encrypt_decrypt() -> Result<(), Error> {
    let plain = b"Hello, world!";
    let key = Aes256Gcm::generate_key(OsRng);
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let enc = encrypt(plain.as_ref(), key, nonce)?;
    let dec = decrypt(&enc, key)?;
    assert_eq!(dec, plain);
    Ok(())
}

#[test]
fn decrypt_wrong_key_fails() {
    let plain = b"Hello, world!";
    let key = Aes256Gcm::generate_key(OsRng);
    let wrong_key = Aes256Gcm::generate_key(OsRng);
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let enc = encrypt(plain.as_ref(), key, nonce).unwrap();
    assert!(decrypt(&enc, wrong_key).is_err());
}

#[test]
fn decrypt_too_short_fails() {
    let key = Aes256Gcm::generate_key(OsRng);
    assert!(matches!(decrypt(&[0u8; 11], key), Err(EncDataTooshort)));
}

#[test]
fn decrypt_empty_fails() {
    let key = Aes256Gcm::generate_key(OsRng);
    assert!(matches!(decrypt(&[], key), Err(EncDataTooshort)));
}

#[test]
fn encrypt_decrypt_empty_plaintext() -> Result<(), Error> {
    let key = Aes256Gcm::generate_key(OsRng);
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let enc = encrypt(&[], key, nonce)?;
    let dec = decrypt(&enc, key)?;
    assert_eq!(dec, b"");
    Ok(())
}

#[test]
fn derive_key_is_nondeterministic() -> Result<(), Error> {
    let (key1, _) = derive_key("password")?;
    let (key2, _) = derive_key("password")?;
    assert_ne!(key1, key2);
    Ok(())
}

#[test]
fn derive_key_with_salt_is_deterministic() -> Result<(), Error> {
    let (key1, salt) = derive_key("password")?;
    let key2 = derive_key_with_salt("password", &salt)?;
    assert_eq!(key1, key2);
    Ok(())
}

#[test]
fn derive_key_with_salt_wrong_password_differs() -> Result<(), Error> {
    let (_, salt) = derive_key("password")?;
    let key1 = derive_key_with_salt("password", &salt)?;
    let key2 = derive_key_with_salt("wrongpassword", &salt)?;
    assert_ne!(key1, key2);
    Ok(())
}

#[test]
fn encrypt_decrypt_with_derived_key() -> Result<(), Error> {
    let plain = b"Hello, world!";
    let (key_bytes, salt) = derive_key("my password")?;
    let nonce = Aes256Gcm::generate_nonce(OsRng);
    let enc = encrypt(plain.as_ref(), key_bytes.into(), nonce)?;

    let key_bytes2 = derive_key_with_salt("my password", &salt)?;
    let dec = decrypt(&enc, key_bytes2.into())?;
    assert_eq!(dec, plain);
    Ok(())
}

#[test]
fn base64_roundtrip() {
    use base64::Engine;
    let data: Vec<u8> = (0u8..=255).collect();
    let encoded = data.to_base64();
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&encoded)
        .unwrap();
    assert_eq!(decoded, data);
}
