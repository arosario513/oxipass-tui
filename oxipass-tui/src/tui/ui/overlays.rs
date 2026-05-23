use super::{ACCENT, DANGER, PRIMARY, block, centered_rect, keybinds};
use crate::tui::form::EntryForm;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

pub fn render_form(f: &mut Frame, form: &EntryForm) {
    let area = centered_rect(55, 70, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block(form.title(), PRIMARY), area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    let field_height = 3u16;
    for (i, field) in form.fields.iter().enumerate() {
        let y = inner.y + i as u16 * field_height;
        if y + field_height > area.y + area.height {
            break;
        }
        let field_area = Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: field_height,
        };

        let label = if field.optional {
            format!("{} (optional)", field.label)
        } else {
            field.label.to_string()
        };

        let border_color = if field.invalid {
            Color::Red
        } else if i == form.focused {
            ACCENT
        } else {
            Color::DarkGray
        };

        f.render_widget(
            Paragraph::new(field.display()).block(block(&label, border_color)),
            field_area,
        );

        if i == form.focused {
            f.set_cursor_position((field_area.x + 1 + field.cursor as u16, field_area.y + 1));
        }
    }
}

pub fn render_pending_add(f: &mut Frame) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(Line::from(keybinds(&[
            ("l", "Login"),
            ("p", "Payment"),
            ("n", "Note"),
        ])))
        .block(block("Add entry", ACCENT))
        .alignment(Alignment::Center),
        area,
    );
}

pub fn render_confirm(f: &mut Frame) {
    let area = centered_rect(40, 20, f.area());
    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "y",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Confirm  "),
            Span::styled(
                "n / Esc",
                Style::default().fg(DANGER).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Cancel"),
        ]))
        .block(block("Delete entry?", DANGER))
        .alignment(Alignment::Center),
        area,
    );
}
