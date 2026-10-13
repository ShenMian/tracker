use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

pub mod information;
pub mod keymap;
pub mod satellite_groups;
pub mod sky;
pub mod tabs;
pub mod timeline;
pub mod world_map;

/// Converts window coordinates to area coordinates.
#[must_use]
fn window_to_area(global: Position, area: Rect) -> Option<Position> {
    if !area.contains(global) {
        return None;
    }
    Some(Position::new(global.x - area.x, global.y - area.y))
}

/// Builds a centered paragraph with word wrapping.
fn centered_paragraph<'a>(text: impl Into<Text<'a>>) -> Paragraph<'a> {
    Paragraph::new(text).centered().wrap(Wrap { trim: true })
}
