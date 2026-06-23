use super::{ACCENT, LABEL, PRIMARY, block};
use crate::core::Entry;
use crate::tui::App;
use ratatui::{
    Frame,
    layout::Alignment,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState, Paragraph},
};
use zxcvbn::zxcvbn;

pub fn render_list(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_entries();

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|entry| {
            let line = match entry {
                Entry::Login {
                    name,
                    username,
                    email,
                    ..
                } => {
                    let identity = username
                        .as_deref()
                        .filter(|s| !s.is_empty())
                        .or(email.as_deref())
                        .unwrap_or("");
                    Line::from(vec![
                        Span::styled(
                            "[Login]   ",
                            Style::default()
                                .fg(Color::LightMagenta)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::raw(format!("{}: {}", name, identity)),
                    ])
                }
                Entry::Payment {
                    name, cardholder, ..
                } => Line::from(vec![
                    Span::styled(
                        "[Payment] ",
                        Style::default()
                            .fg(Color::LightYellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!("{}: {}", name, cardholder)),
                ]),
                Entry::Note { name, .. } => Line::from(vec![
                    Span::styled(
                        "[Note]    ",
                        Style::default()
                            .fg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(name.clone()),
                ]),
            };
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    if !items.is_empty() {
        state.select(Some(app.selected));
    }

    let title = if app.search.is_empty() {
        "Entries".to_string()
    } else {
        format!("Entries [{}]", app.search)
    };

    let list = List::new(items)
        .block(block(&title, PRIMARY))
        .highlight_style(Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");

    f.render_stateful_widget(list, area, &mut state);
}

struct PreviewData<'a> {
    title: &'a str,
    title_color: Color,
    fields: Vec<(&'static str, String, bool)>,
    password: Option<&'a str>,
}

pub fn render_preview(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_entries();
    let Some(entry) = filtered.get(app.selected) else {
        f.render_widget(block("Preview", Color::DarkGray), area);
        return;
    };

    let preview = match entry {
        Entry::Login {
            name,
            username,
            email,
            password,
            url,
            notes,
            ..
        } => {
            let mut fields = vec![("Name", name.clone(), false)];
            if let Some(u) = username {
                fields.push(("Username", u.clone(), false));
            }
            if let Some(e) = email {
                fields.push(("Email", e.clone(), false));
            }
            fields.push(("Password", password.clone(), true));
            if let Some(u) = url {
                fields.push(("URL", u.clone(), false));
            }
            if let Some(n) = notes {
                fields.push(("Notes", n.clone(), false));
            }
            PreviewData {
                title: name.as_str(),
                title_color: Color::LightMagenta,
                fields,
                password: Some(password.as_str()),
            }
        }
        Entry::Payment {
            name,
            cardholder,
            card_number,
            exp_date,
            cvv,
            notes,
            ..
        } => {
            let mut fields = vec![
                ("Name", name.clone(), false),
                ("Cardholder", cardholder.clone(), false),
                ("Card number", card_number.clone(), true),
                ("Expiry", exp_date.clone(), false),
                ("CVV", cvv.clone(), true),
            ];
            if let Some(n) = notes {
                fields.push(("Notes", n.clone(), false));
            }
            PreviewData {
                title: name.as_str(),
                title_color: Color::LightYellow,
                fields,
                password: None,
            }
        }
        Entry::Note {
            name,
            description,
            content,
            ..
        } => {
            let mut fields = vec![("Name", name.clone(), false)];
            if let Some(d) = description {
                fields.push(("Description", d.clone(), false));
            }
            fields.push(("Content", content.clone(), false));
            PreviewData {
                title: name.as_str(),
                title_color: Color::LightGreen,
                fields,
                password: None,
            }
        }
    };

    f.render_widget(block(preview.title, preview.title_color), area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    let has_pw_strength = preview.password.is_some_and(|p| !p.is_empty());
    let max_label_width = preview
        .fields
        .iter()
        .filter(|(_, value, secret)| {
            let display = if *secret && !app.reveal {
                "********"
            } else {
                value.as_str()
            };
            !display.is_empty()
        })
        .map(|(label, _, _)| label.len())
        .chain(has_pw_strength.then_some("Password Strength".len()))
        .max()
        .unwrap_or(0);

    let mut lines: Vec<Line> = preview
        .fields
        .iter()
        .flat_map(|(label, value, secret)| {
            let display = if *secret && !app.reveal {
                "********".to_string()
            } else {
                value.clone()
            };
            if display.is_empty() {
                return vec![];
            }
            let label_style = Style::default().fg(LABEL).add_modifier(Modifier::BOLD);
            let value_style = Style::default();
            let indent = " ".repeat(max_label_width + 2);
            let mut field_lines: Vec<Line> = Vec::new();
            let mut parts = display.splitn(2, '\n');
            let first = parts.next().unwrap_or("");
            field_lines.push(Line::from(vec![
                Span::styled(format!("{label:<max_label_width$}: "), label_style),
                Span::styled(first.to_string(), value_style),
            ]));
            if let Some(rest) = parts.next() {
                for part in rest.split('\n') {
                    field_lines.push(Line::from(Span::styled(
                        format!("{indent}{part}"),
                        value_style,
                    )));
                }
            }
            field_lines
        })
        .collect();

    if has_pw_strength {
        let pw = preview.password.unwrap_or("");
        let score = u8::from(zxcvbn(pw, &[]).score());
        let (strength_label, color) = match score {
            0 | 1 => ("Weak", Color::Red),
            2 => ("Moderate", Color::Yellow),
            3 => ("Strong", Color::Green),
            _ => ("Very Strong", Color::Cyan),
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<max_label_width$}: ", "Password Strength"),
                Style::default().fg(LABEL).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                strength_label,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    f.render_widget(Paragraph::new(lines), inner);
}

pub fn render_stats(f: &mut Frame, app: &App, area: Rect) {
    let entries = app.vault.entries();
    let logins = entries
        .iter()
        .filter(|e| matches!(e, Entry::Login { .. }))
        .count();
    let payments = entries
        .iter()
        .filter(|e| matches!(e, Entry::Payment { .. }))
        .count();
    let notes = entries
        .iter()
        .filter(|e| matches!(e, Entry::Note { .. }))
        .count();

    let line = Line::from(vec![
        Span::styled("Total ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            entries.len().to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "[Login] ",
            Style::default()
                .fg(Color::LightMagenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            logins.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "[Payment] ",
            Style::default()
                .fg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            payments.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("   "),
        Span::styled(
            "[Note] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            notes.to_string(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    f.render_widget(
        Paragraph::new(line)
            .block(block("Vault", Color::DarkGray))
            .alignment(Alignment::Center),
        area,
    );
}
