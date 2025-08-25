use ratatui::prelude::*;

pub mod information;
pub mod satellite_groups;
pub mod sky;
pub mod tabs;
pub mod world_map;

/// Converts window coordinates to area coordinates.
#[must_use]
fn window_to_area(global: Position, area: Rect) -> Option<Position> {
    if !area.contains(global) {
        return None;
    }
    Some(Position::new(global.x - area.x, global.y - area.y))
}
