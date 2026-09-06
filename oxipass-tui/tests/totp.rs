use oxipass_tui::core::totp;

const RFC_SECRET_B32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

#[test]
fn bare_base32_matches_rfc6238_vectors() {
    let t = totp::parse(RFC_SECRET_B32).unwrap();
    assert_eq!(t.generate(59), "287082");
    assert_eq!(t.generate(1_111_111_109), "081804");
    assert_eq!(t.generate(1_234_567_890), "005924");
}

#[test]
fn otpauth_uri_parses_and_matches() {
    let uri = format!(
        "otpauth://totp/oxipass:alice?secret={RFC_SECRET_B32}&issuer=oxipass&period=30&digits=6"
    );
    let t = totp::parse(&uri).unwrap();
    assert_eq!(t.generate(59), "287082");
}

#[test]
fn invalid_values_are_rejected() {
    assert!(!totp::is_valid("not base32 !!!"));
    assert!(!totp::is_valid("otpauth://totp/x?issuer=y"));
    assert!(!totp::is_valid(""));
    assert!(!totp::is_valid("    "));
}

#[test]
fn short_secrets_are_accepted() {
    // Some providers still issue 80-bit (16-char) secrets; don't reject them.
    assert!(totp::is_valid("JBSWY3DPEHPK3PXP"));
}

#[test]
fn messy_bare_secrets_are_normalised() {
    let messy = "gezd gnbv gy3t qojq gezd gnbv gy3t qojq";
    let dashed = "GEZD-GNBV-GY3T-QOJQ-GEZD-GNBV-GY3T-QOJQ";
    assert_eq!(totp::parse(messy).unwrap().generate(59), "287082");
    assert_eq!(totp::parse(dashed).unwrap().generate(59), "287082");
}

#[test]
fn current_returns_code_and_ttl_in_range() {
    let (code, ttl) = totp::current(RFC_SECRET_B32).unwrap();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
    assert!((1..=30).contains(&ttl));
}
