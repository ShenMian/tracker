use ratatui::{
    prelude::*,
    widgets::{Block, Clear, Paragraph},
};
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

// Global
const GLOBAL_BINDINGS: &[(&str, &str)] = &[
    ("q, <C-c>", "keymap.quit"),
    ("?", "keymap.help"),
    ("<Esc>", "keymap.dismiss"),
    ("<Tab> / <S-Tab>", "keymap.switch_tab"),
];

// World Map
const MAP_BINDINGS: &[(&str, &str)] = &[
    ("<LeftMouse>", "keymap.select"),
    ("<RightMouse>", "keymap.deselect"),
    ("<ScrollWheelUp> / <ScrollWheelDown>", "keymap.map_move"),
    ("[ / ]", "keymap.map_move"),
    ("f", "keymap.follow"),
    ("t", "keymap.terminator"),
];

// Timeline
const TIMELINE_BINDINGS: &[(&str, &str)] = &[
    ("<ScrollWheelUp> / <ScrollWheelDown>", "keymap.adjust_time"),
    ("r", "keymap.reset_time"),
];

const SECTIONS: &[(&str, &[(&str, &str)])] = &[
    ("keymap.global", GLOBAL_BINDINGS),
    ("keymap.world_map", MAP_BINDINGS),
    ("keymap.timeline", TIMELINE_BINDINGS),
];

pub struct Keymap;

impl Keymap {
    fn block() -> Block<'static> {
        Block::bordered().title(t!("keymap.title").to_string().blue())
    }
}

impl Widget for Keymap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Translate section names and descriptions
        let sections: Vec<_> = SECTIONS
            .iter()
            .map(|(section_key, bindings)| {
                let translated_bindings: Vec<(&str, String)> = bindings
                    .iter()
                    .map(|(key, i18n_key)| (*key, t!(*i18n_key).to_string()))
                    .collect();
                (t!(*section_key).to_string(), translated_bindings)
            })
            .collect();

        // Calculate max widths
        let (key_width, desc_width) = sections.iter().flat_map(|(_, bindings)| bindings).fold(
            (0usize, 0usize),
            |(key_width, desc_width), (key, desc)| {
                (key_width.max(key.width()), desc_width.max(desc.width()))
            },
        );

        // Build lines
        let mut lines = Vec::new();
        for (i, (section_title, bindings)) in sections.into_iter().enumerate() {
            if i > 0 {
                lines.push(Line::raw(""));
            }
            lines.push(
                Line::styled(
                    format!(" {} ", section_title),
                    Style::default().bold().reversed(),
                )
                .centered(),
            );

            for (key, desc) in bindings {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{:>key_width$}", key),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(" "),
                    Span::raw(desc),
                ]));
            }
        }

        let inner_width = key_width as u16 + 1 + desc_width as u16;
        let inner_height = lines.len() as u16;

        const BORDER_WIDTH: u16 = 1;
        let popup_area = centered_rect(
            inner_width + BORDER_WIDTH * 2,
            inner_height + BORDER_WIDTH * 2,
            area,
        );

        Clear.render(popup_area, buf);
        Paragraph::new(lines)
            .block(Self::block())
            .render(popup_area, buf);
    }
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect::new(x, y, width.min(area.width), height.min(area.height))
}
