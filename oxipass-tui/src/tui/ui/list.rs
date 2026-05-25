use super::{ACCENT, PRIMARY, block};
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

pub fn render_preview(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_entries();
    let Some(entry) = filtered.get(app.selected) else {
        f.render_widget(block("Preview", Color::DarkGray), area);
        return;
    };

    let (title, title_color, fields): (&str, Color, Vec<(&str, String, bool)>) = match entry {
        Entry::Login {
            name,
            username,
            email,
            password,
            url,
            ..
        } => {
            let mut v = vec![("Name", name.clone(), false)];
            if let Some(u) = username {
                v.push(("Username", u.clone(), false));
            }
            if let Some(e) = email {
                v.push(("Email", e.clone(), false));
            }
            v.push(("Password", password.clone(), true));
            if let Some(u) = url {
                v.push(("URL", u.clone(), false));
            }
            (name.as_str(), Color::LightMagenta, v)
        }
        Entry::Payment {
            name,
            cardholder,
            card_number,
            exp_date,
            cvv,
            ..
        } => (
            name.as_str(),
            Color::LightYellow,
            vec![
                ("Name", name.clone(), false),
                ("Cardholder", cardholder.clone(), false),
                ("Card number", card_number.clone(), true),
                ("Expiry", exp_date.clone(), false),
                ("CVV", cvv.clone(), true),
            ],
        ),
        Entry::Note {
            name,
            description,
            content,
            ..
        } => {
            let mut v = vec![("Name", name.clone(), false)];
            if let Some(d) = description {
                v.push(("Description", d.clone(), false));
            }
            v.push(("Content", content.clone(), false));
            (name.as_str(), Color::LightGreen, v)
        }
    };

    f.render_widget(block(title, title_color), area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    let lines: Vec<Line> = fields
        .iter()
        .filter_map(|(label, value, secret)| {
            let display = if *secret && !app.reveal {
                "********".to_string()
            } else {
                value.clone()
            };
            if display.is_empty() {
                return None;
            }
            Some(Line::from(vec![
                Span::styled(
                    format!("{label}: "),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(display, Style::default().fg(Color::White)),
            ]))
        })
        .collect();

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
