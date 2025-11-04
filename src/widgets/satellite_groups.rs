use rust_i18n::t;
use std::time::{Duration, Instant};

use crate::{
    app::States, config::SatelliteGroupsConfig, event::Event, group::Group, object::Object,
    widgets::window_to_area,
};
use anyhow::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    style::Styled,
    widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarState},
};

/// A widget that displays a list of satellite groups.
pub struct SatelliteGroups<'a> {
    pub state: &'a mut SatelliteGroupsState,
}

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
            cache_lifetime: Duration::from_mins(config.cache_lifetime_min),
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
        *self.list_state.offset_mut() = (self.list_state.offset() + 1).min(self.max_offset());
    }

    fn max_offset(&self) -> usize {
        self.list_entries
            .len()
            .saturating_sub(self.inner_area.height as usize)
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

impl Widget for SatelliteGroups<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let block = Self::block();
        self.state.inner_area = block.inner(area);
        block.render(area, buf);

        self.render_list(buf);
        self.render_scrollbar(area, buf);
    }
}

impl SatelliteGroups<'_> {
    fn block() -> Block<'static> {
        Block::bordered().title(t!("group.title").to_string().blue())
    }

    fn list(&self) -> List<'static> {
        let items = self.state.list_entries.iter().map(|entry| {
            let icon = if entry.selected { "✓" } else { "☐" };
            let style = if entry.selected {
                Style::new().fg(Color::White)
            } else {
                Style::new()
            };
            ListItem::new(format!("{} {}", icon, entry.satellite.label()).set_style(style))
        });
        List::new(items).highlight_style(Style::new().add_modifier(Modifier::REVERSED))
    }

    fn render_list(&mut self, buf: &mut Buffer) {
        StatefulWidget::render(
            self.list(),
            self.state.inner_area,
            buf,
            &mut self.state.list_state,
        );
    }

    fn render_scrollbar(&self, area: Rect, buf: &mut Buffer) {
        let inner_area = area.inner(Margin::new(0, 1));
        Scrollbar::default().render(
            inner_area,
            buf,
            &mut ScrollbarState::new(self.state.max_offset())
                .position(self.state.list_state.offset()),
        );
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

pub async fn handle_event(event: Event, states: &mut States) -> Result<()> {
    match event {
        Event::Update => {
            handle_update_event(states).await;
            Ok(())
        }
        Event::Mouse(event) => handle_mouse_event(event, states).await,
        _ => Ok(()),
    }
}

/// Handle update events.
async fn handle_update_event(states: &mut States) {
    let state = &mut states.satellite_groups_state;

    let now = Instant::now();
    if now.duration_since(state.last_update_instant) >= state.cache_lifetime {
        state.refresh_objects().await;
        state.last_update_instant = now;
    }
}

async fn handle_mouse_event(event: MouseEvent, states: &mut States) -> Result<()> {
    let state = &mut states.satellite_groups_state;

    let global_mouse = Position::new(event.column, event.row);
    let Some(local_mouse) = window_to_area(global_mouse, state.inner_area) else {
        *state.list_state.selected_mut() = None;
        return Ok(());
    };

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Select the clicked entry.
            if let Some(index) = state.list_state.selected() {
                state.list_entries[index].selected = !state.list_entries[index].selected;
                states.world_map_state.selected_object = None;
                state.refresh_objects().await;
            }
        }
        MouseEventKind::ScrollUp => state.scroll_up(),
        MouseEventKind::ScrollDown => state.scroll_down(),
        _ => {}
    }

    // Highlight the hovered entry.
    let row = local_mouse.y as usize + state.list_state.offset();
    let index = if row < state.list_entries.len() {
        Some(row)
    } else {
        None
    };
    state.list_state.select(index);

    Ok(())
}
