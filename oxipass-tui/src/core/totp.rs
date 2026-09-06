use crate::core::VaultError;
use totp_rs::{Algorithm, Secret, TOTP};

const OTPAUTH_PREFIX: &str = "otpauth://";

pub fn parse(input: &str) -> Result<TOTP, VaultError> {
    let input = input.trim();
    if input.starts_with(OTPAUTH_PREFIX) {
        return TOTP::from_url(input).map_err(|_| VaultError::InvalidTotp);
    }

    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '=')
        .collect::<String>()
        .to_uppercase();

    let secret = Secret::Encoded(cleaned)
        .to_bytes()
        .map_err(|_| VaultError::InvalidTotp)?;
    if secret.is_empty() {
        return Err(VaultError::InvalidTotp);
    }

    Ok(TOTP::new_unchecked(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret,
        None,
        String::new(),
    ))
}

pub fn is_valid(input: &str) -> bool {
    parse(input).is_ok()
}

pub fn current(input: &str) -> Option<(String, u64)> {
    let totp = parse(input).ok()?;
    let code = totp.generate_current().ok()?;
    let ttl = totp.ttl().ok()?;
    Some((code, ttl))
}
