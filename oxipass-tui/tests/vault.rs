use oxipass_tui::core::{Entry, Vault};
use uuid::Uuid;

fn login(name: &str, password: &str) -> Entry {
    Entry::Login {
        id: Uuid::new_v4(),
        name: name.to_string(),
        username: Some("user".to_string()),
        email: None,
        password: password.to_string(),
        url: None,
        notes: None,
    }
}

#[test]
fn save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");
    let password = "correct-horse-battery-staple";

    let mut vault = Vault::new();
    vault.push_entry(login("GitHub", "s3cr3t"));
    vault.push_entry(login("Email", "hunter2"));
    vault.save(&path, password, None).unwrap();

    let loaded = Vault::load(&path.with_extension("opdb"), password, None).unwrap();
    assert_eq!(loaded.entries().len(), 2);
    assert!(matches!(&loaded.entries()[0], Entry::Login { name, .. } if name == "GitHub"));
}

#[test]
fn wrong_password_fails() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("vault");

    Vault::new().save(&path, "correct", None).unwrap();
    assert!(Vault::load(&path.with_extension("opdb"), "wrong", None).is_err());
}

#[test]
fn remove_entry() {
    let mut vault = Vault::new();
    let entry = login("test", "pass");
    let id = match &entry {
        Entry::Login { id, .. } => *id,
        _ => unreachable!(),
    };
    vault.push_entry(entry);
    assert_eq!(vault.entries().len(), 1);
    assert!(vault.remove_entry(id));
    assert!(vault.entries().is_empty());
}

#[test]
fn replace_entry() {
    let mut vault = Vault::new();
    let entry = login("old", "oldpass");
    let id = match &entry {
        Entry::Login { id, .. } => *id,
        _ => unreachable!(),
    };
    vault.push_entry(entry);
    vault.replace_entry(id, login("new", "newpass"));
    assert!(matches!(&vault.entries()[0], Entry::Login { name, .. } if name == "new"));
}
