pub mod form;
mod ui;

use crate::core::PasswordGen;
use crate::core::{Entry, Vault, VaultError};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use form::{EntryForm, EntryType};
use ratatui::{Terminal, backend::CrosstermBackend};
#[cfg(not(windows))]
use std::io::Write;
use std::io::{self};
use std::path::PathBuf;
use uuid::Uuid;

pub enum Mode {
    Normal,
    Searching,
    PendingAdd,
    Adding(EntryForm),
    Editing(EntryForm, Uuid),
    ConfirmDelete,
    CopyPicker(Vec<(String, String)>), // (label, value)
}

pub struct App {
    pub vault: Vault,
    pub path: PathBuf,
    pub password: String,
    pub keyfile: Option<Vec<u8>>,
    pub selected: usize,
    pub mode: Mode,
    pub status_msg: Option<&'static str>,
    pub search: String,
    pub reveal: bool,
    pub generator: Option<(PasswordGen, bool)>, // (gen, standalone)
}

impl App {
    fn new(vault: Vault, path: PathBuf, password: String, keyfile: Option<Vec<u8>>) -> Self {
        Self {
            vault,
            path,
            password,
            keyfile,
            selected: 0,
            mode: Mode::Normal,
            status_msg: None,
            search: String::new(),
            reveal: false,
            generator: None,
        }
    }

    pub fn filtered_entries(&self) -> Vec<&Entry> {
        if self.search.is_empty() {
            self.vault.entries().iter().collect()
        } else {
            let q = self.search.to_lowercase();
            self.vault
                .entries()
                .iter()
                .filter(|e| matches_search(e, &q))
                .collect()
        }
    }

    fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected + 1 < self.filtered_entries().len() {
            self.selected += 1;
        }
    }

    fn delete_selected(&mut self) {
        let id = {
            let filtered = self.filtered_entries();
            match filtered.get(self.selected) {
                Some(Entry::Login { id, .. }) => *id,
                Some(Entry::Payment { id, .. }) => *id,
                Some(Entry::Note { id, .. }) => *id,
                None => return,
            }
        };
        self.vault.remove_entry(id);
        if self.selected > 0 && self.selected >= self.filtered_entries().len() {
            self.selected -= 1;
        }
        let _ = self
            .vault
            .save(&self.path, &self.password, self.keyfile.as_deref());
    }

    fn submit_form(&mut self) {
        let Mode::Adding(form) = &mut self.mode else {
            return;
        };
        if !form.validate() {
            return;
        }
        let entry = build_entry(form);
        self.mode = Mode::Normal;
        self.vault.push_entry(entry);
        let _ = self
            .vault
            .save(&self.path, &self.password, self.keyfile.as_deref());
    }

    fn submit_edit(&mut self) {
        let Mode::Editing(form, id) = &mut self.mode else {
            return;
        };
        if !form.validate() {
            return;
        }
        let id = *id;
        let mut entry = build_entry(form);
        set_entry_id(&mut entry, id);
        self.mode = Mode::Normal;
        self.vault.replace_entry(id, entry);
        let _ = self
            .vault
            .save(&self.path, &self.password, self.keyfile.as_deref());
    }
}

fn matches_search(entry: &Entry, query: &str) -> bool {
    match entry {
        Entry::Login {
            name,
            username,
            email,
            url,
            notes,
            ..
        } => {
            name.to_lowercase().contains(query)
                || username
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(query)
                || email
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(query)
                || url.as_deref().unwrap_or("").to_lowercase().contains(query)
                || notes
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(query)
        }
        Entry::Payment {
            name,
            cardholder,
            notes,
            ..
        } => {
            name.to_lowercase().contains(query)
                || cardholder.to_lowercase().contains(query)
                || notes
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(query)
        }
        Entry::Note {
            name,
            description,
            content,
            ..
        } => {
            name.to_lowercase().contains(query)
                || description
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(query)
                || content.to_lowercase().contains(query)
        }
    }
}

fn non_empty(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

fn build_entry(form: &form::EntryForm) -> Entry {
    match form.entry_type {
        form::EntryType::Login => Entry::Login {
            id: Uuid::new_v4(),
            name: form.fields[0].value.clone(),
            username: non_empty(form.fields[1].value.clone()),
            email: non_empty(form.fields[2].value.clone()),
            password: form.fields[3].value.clone(),
            url: non_empty(form.fields[4].value.clone()),
            notes: non_empty(form.fields[5].value.clone()),
        },
        form::EntryType::Payment => Entry::Payment {
            id: Uuid::new_v4(),
            name: form.fields[0].value.clone(),
            cardholder: form.fields[1].value.clone(),
            card_number: form.fields[2].value.clone(),
            exp_date: form.fields[3].value.clone(),
            cvv: form.fields[4].value.clone(),
            notes: non_empty(form.fields[5].value.clone()),
        },
        form::EntryType::Note => Entry::Note {
            id: Uuid::new_v4(),
            name: form.fields[0].value.clone(),
            description: non_empty(form.fields[1].value.clone()),
            content: form.fields[2].value.clone(),
        },
    }
}

fn set_entry_id(entry: &mut Entry, id: Uuid) {
    match entry {
        Entry::Login { id: eid, .. } => *eid = id,
        Entry::Payment { id: eid, .. } => *eid = id,
        Entry::Note { id: eid, .. } => *eid = id,
    }
}

fn copy_fields(entry: &Entry) -> Vec<(String, String)> {
    let mut fields = Vec::new();
    let mut push = |label: &str, value: &str| {
        if !value.is_empty() {
            fields.push((label.to_string(), value.to_string()));
        }
    };
    match entry {
        Entry::Login {
            username,
            email,
            password,
            url,
            notes,
            ..
        } => {
            push("Username", username.as_deref().unwrap_or(""));
            push("Email", email.as_deref().unwrap_or(""));
            push("Password", password);
            push("URL", url.as_deref().unwrap_or(""));
            push("Notes", notes.as_deref().unwrap_or(""));
        }
        Entry::Payment {
            cardholder,
            card_number,
            exp_date,
            cvv,
            notes,
            ..
        } => {
            push("Cardholder", cardholder);
            push("Card number", card_number);
            push("Expiry", exp_date);
            push("CVV", cvv);
            push("Notes", notes.as_deref().unwrap_or(""));
        }
        Entry::Note {
            description,
            content,
            ..
        } => {
            push("Content", content);
            push("Description", description.as_deref().unwrap_or(""));
        }
    }
    fields
}

fn copy_to_clipboard(text: &str) -> Result<(), io::Error> {
    #[cfg(windows)]
    {
        clipboard_win::set_clipboard_string(text)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        return Ok(());
    }
    #[cfg(not(windows))]
    {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
        io::stdout().write_all(format!("\x1b]52;c;{}\x07", encoded).as_bytes())?;
        io::stdout().flush()?;
        Ok(())
    }
}

pub fn run(
    vault: Vault,
    path: PathBuf,
    password: String,
    keyfile: Option<Vec<u8>>,
) -> Result<(), VaultError> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(vault, path, password, keyfile);
    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<(), VaultError> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        let Event::Key(key) = event::read()? else {
            continue;
        };

        app.status_msg = None;

        // Global: Ctrl+C always quits
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            break;
        }

        // Generator overlay intercepts all keys when active
        if app.generator.is_some() {
            match key.code {
                KeyCode::Esc => {
                    app.generator = None;
                }
                KeyCode::Char('c') => {
                    if let Some((g, _)) = app.generator.take() {
                        copy_to_clipboard(&g.password)?;
                        app.status_msg = Some("Copied to clipboard!");
                    }
                }
                KeyCode::Enter => {
                    if let Some((g, standalone)) = app.generator.take()
                        && !standalone
                    {
                        let pwd = g.password;
                        match &mut app.mode {
                            Mode::Adding(form) | Mode::Editing(form, _) => {
                                form.fields[form.focused].set_value(pwd);
                            }
                            _ => {}
                        }
                    }
                }
                KeyCode::Char('r') | KeyCode::Char(' ') => {
                    if let Some((g, _)) = &mut app.generator {
                        g.regenerate();
                    }
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    if let Some((g, _)) = &mut app.generator {
                        g.decrease_length();
                    }
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    if let Some((g, _)) = &mut app.generator {
                        g.increase_length();
                    }
                }
                KeyCode::Char('u') => {
                    if let Some((g, _)) = &mut app.generator {
                        let next = !g.use_upper;
                        if next || g.use_lower || g.use_digits || g.use_symbols {
                            g.use_upper = next;
                            g.regenerate();
                        }
                    }
                }
                KeyCode::Char('l') => {
                    if let Some((g, _)) = &mut app.generator {
                        let next = !g.use_lower;
                        if next || g.use_upper || g.use_digits || g.use_symbols {
                            g.use_lower = next;
                            g.regenerate();
                        }
                    }
                }
                KeyCode::Char('d') => {
                    if let Some((g, _)) = &mut app.generator {
                        let next = !g.use_digits;
                        if next || g.use_upper || g.use_lower || g.use_symbols {
                            g.use_digits = next;
                            g.regenerate();
                        }
                    }
                }
                KeyCode::Char('s') => {
                    if let Some((g, _)) = &mut app.generator {
                        let next = !g.use_symbols;
                        if next || g.use_upper || g.use_lower || g.use_digits {
                            g.use_symbols = next;
                            g.regenerate();
                        }
                    }
                }
                _ => {}
            }
            continue;
        }

        match &app.mode {
            Mode::Normal => match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('k') | KeyCode::Up => app.move_up(),
                KeyCode::Char('j') | KeyCode::Down => app.move_down(),
                KeyCode::Char('/') => app.mode = Mode::Searching,
                KeyCode::Esc if !app.search.is_empty() => {
                    app.search.clear();
                    app.selected = 0;
                }
                KeyCode::Char('r') if !app.filtered_entries().is_empty() => {
                    app.reveal = !app.reveal;
                }
                KeyCode::Char('c') if !app.filtered_entries().is_empty() => {
                    let fields = {
                        let filtered = app.filtered_entries();
                        filtered.get(app.selected).map(|e| copy_fields(e))
                    };
                    if let Some(fields) = fields {
                        if fields.len() == 1 {
                            copy_to_clipboard(&fields[0].1)?;
                            app.status_msg = Some("Copied to clipboard!");
                        } else {
                            app.mode = Mode::CopyPicker(fields);
                        }
                    }
                }
                KeyCode::Char('a') => app.mode = Mode::PendingAdd,
                KeyCode::Char('g') => app.generator = Some((PasswordGen::new(), true)),
                KeyCode::Char('e') if !app.filtered_entries().is_empty() => {
                    let result = {
                        let filtered = app.filtered_entries();
                        filtered.get(app.selected).map(|entry| {
                            let id = match entry {
                                Entry::Login { id, .. } => *id,
                                Entry::Payment { id, .. } => *id,
                                Entry::Note { id, .. } => *id,
                            };
                            (id, EntryForm::from_entry(entry))
                        })
                    };
                    if let Some((id, form)) = result {
                        app.mode = Mode::Editing(form, id);
                    }
                }
                KeyCode::Char('d') if !app.filtered_entries().is_empty() => {
                    app.mode = Mode::ConfirmDelete
                }
                _ => {}
            },
            Mode::Searching => match key.code {
                KeyCode::Esc => {
                    app.search.clear();
                    app.selected = 0;
                    app.mode = Mode::Normal;
                }
                KeyCode::Enter => {
                    app.selected = 0;
                    app.mode = Mode::Normal;
                }
                KeyCode::Backspace => {
                    app.search.pop();
                    app.selected = 0;
                }
                KeyCode::Char(c) => {
                    app.search.push(c);
                    app.selected = 0;
                }
                _ => {}
            },
            Mode::PendingAdd => match key.code {
                KeyCode::Char('l') => app.mode = Mode::Adding(EntryForm::new(EntryType::Login)),
                KeyCode::Char('p') => app.mode = Mode::Adding(EntryForm::new(EntryType::Payment)),
                KeyCode::Char('n') => app.mode = Mode::Adding(EntryForm::new(EntryType::Note)),
                KeyCode::Esc => app.mode = Mode::Normal,
                _ => {}
            },
            Mode::Adding(_) | Mode::Editing(_, _) => match key.code {
                KeyCode::Esc => app.mode = Mode::Normal,
                KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => match app.mode {
                    Mode::Adding(_) => app.submit_form(),
                    Mode::Editing(_, _) => app.submit_edit(),
                    _ => {}
                },
                KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let is_generatable = match &app.mode {
                        Mode::Adding(form) | Mode::Editing(form, _) => {
                            form.fields[form.focused].generatable
                        }
                        _ => false,
                    };
                    if is_generatable {
                        app.generator = Some((PasswordGen::new(), false));
                    }
                }
                KeyCode::Tab | KeyCode::Down => match &mut app.mode {
                    Mode::Adding(form) | Mode::Editing(form, _) => form.next_field(),
                    _ => {}
                },
                KeyCode::BackTab | KeyCode::Up => match &mut app.mode {
                    Mode::Adding(form) | Mode::Editing(form, _) => form.prev_field(),
                    _ => {}
                },
                KeyCode::Enter => {
                    let is_multiline = match &app.mode {
                        Mode::Adding(f) | Mode::Editing(f, _) => f.fields[f.focused].multiline,
                        _ => false,
                    };
                    if is_multiline {
                        match &mut app.mode {
                            Mode::Adding(form) | Mode::Editing(form, _) => {
                                form.fields[form.focused].push('\n')
                            }
                            _ => {}
                        }
                    } else {
                        let on_last = match &app.mode {
                            Mode::Adding(f) | Mode::Editing(f, _) => {
                                f.focused == f.fields.len() - 1
                            }
                            _ => false,
                        };
                        if on_last {
                            match app.mode {
                                Mode::Adding(_) => app.submit_form(),
                                Mode::Editing(_, _) => app.submit_edit(),
                                _ => {}
                            }
                        } else {
                            match &mut app.mode {
                                Mode::Adding(form) | Mode::Editing(form, _) => form.next_field(),
                                _ => {}
                            }
                        }
                    }
                }
                KeyCode::Backspace => match &mut app.mode {
                    Mode::Adding(form) | Mode::Editing(form, _) => form.fields[form.focused].pop(),
                    _ => {}
                },
                KeyCode::Delete => match &mut app.mode {
                    Mode::Adding(form) | Mode::Editing(form, _) => {
                        form.fields[form.focused].delete_forward()
                    }
                    _ => {}
                },
                KeyCode::Left => match &mut app.mode {
                    Mode::Adding(form) | Mode::Editing(form, _) => {
                        form.fields[form.focused].move_left()
                    }
                    _ => {}
                },
                KeyCode::Right => match &mut app.mode {
                    Mode::Adding(form) | Mode::Editing(form, _) => {
                        form.fields[form.focused].move_right()
                    }
                    _ => {}
                },
                KeyCode::Home => match &mut app.mode {
                    Mode::Adding(form) | Mode::Editing(form, _) => {
                        form.fields[form.focused].move_home()
                    }
                    _ => {}
                },
                KeyCode::End => match &mut app.mode {
                    Mode::Adding(form) | Mode::Editing(form, _) => {
                        form.fields[form.focused].move_end()
                    }
                    _ => {}
                },
                KeyCode::Char(c) => match &mut app.mode {
                    Mode::Adding(form) | Mode::Editing(form, _) => {
                        form.fields[form.focused].push(c)
                    }
                    _ => {}
                },
                _ => {}
            },
            Mode::ConfirmDelete => match key.code {
                KeyCode::Char('y') => {
                    app.delete_selected();
                    app.mode = Mode::Normal;
                }
                KeyCode::Char('n') | KeyCode::Esc => app.mode = Mode::Normal,
                _ => {}
            },
            Mode::CopyPicker(_) => match key.code {
                KeyCode::Esc => app.mode = Mode::Normal,
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    let idx = c.to_digit(10).unwrap() as usize;
                    let value = match &app.mode {
                        Mode::CopyPicker(fields) => {
                            fields.get(idx.saturating_sub(1)).map(|(_, v)| v.clone())
                        }
                        _ => None,
                    };
                    app.mode = Mode::Normal;
                    if let Some(text) = value {
                        copy_to_clipboard(&text)?;
                        app.status_msg = Some("Copied to clipboard!");
                    }
                }
                _ => {}
            },
        }
    }
    Ok(())
}
