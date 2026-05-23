use super::{ACCENT, block, centered_rect, keybinds};
use crate::core::PasswordGen;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

pub fn render_generator(f: &mut Frame, g: &PasswordGen, standalone: bool) {
    let area = centered_rect(60, 55, f.area());
    f.render_widget(Clear, area);
    f.render_widget(block("Password Generator", ACCENT), area);

    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Generated password
    f.render_widget(
        Paragraph::new(Span::styled(
            g.password.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .block(block("Generated password", ACCENT)),
        Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: 3,
        },
    );

    // Length row
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                "j ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}  ", g.length)),
            Span::styled(
                "k",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(block("Length", Color::DarkGray))
        .alignment(Alignment::Center),
        Rect {
            x: inner.x,
            y: inner.y + 3,
            width: inner.width,
            height: 3,
        },
    );

    // Entropy + strength
    let strength_color = match g.score() {
        0 | 1 => Color::Red,
        2 => Color::Yellow,
        3 => Color::Green,
        _ => Color::Cyan,
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(format!("{:.1} bits   ", g.entropy_bits())),
            Span::styled(
                g.strength_label(),
                Style::default()
                    .fg(strength_color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .block(block("Entropy / Strength", Color::DarkGray))
        .alignment(Alignment::Center),
        Rect {
            x: inner.x,
            y: inner.y + 6,
            width: inner.width,
            height: 3,
        },
    );

    // Character sets
    fn toggle_span(label: &'static str, key: &'static str, on: bool) -> Vec<Span<'static>> {
        let bracket_color = if on { Color::Green } else { Color::DarkGray };
        vec![
            Span::styled(
                if on { "[x] " } else { "[ ] " },
                Style::default()
                    .fg(bracket_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(label),
            Span::styled(format!("({key})  "), Style::default().fg(ACCENT)),
        ]
    }
    let mut charset_spans: Vec<Span> = Vec::new();
    charset_spans.extend(toggle_span("Upper", "u", g.use_upper));
    charset_spans.extend(toggle_span("Lower", "l", g.use_lower));
    charset_spans.extend(toggle_span("Digits", "d", g.use_digits));
    charset_spans.extend(toggle_span("Symbols", "s", g.use_symbols));

    f.render_widget(
        Paragraph::new(Line::from(charset_spans))
            .block(block("Character sets", Color::DarkGray))
            .alignment(Alignment::Center),
        Rect {
            x: inner.x,
            y: inner.y + 9,
            width: inner.width,
            height: 3,
        },
    );

    // Keybinds
    let enter_label = if standalone { "Close" } else { "Use" };
    f.render_widget(
        Paragraph::new(Line::from(keybinds(&[
            ("r / Space", "Regenerate"),
            ("c", "Copy"),
            ("Enter", enter_label),
            ("Esc", "Cancel"),
        ])))
        .block(block("", Color::DarkGray))
        .alignment(Alignment::Center),
        Rect {
            x: inner.x,
            y: inner.y + 12,
            width: inner.width,
            height: 3,
        },
    );
}
