mod generator;
mod list;
mod overlays;

use crate::tui::{App, Mode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub(super) const PRIMARY: Color = Color::Blue;
pub(super) const ACCENT: Color = Color::Cyan;
pub(super) const DANGER: Color = Color::Yellow;
const LOGO_COLOR: Color = Color::White;

const LOGO: &str = concat!(
    "________         .__\n",
    "\\_____  \\ ___  __|__|__________    ______ ______\n",
    " /   |   \\\\  \\/  /  \\____ \\__  \\  /  ___//  ___/\n",
    "/    |    \\>    <|  |  |_> > __ \\_\\___ \\ \\___ \\\n",
    "\\_______  /__/\\_ \\__|   __(____  /____  >____  >\n",
    "        \\/      \\/  |__|       \\/     \\/     \\/\n",
    "             Secure. Private. Yours."
);

pub(super) fn block(title: &str, border_color: Color) -> Block<'_> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            title,
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ))
}

pub(super) fn keybinds<'a>(pairs: &[(&'a str, &'a str)]) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    for (i, (key, action)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  │  ", Style::default().fg(Color::DarkGray)));
        }
        spans.push(Span::styled(
            *key,
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(format!(" {}", action)));
    }
    spans
}

pub(super) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(layout[1])[1]
}

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(5)])
        .split(main[1]);

    render_title(f, app, chunks[0]);
    list::render_list(f, app, main[0]);
    list::render_preview(f, app, right[0]);
    list::render_stats(f, app, right[1]);
    render_statusbar(f, app, chunks[2]);

    match &app.mode {
        Mode::PendingAdd => overlays::render_pending_add(f),
        Mode::Adding(form) | Mode::Editing(form, _) => overlays::render_form(f, form),
        Mode::ConfirmDelete => overlays::render_confirm(f),
        Mode::Normal | Mode::Searching => {}
    }

    if let Some((g, standalone)) = &app.generator {
        generator::render_generator(f, g, *standalone);
    }
}

fn render_title(f: &mut Frame, app: &App, area: Rect) {
    let path = app.path.display().to_string();
    let title = Paragraph::new(LOGO)
        .block(block(&path, PRIMARY))
        .style(Style::default().fg(LOGO_COLOR).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Left);
    f.render_widget(title, area);
}

fn render_statusbar(f: &mut Frame, app: &App, area: Rect) {
    use crate::core::Entry;

    if let Mode::Searching = &app.mode {
        let content = format!("/ {}", app.search);
        let bar = Paragraph::new(Span::styled(
            content.clone(),
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .block(block("Search", PRIMARY))
        .alignment(Alignment::Left);
        f.render_widget(bar, area);
        f.set_cursor_position((area.x + 1 + content.len() as u16, area.y + 1));
        return;
    }

    if let Some(msg) = app.status_msg {
        let bar = Paragraph::new(Span::styled(
            msg,
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        ))
        .block(block("", PRIMARY))
        .alignment(Alignment::Center);
        f.render_widget(bar, area);
        return;
    }

    let spans: Vec<Span> = match &app.mode {
        Mode::Normal => {
            let has_secret = matches!(
                app.filtered_entries().get(app.selected),
                Some(Entry::Login { .. }) | Some(Entry::Payment { .. })
            );
            let mut pairs: Vec<(&str, &str)> = vec![("j/k", "Navigate")];
            if has_secret {
                pairs.push(("r", if app.reveal { "Hide" } else { "Reveal" }));
            }
            pairs.extend([
                ("c", "Copy"),
                ("/", "Search"),
                ("a", "Add"),
                ("e", "Edit"),
                ("d", "Delete"),
                ("g", "Generator"),
                ("q", "Quit"),
            ]);
            if !app.search.is_empty() {
                pairs.push(("Esc", "Clear search"));
            }
            keybinds(&pairs)
        }
        Mode::PendingAdd => keybinds(&[
            ("l", "Login"),
            ("p", "Payment"),
            ("n", "Note"),
            ("Esc", "Cancel"),
        ]),
        Mode::Adding(form) | Mode::Editing(form, _) => {
            let on_multiline = form.fields[form.focused].multiline;
            let mut pairs: Vec<(&str, &str)> = vec![("Tab/↓", "Next"), ("Shift+Tab/↑", "Prev")];
            if on_multiline {
                pairs.push(("Enter", "Newline"));
                pairs.push(("Alt+Enter", "Confirm"));
            } else {
                pairs.push(("Enter", "Confirm"));
            }
            pairs.push(("Esc", "Cancel"));
            if form.fields[form.focused].generatable {
                pairs.push(("Ctrl+G", "Generate"));
            }
            keybinds(&pairs)
        }
        Mode::ConfirmDelete => vec![
            Span::styled(
                "Delete selected entry?  ",
                Style::default().fg(DANGER).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "y ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Confirm  "),
            Span::styled(
                "n / Esc ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw("Cancel"),
        ],
        Mode::Searching => unreachable!(),
    };

    let bar = Paragraph::new(Line::from(spans))
        .block(block("", PRIMARY))
        .alignment(Alignment::Center);
    f.render_widget(bar, area);
}
