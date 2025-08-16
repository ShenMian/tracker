use rust_i18n::t;
use std::time::{Duration, Instant};

use crate::{app::App, config::SatelliteGroupsConfig, event::Event, group::Group, object::Object};
use anyhow::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarState},
};

/// A widget to display a list of satellite groups.
#[derive(Default)]
pub struct SatelliteGroups;

/// State of a [`SatelliteGroups`] widget.
pub struct SatelliteGroupsState {
    /// Collection of satellite objects loaded from the selected satellite
    /// groups.
    pub objects: Vec<Object>,
    /// List entries representing available satellite groups with their
    /// selection state.
    list_entries: Vec<Entry>,
    /// The current state of the list widget.
    list_state: ListState,
    /// Timestamp of the last orbital elements update.
    last_update_instant: Instant,
    /// Duration that cached orbital elements remain valid before requiring a
    /// refresh.
    cache_lifetime: Duration,
    /// The inner rendering area of the widget.
    inner_area: Rect,
}

impl SatelliteGroupsState {
    /// Creates a new `SatelliteGroupsState` with the given configuration.
    pub fn with_config(config: SatelliteGroupsConfig) -> Self {
        Self {
            list_entries: config
                .groups
                .into_iter()
                .map(Group::from)
                .map(Entry::from)
                .collect(),
            cache_lifetime: Duration::from_secs(config.cache_lifetime_min * 60),
            ..Self::default()
        }
    }

    /// Updates the orbital elements for selected satellite group.
    pub async fn refresh_objects(&mut self) {
        self.objects.clear();
        for entry in self.list_entries.iter_mut().filter(|e| e.selected) {
            if let Some(elements) = entry.satellite.get_elements(self.cache_lifetime).await {
                self.objects
                    .extend(elements.into_iter().map(Object::from_elements));
            } else {
                entry.selected = false;
            }
        }
    }

    fn scroll_up(&mut self) {
        *self.list_state.offset_mut() = self.list_state.offset().saturating_sub(1);
    }

    fn scroll_down(&mut self) {
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
            list_entries: Default::default(),
            list_state: Default::default(),
            inner_area: Default::default(),
            cache_lifetime: Default::default(),
            last_update_instant: Instant::now(),
        }
    }
}

impl SatelliteGroups {
    fn render_block(&self, area: Rect, buf: &mut Buffer, state: &mut SatelliteGroupsState) {
        let block = Block::bordered().title(t!("sg.title").to_string().blue());
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
            ListItem::new(Text::styled(
                format!("{} {}", icon, entry.satellite.label()),
                style,
            ))
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
    satellite: Group,
    selected: bool,
}

impl From<Group> for Entry {
    fn from(satellite: Group) -> Self {
        Self {
            satellite,
            selected: false,
        }
    }
}

pub async fn handle_event(event: Event, app: &mut App) -> Result<()> {
    match event {
        Event::Update => {
            handle_update_event(app).await;
            Ok(())
        }
        Event::Mouse(event) => handle_mouse_event(event, app).await,
        _ => Ok(()),
    }
}

/// Handle update events.
async fn handle_update_event(app: &mut App) {
    // Refresh satellite data every 2 minutes.
    const OBJECT_UPDATE_INTERVAL: Duration = Duration::from_secs(2 * 60);
    let now = Instant::now();
    if now.duration_since(app.satellite_groups_state.last_update_instant) >= OBJECT_UPDATE_INTERVAL
    {
        app.satellite_groups_state.refresh_objects().await;
        app.satellite_groups_state.last_update_instant = now;
    }
}

async fn handle_mouse_event(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.satellite_groups_state.inner_area;
    if !inner_area.contains(Position::new(event.column, event.row)) {
        *app.satellite_groups_state.list_state.selected_mut() = None;
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
