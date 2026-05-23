use oxipass_tui::core::PasswordGen;

#[test]
fn charset_size_all_enabled() {
    let g = PasswordGen::new();
    // lower(26) + upper(26) + digits(10) + symbols(26)
    assert_eq!(g.charset_size(), 88);
}

#[test]
fn charset_size_only_digits() {
    let mut g = PasswordGen::new();
    g.use_upper = false;
    g.use_lower = false;
    g.use_symbols = false;
    assert_eq!(g.charset_size(), 10);
}

#[test]
fn password_length_matches() {
    let mut g = PasswordGen::new();
    g.length = 24;
    g.regenerate();
    assert_eq!(g.password.chars().count(), 24);
}

#[test]
fn entropy_positive() {
    let g = PasswordGen::new();
    assert!(g.entropy_bits() > 0.0);
}

#[test]
fn entropy_formula() {
    let g = PasswordGen::new();
    let expected = g.length as f64 * (g.charset_size() as f64).log2();
    assert!((g.entropy_bits() - expected).abs() < f64::EPSILON);
}

#[test]
fn length_floor() {
    let mut g = PasswordGen::new();
    g.length = 4;
    g.decrease_length();
    assert_eq!(g.length, 4);
}

#[test]
fn length_ceiling() {
    let mut g = PasswordGen::new();
    g.length = 64;
    g.increase_length();
    assert_eq!(g.length, 64);
}

#[test]
fn strength_label_weak() {
    let mut g = PasswordGen::new();
    g.password = "a".to_string();
    assert_eq!(g.strength_label(), "Weak");
}

#[test]
fn strength_label_very_strong() {
    let mut g = PasswordGen::new();
    g.length = 32;
    g.regenerate();
    // a 32-char random password with full charset should always score 4
    assert_eq!(g.strength_label(), "Very Strong");
}
