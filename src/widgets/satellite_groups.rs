use rust_i18n::t;
use std::time::{Duration, Instant};
use tokio::{sync::mpsc, task::AbortHandle};

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
    /// Sender for async data updates.
    update_sender: mpsc::UnboundedSender<UpdateResult>,
    /// Receiver for async data updates.
    update_receiver: mpsc::UnboundedReceiver<UpdateResult>,
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

    /// Spawns async task to load orbital elements for a single entry.
    fn load_entry(&mut self, index: usize) {
        let entry = &mut self.list_entries[index];
        entry.loading = true;
        let tx = self.update_sender.clone();
        let group = entry.group.clone();
        let cache_lifetime = self.cache_lifetime;
        let handle = tokio::spawn(async move {
            let elements = group.get_elements(cache_lifetime).await;
            let _ = tx.send(UpdateResult { index, elements });
        });
        entry.abort_handle = Some(handle.abort_handle());
    }

    /// Cancels the entry loading task at the given index.
    fn cancel_entry_loading(&mut self, index: usize) {
        let entry = &mut self.list_entries[index];
        if let Some(handle) = entry.abort_handle.take() {
            handle.abort();
        }
        entry.loading = false;
    }

    /// Spawns async tasks to reload orbital elements for all selected entries.
    pub fn reload_selected_entries(&mut self) {
        self.objects.clear();
        for index in 0..self.list_entries.len() {
            if self.list_entries[index].selected {
                self.load_entry(index);
            }
        }
    }

    /// Polls for async entry update results.
    pub fn poll_entry_updates(&mut self) {
        while let Ok(result) = self.update_receiver.try_recv() {
            let entry = &mut self.list_entries[result.index];
            entry.loading = false;
            entry.abort_handle = None;
            if let Some(elements) = result.elements {
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
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            objects: Vec::new(),
            list_entries: Default::default(),
            list_state: Default::default(),
            inner_area: Default::default(),
            cache_lifetime: Default::default(),
            last_update_instant: Instant::now(),
            update_sender: tx,
            update_receiver: rx,
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
            let icon = if entry.loading {
                "⋯"
            } else if entry.selected {
                "✓"
            } else {
                "☐"
            };
            let style = if entry.selected {
                Style::new().fg(Color::White)
            } else {
                Style::new()
            };
            ListItem::new(format!("{} {}", icon, entry.group.label()).set_style(style))
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

/// Result of an async satellite group update task.
struct UpdateResult {
    /// Index of the entry in the list that was updated.
    index: usize,
    /// Fetched orbital elements, or `None` if the fetch failed.
    elements: Option<Vec<sgp4::Elements>>,
}

/// A satellite group entry in the list.
pub struct Entry {
    /// The satellite group.
    group: Group,
    /// Whether this entry is selected.
    selected: bool,
    /// Whether this entry is currently loading data.
    loading: bool,
    /// Handle to abort the loading task.
    abort_handle: Option<AbortHandle>,
}

impl From<Group> for Entry {
    fn from(group: Group) -> Self {
        Self {
            group,
            selected: false,
            loading: false,
            abort_handle: None,
        }
    }
}

pub fn handle_event(event: Event, states: &mut States) -> Result<()> {
    match event {
        Event::Update => {
            handle_update_event(states);
            Ok(())
        }
        Event::Mouse(event) => handle_mouse_event(event, states),
        _ => Ok(()),
    }
}

/// Handle update events.
fn handle_update_event(states: &mut States) {
    let state = &mut states.satellite_groups_state;

    // Poll for async update results
    state.poll_entry_updates();

    let now = Instant::now();
    if now.duration_since(state.last_update_instant) >= state.cache_lifetime {
        state.reload_selected_entries();
        state.last_update_instant = now;
    }
}

fn handle_mouse_event(event: MouseEvent, states: &mut States) -> Result<()> {
    let state = &mut states.satellite_groups_state;

    let global_mouse = Position::new(event.column, event.row);
    let Some(local_mouse) = window_to_area(global_mouse, state.inner_area) else {
        *state.list_state.selected_mut() = None;
        return Ok(());
    };

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Toggle selection of the clicked entry.
            if let Some(index) = state.list_state.selected() {
                let was_selected = state.list_entries[index].selected;
                state.list_entries[index].selected = !was_selected;
                states.world_map_state.selected_object = None;

                if was_selected {
                    // Deselecting: cancel if loading
                    state.cancel_entry_loading(index);
                    state.reload_selected_entries();
                } else {
                    // Selecting: start loading
                    state.load_entry(index);
                }
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
