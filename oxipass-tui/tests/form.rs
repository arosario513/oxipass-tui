use oxipass_tui::tui::form::{EntryForm, EntryType, Field};

// --- Field ---

#[test]
fn push_and_pop() {
    let mut f = Field::new("x", false, false);
    f.push('a');
    f.push('b');
    f.push('c');
    assert_eq!(f.value, "abc");
    assert_eq!(f.cursor, 3);
    f.pop();
    assert_eq!(f.value, "ab");
    assert_eq!(f.cursor, 2);
}

#[test]
fn insert_at_cursor() {
    let mut f = Field::new("x", false, false);
    f.push('a');
    f.push('c');
    f.move_left();
    f.push('b');
    assert_eq!(f.value, "abc");
    assert_eq!(f.cursor, 2);
}

#[test]
fn delete_forward() {
    let mut f = Field::new("x", false, false);
    for c in "hello".chars() {
        f.push(c);
    }
    f.move_home();
    f.delete_forward();
    assert_eq!(f.value, "ello");
    assert_eq!(f.cursor, 0);
}

#[test]
fn cursor_bounds() {
    let mut f = Field::new("x", false, false);
    f.move_left();
    assert_eq!(f.cursor, 0);
    f.push('a');
    f.move_right();
    assert_eq!(f.cursor, 1);
}

#[test]
fn set_value_updates_cursor() {
    let mut f = Field::new("x", false, false);
    f.set_value("hello".to_string());
    assert_eq!(f.cursor, 5);
    assert_eq!(f.value, "hello");
}

#[test]
fn secret_display_masked() {
    let mut f = Field::new("x", true, false);
    for c in "pass".chars() {
        f.push(c);
    }
    assert_eq!(f.display(), "****");
}

// --- EntryForm validation ---

#[test]
fn empty_required_fields_invalid() {
    let mut form = EntryForm::new(EntryType::Login);
    assert!(!form.validate());
    assert!(form.fields[0].invalid); // Name
    assert!(form.fields[3].invalid); // Password
}

#[test]
fn invalid_email_format_fails() {
    let mut form = EntryForm::new(EntryType::Login);
    form.fields[0].value = "GitHub".to_string();
    form.fields[2].value = "notanemail".to_string();
    form.fields[3].value = "secret".to_string();
    assert!(!form.validate());
    assert!(form.fields[2].invalid);
}

#[test]
fn valid_email_passes() {
    let mut form = EntryForm::new(EntryType::Login);
    form.fields[0].value = "GitHub".to_string();
    form.fields[2].value = "user@example.com".to_string();
    form.fields[3].value = "secret".to_string();
    assert!(form.validate());
    assert!(!form.fields[2].invalid);
}

#[test]
fn optional_empty_field_passes() {
    let mut form = EntryForm::new(EntryType::Login);
    form.fields[0].value = "Site".to_string();
    form.fields[3].value = "pass".to_string();
    // username, email, url all empty — optional, should be fine
    assert!(form.validate());
}
