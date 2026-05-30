use rayon::prelude::*;
use rust_i18n::t;
use std::time::{Duration, Instant};
use tokio::{sync::mpsc, task::AbortHandle};

use crate::{
    app::States, config::SatelliteGroupsConfig, event::Event, group::Group, object::Object,
    shared_state::SharedState, widgets::window_to_area,
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
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
    /// List entries representing available satellite groups with their
    /// selection state.
    list_entries: Vec<Entry>,
    /// The current state of the list widget.
    list_state: ListState,

    /// The current search query.
    search_query: String,
    /// Whether the search mode is active.
    is_searching: bool,

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
            cache_lifetime: Duration::from_secs(config.cache_lifetime_mins * 60),
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
    /// Returns the loaded objects.
    pub fn reload_selected_entries(&mut self) -> Vec<Object> {
        let objects = Vec::new();
        for index in 0..self.list_entries.len() {
            if self.list_entries[index].selected {
                self.load_entry(index);
            }
        }
        objects
    }

    /// Polls for async entry update results and returns new objects.
    pub fn poll_entry_updates(&mut self) -> Vec<Object> {
        let mut new_objects = Vec::new();
        while let Ok(result) = self.update_receiver.try_recv() {
            let entry = &mut self.list_entries[result.index];
            entry.loading = false;
            entry.abort_handle = None;
            if let Some(elements) = result.elements {
                new_objects.extend(
                    elements
                        .into_par_iter()
                        .map(Object::from_elements)
                        .collect::<Vec<_>>(),
                );
            } else {
                entry.selected = false;
            }
        }
        new_objects
    }

    fn scroll_up(&mut self) {
        let indices = self.filtered_indices();
        if indices.is_empty() {
            self.list_state.select(None);
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = current.saturating_sub(1);
        self.list_state.select(Some(next));
    }

    fn scroll_down(&mut self) {
        let indices = self.filtered_indices();
        if indices.is_empty() {
            self.list_state.select(None);
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let next = (current + 1).min(indices.len().saturating_sub(1));
        self.list_state.select(Some(next));
    }

    fn toggle_selected(&mut self, shared: &mut SharedState) {
        let indices = self.filtered_indices();
        let Some(selected_index) = self.list_state.selected() else {
            return;
        };
        let actual_index = indices[selected_index];

        let was_selected = self.list_entries[actual_index].selected;
        self.list_entries[actual_index].selected = !was_selected;
        shared.selected_object = None;

        if was_selected {
            // Deselecting: cancel if loading
            self.cancel_entry_loading(actual_index);
            shared.objects.clear();
            self.reload_selected_entries();
        } else {
            // Selecting: start loading
            self.load_entry(actual_index);
        }
    }

    fn filtered_indices(&self) -> Vec<usize> {
        self.list_entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                self.search_query.is_empty()
                    || entry
                        .group
                        .label()
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn max_offset(&self) -> usize {
        self.filtered_indices()
            .len()
            .saturating_sub(self.inner_area.height as usize)
    }
}

impl Default for SatelliteGroupsState {
    fn default() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            list_entries: Default::default(),
            list_state: Default::default(),
            search_query: String::new(),
            is_searching: false,
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
        let block = self.block();
        self.state.inner_area = block.inner(area);
        block.render(area, buf);

        self.render_list(buf);
        self.render_scrollbar(area, buf);
    }
}

impl SatelliteGroups<'_> {
    fn block(&self) -> Block<'static> {
        let mut title = t!("group.title").to_string();
        if self.state.is_searching {
            title = format!("{}: {}", title, self.state.search_query);
        }
        Block::bordered().title(title.blue())
    }

    fn list(&self) -> List<'static> {
        let indices = self.state.filtered_indices();
        let items = indices.into_iter().map(|index| {
            let entry = &self.state.list_entries[index];
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
        Event::Key(event) => handle_key_event(event, states),
        Event::Mouse(event) => handle_mouse_event(event, states),
        _ => Ok(()),
    }
}

/// Handle update events.
fn handle_update_event(states: &mut States) {
    let state = &mut states.satellite_groups_state;

    // Poll for async update results
    let new_objects = state.poll_entry_updates();
    states.shared.objects.extend(new_objects);

    let now = Instant::now();
    if now.duration_since(state.last_update_instant) >= state.cache_lifetime {
        states.shared.objects.clear();
        state.reload_selected_entries();
        state.last_update_instant = now;
    }
}

fn handle_key_event(event: KeyEvent, states: &mut States) -> Result<()> {
    let state = &mut states.satellite_groups_state;

    if state.is_searching {
        match event.code {
            KeyCode::Esc | KeyCode::Enter => {
                state.is_searching = false;
            }
            KeyCode::Backspace => {
                state.search_query.pop();
            }
            KeyCode::Char(c) => {
                state.search_query.push(c);
            }
            _ => {}
        }
    } else {
        match event.code {
            KeyCode::Char('/') => {
                state.is_searching = true;
                state.search_query.clear();
            }
            KeyCode::Up => state.scroll_up(),
            KeyCode::Down => state.scroll_down(),
            KeyCode::Char(' ') => state.toggle_selected(&mut states.shared),
            _ => {}
        }
    }

    Ok(())
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
            state.toggle_selected(&mut states.shared);
        }
        MouseEventKind::ScrollUp => state.scroll_up(),
        MouseEventKind::ScrollDown => state.scroll_down(),
        _ => {}
    }

    // Highlight the hovered entry.
    let row = local_mouse.y as usize + state.list_state.offset();
    let index = if row < state.filtered_indices().len() {
        Some(row)
    } else {
        None
    };
    state.list_state.select(index);

    Ok(())
}
