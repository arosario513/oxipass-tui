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
    let total_height: u16 = form
        .fields
        .iter()
        .map(|field| if field.multiline { 7 } else { 3 })
        .sum::<u16>()
        + 2; // border
    let percent_y =
        ((total_height as f32 / f.area().height as f32) * 100.0).clamp(40.0, 95.0) as u16;
    let area = centered_rect(55, percent_y, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block(form.title(), PRIMARY), area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    let mut y = inner.y;
    for (i, field) in form.fields.iter().enumerate() {
        let field_height: u16 = if field.multiline { 7 } else { 3 };
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

        if i == form.focused {
            let (row, col) = field.cursor_pos();
            let inner_height = field_height.saturating_sub(2);
            let scroll_row = row.saturating_sub(inner_height.saturating_sub(1));
            f.render_widget(
                Paragraph::new(field.display())
                    .block(block(&label, border_color))
                    .scroll((scroll_row, 0)),
                field_area,
            );
            f.set_cursor_position((
                field_area.x + 1 + col,
                field_area.y + 1 + (row - scroll_row),
            ));
        } else {
            f.render_widget(
                Paragraph::new(field.display()).block(block(&label, border_color)),
                field_area,
            );
        }

        y += field_height;
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

pub fn render_copy_picker(f: &mut Frame, fields: &[(String, String)]) {
    let height = fields.len() as u16 * 2 + 2;
    let area = Rect {
        x: (f.area().width.saturating_sub(44)) / 2,
        y: (f.area().height.saturating_sub(height)) / 2,
        width: 44,
        height,
    };
    f.render_widget(Clear, area);
    f.render_widget(block("Copy field", ACCENT), area);

    for (i, (label, _)) in fields.iter().enumerate() {
        let y = area.y + 1 + i as u16 * 2;
        if y >= area.y + area.height {
            break;
        }
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{}  ", i + 1),
                    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
                ),
                Span::raw(label.clone()),
            ])),
            Rect {
                x: area.x + 2,
                y,
                width: area.width.saturating_sub(4),
                height: 1,
            },
        );
    }
}
