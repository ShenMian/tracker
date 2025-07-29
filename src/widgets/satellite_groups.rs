use std::time::Instant;

use anyhow::Result;
use chrono::{DateTime, Utc};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarState},
};
use strum::IntoEnumIterator;

use crate::{app::App, object::Object, satellite_group::SatelliteGroup};

/// A widget to display a list of satellite groups.
#[derive(Default)]
pub struct SatelliteGroups;

/// State of a [`SatelliteGroups`] widget
pub struct SatelliteGroupsState {
    pub objects: Vec<Object>,

    pub list_entries: Vec<Entry>,
    pub list_state: ListState,

    pub last_object_update: Instant,

    inner_area: Rect,
}

impl SatelliteGroupsState {
    /// Updates the orbital elements for selected satellite group.
    pub async fn refresh_objects(&mut self) {
        self.objects.clear();
        for entry in self.list_entries.iter_mut().filter(|e| e.selected) {
            if let Some(elements) = entry.satellite.get_elements().await {
                self.objects
                    .extend(elements.into_iter().map(Object::from_elements));
            } else {
                entry.selected = false;
            }
        }
    }

    /// Get the index of the nearest object to the given area coordinates
    pub fn get_nearest_object_index(
        &self,
        time: DateTime<Utc>,
        lon: f64,
        lat: f64,
    ) -> Option<usize> {
        self.objects
            .iter()
            .enumerate()
            .min_by_key(|(_, obj)| {
                let state = obj.predict(time).unwrap();
                let lon_diff = state.longitude() - lon;
                let lat_diff = state.latitude() - lat;
                ((lon_diff.powi(2) + lat_diff.powi(2)) * 1000.0) as i32
            })
            .map(|(index, _)| index)
    }

    pub fn scroll_up(&mut self) {
        *self.list_state.offset_mut() = self.list_state.offset().saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max_offset = self
            .list_entries
            .len()
            .saturating_sub(self.inner_area.height as usize);
        *self.list_state.offset_mut() = (self.list_state.offset() + 1).min(max_offset);
    }
}

impl Default for SatelliteGroupsState {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            list_entries: SatelliteGroup::iter().map(Entry::from).collect(),
            list_state: Default::default(),
            inner_area: Default::default(),
            last_object_update: Instant::now(),
        }
    }
}

impl SatelliteGroups {
    fn render_block(&self, area: Rect, buf: &mut Buffer, state: &mut SatelliteGroupsState) {
        let block = Block::bordered().title("Satellite groups".blue());
        state.inner_area = block.inner(area);
        block.render(area, buf);
    }

    fn render_list(&self, buf: &mut Buffer, state: &mut SatelliteGroupsState) {
        let items = state.list_entries.iter().map(|entry| {
            let style = if entry.selected {
                Style::default().fg(Color::White)
            } else {
                Style::default()
            };
            let icon = if entry.selected { "✓" } else { "☐" };
            ListItem::new(Text::styled(format!("{} {}", icon, entry.satellite), style))
        });

        let list =
            List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        StatefulWidget::render(list, state.inner_area, buf, &mut state.list_state);
    }

    fn render_scrollbar(&self, area: Rect, buf: &mut Buffer, state: &mut SatelliteGroupsState) {
        let inner_area = area.inner(Margin::new(0, 1));
        let mut scrollbar_state = ScrollbarState::new(
            state
                .list_entries
                .len()
                .saturating_sub(inner_area.height as usize),
        )
        .position(state.list_state.offset());
        Scrollbar::default().render(inner_area, buf, &mut scrollbar_state);
    }
}

impl StatefulWidget for SatelliteGroups {
    type State = SatelliteGroupsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_block(area, buf, state);
        self.render_list(buf, state);
        self.render_scrollbar(area, buf, state);
    }
}

pub struct Entry {
    pub satellite: SatelliteGroup,
    selected: bool,
}

impl From<SatelliteGroup> for Entry {
    fn from(satellite: SatelliteGroup) -> Self {
        Self {
            satellite,
            selected: false,
        }
    }
}

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.satellite_groups_state.inner_area;
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.satellite_groups_state.list_state.select(None);
        return Ok(());
    }

    // Convert window coordinates to area coordinates
    let mouse = Position::new(event.column - inner_area.x, event.row - inner_area.y);

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Select the clicked entry.
            if let Some(index) = app.satellite_groups_state.list_state.selected() {
                app.satellite_groups_state.list_entries[index].selected =
                    !app.satellite_groups_state.list_entries[index].selected;
                app.world_map_state.selected_object_index = None;
                app.satellite_groups_state.refresh_objects().await;
            }
        }
        MouseEventKind::ScrollUp => app.satellite_groups_state.scroll_up(),
        MouseEventKind::ScrollDown => app.satellite_groups_state.scroll_down(),
        _ => {}
    }

    // Highlight the hovered entry.
    let row = mouse.y as usize + app.satellite_groups_state.list_state.offset();
    let index = if row < app.satellite_groups_state.list_entries.len() {
        Some(row)
    } else {
        None
    };
    app.satellite_groups_state.list_state.select(index);

    Ok(())
}
