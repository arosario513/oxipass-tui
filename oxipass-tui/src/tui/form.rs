use crate::core::{Entry, totp};

pub enum EntryType {
    Login,
    Payment,
    Note,
}

pub struct Field {
    pub label: &'static str,
    pub value: String,
    pub cursor: usize,
    pub secret: bool,
    pub optional: bool,
    pub invalid: bool,
    pub generatable: bool,
    pub multiline: bool,
    validator: Option<fn(&str) -> bool>,
}

impl Field {
    pub fn new(label: &'static str, secret: bool, optional: bool) -> Self {
        Self {
            label,
            value: String::new(),
            cursor: 0,
            secret,
            optional,
            invalid: false,
            generatable: false,
            multiline: false,
            validator: None,
        }
    }

    fn with_value(label: &'static str, secret: bool, optional: bool, value: String) -> Self {
        let cursor = value.chars().count();
        Self {
            label,
            value,
            cursor,
            secret,
            optional,
            invalid: false,
            generatable: false,
            multiline: false,
            validator: None,
        }
    }

    fn generatable(mut self) -> Self {
        self.generatable = true;
        self
    }

    fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    fn with_validator(mut self, validator: fn(&str) -> bool) -> Self {
        self.validator = Some(validator);
        self
    }

    /// Returns `(row, col)` of the cursor within the field value.
    /// For single-line fields this is always `(0, cursor)`.
    pub fn cursor_pos(&self) -> (u16, u16) {
        let byte_idx = char_to_byte(&self.value, self.cursor);
        let before = &self.value[..byte_idx];
        let row = before.chars().filter(|&c| c == '\n').count() as u16;
        let col = match before.rfind('\n') {
            Some(i) => before[i + 1..].chars().count() as u16,
            None => before.chars().count() as u16,
        };
        (row, col)
    }

    pub fn push(&mut self, c: char) {
        let byte_idx = char_to_byte(&self.value, self.cursor);
        self.value.insert(byte_idx, c);
        self.cursor += 1;
        self.invalid = false;
    }

    pub fn pop(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let byte_idx = char_to_byte(&self.value, self.cursor - 1);
        self.value.remove(byte_idx);
        self.cursor -= 1;
    }

    pub fn delete_forward(&mut self) {
        let len = self.value.chars().count();
        if self.cursor >= len {
            return;
        }
        let byte_idx = char_to_byte(&self.value, self.cursor);
        self.value.remove(byte_idx);
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.value.chars().count() {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.value.chars().count();
    }

    pub fn set_value(&mut self, value: String) {
        self.cursor = value.chars().count();
        self.value = value;
        self.invalid = false;
    }

    pub fn display(&self) -> String {
        if self.secret {
            "*".repeat(self.value.len())
        } else {
            self.value.clone()
        }
    }
}

pub struct EntryForm {
    pub entry_type: EntryType,
    pub fields: Vec<Field>,
    pub focused: usize,
    pub is_edit: bool,
}

impl EntryForm {
    pub fn new(entry_type: EntryType) -> Self {
        let fields = match entry_type {
            EntryType::Login => vec![
                Field::new("Name", false, false),
                Field::new("Username", false, true),
                Field::new("Email", false, true).with_validator(is_valid_email),
                Field::new("Password", true, false).generatable(),
                Field::new("TOTP", false, true).with_validator(totp::is_valid),
                Field::new("URL", false, true),
                Field::new("Notes", false, true).multiline(),
            ],
            EntryType::Payment => vec![
                Field::new("Name", false, false),
                Field::new("Cardholder", false, false),
                Field::new("Card number", false, false),
                Field::new("Expiry date (MM/YY)", false, false),
                Field::new("CVV", true, false),
                Field::new("Notes", false, true).multiline(),
            ],
            EntryType::Note => vec![
                Field::new("Name", false, false),
                Field::new("Description", false, true),
                Field::new("Content", false, false).multiline(),
            ],
        };
        Self {
            entry_type,
            fields,
            focused: 0,
            is_edit: false,
        }
    }

    pub fn from_entry(entry: &Entry) -> Self {
        let (entry_type, fields) = match entry {
            Entry::Login {
                name,
                username,
                email,
                password,
                url,
                notes,
                totp,
                ..
            } => (
                EntryType::Login,
                vec![
                    Field::with_value("Name", false, false, name.clone()),
                    Field::with_value(
                        "Username",
                        false,
                        true,
                        username.as_deref().unwrap_or("").to_string(),
                    ),
                    Field::with_value(
                        "Email",
                        false,
                        true,
                        email.as_deref().unwrap_or("").to_string(),
                    )
                    .with_validator(is_valid_email),
                    Field::with_value("Password", true, false, password.clone()).generatable(),
                    Field::with_value(
                        "TOTP",
                        false,
                        true,
                        totp.as_deref().unwrap_or("").to_string(),
                    )
                    .with_validator(totp::is_valid),
                    Field::with_value("URL", false, true, url.as_deref().unwrap_or("").to_string()),
                    Field::with_value(
                        "Notes",
                        false,
                        true,
                        notes.as_deref().unwrap_or("").to_string(),
                    )
                    .multiline(),
                ],
            ),
            Entry::Payment {
                name,
                cardholder,
                card_number,
                exp_date,
                cvv,
                notes,
                ..
            } => (
                EntryType::Payment,
                vec![
                    Field::with_value("Name", false, false, name.clone()),
                    Field::with_value("Cardholder", false, false, cardholder.clone()),
                    Field::with_value("Card number", false, false, card_number.clone()),
                    Field::with_value("Expiry date (MM/YY)", false, false, exp_date.clone()),
                    Field::with_value("CVV", true, false, cvv.clone()),
                    Field::with_value(
                        "Notes",
                        false,
                        true,
                        notes.as_deref().unwrap_or("").to_string(),
                    )
                    .multiline(),
                ],
            ),
            Entry::Note {
                name,
                description,
                content,
                ..
            } => (
                EntryType::Note,
                vec![
                    Field::with_value("Name", false, false, name.clone()),
                    Field::with_value(
                        "Description",
                        false,
                        true,
                        description.as_deref().unwrap_or("").to_string(),
                    ),
                    Field::with_value("Content", false, false, content.clone()).multiline(),
                ],
            ),
        };
        Self {
            entry_type,
            fields,
            focused: 0,
            is_edit: true,
        }
    }

    /// Marks empty required fields and fields failing their validator as invalid.
    /// Returns true if all fields pass.
    pub fn validate(&mut self) -> bool {
        let mut valid = true;
        for field in &mut self.fields {
            let empty = field.value.trim().is_empty();
            let format_ok = !empty && field.validator.is_none_or(|f| f(field.value.trim()));
            if !field.optional && empty {
                field.invalid = true;
                valid = false;
            } else {
                match !empty && !format_ok {
                    true => {
                        field.invalid = true;
                        valid = false;
                    }
                    false => {
                        field.invalid = false;
                    }
                }
            }
        }
        valid
    }

    pub fn next_field(&mut self) {
        if self.focused < self.fields.len() - 1 {
            self.focused += 1;
        }
    }

    pub fn prev_field(&mut self) {
        if self.focused > 0 {
            self.focused -= 1;
        }
    }

    pub fn title(&self) -> &'static str {
        match (&self.entry_type, self.is_edit) {
            (EntryType::Login, false) => "Add Login",
            (EntryType::Login, true) => "Edit Login",
            (EntryType::Payment, false) => "Add Payment",
            (EntryType::Payment, true) => "Edit Payment",
            (EntryType::Note, false) => "Add Note",
            (EntryType::Note, true) => "Edit Note",
        }
    }
}

fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(s.len())
}

fn is_valid_email(s: &str) -> bool {
    match s.find('@') {
        None | Some(0) => false,
        Some(at) => {
            let domain = &s[at + 1..];
            !domain.is_empty()
                && !domain.starts_with('.')
                && !domain.ends_with('.')
                && domain.contains('.')
        }
    }
}
