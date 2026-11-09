use rayon::prelude::*;
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
    /// Index of the entry whose first object is selected once it has loaded.
    follow_index: Option<usize>,
    /// The followed entry's first object, until it is taken.
    followed_object: Option<Object>,
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
            cache_lifetime: Duration::from_mins(config.cache_lifetime_mins),
            ..Self::default()
        }
    }

    /// Spawns async task to load orbital elements for a single entry.
    fn load_entry(&mut self, index: usize) {
        let entry = &mut self.list_entries[index];
        let tx = self.update_sender.clone();
        let group = entry.group.clone();
        let cache_lifetime = self.cache_lifetime;
        let handle = tokio::spawn(async move {
            let elements = group.get_elements(cache_lifetime).await.ok();
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
    }

    /// Spawns async tasks to reload orbital elements for all selected entries.
    pub fn reload_selected_entries(&mut self) {
        for index in 0..self.list_entries.len() {
            if self.list_entries[index].selected {
                self.load_entry(index);
            }
        }
    }

    /// Selects the groups with these labels and notes the group to follow.
    pub fn start_with(&mut self, groups: &[String], follow: Option<&str>) {
        for entry in &mut self.list_entries {
            entry.selected = groups.iter().any(|group| group == entry.group.label());
        }
        self.follow_index = self
            .list_entries
            .iter()
            .position(|entry| Some(entry.group.label()) == follow);
    }

    /// Returns the followed group's first object, once, after it has loaded.
    pub fn take_followed_object(&mut self) -> Option<Object> {
        self.followed_object.take()
    }

    /// Polls for async entry update results and returns new objects.
    pub fn poll_entry_updates(&mut self) -> Vec<Object> {
        let mut new_objects = Vec::new();
        while let Ok(result) = self.update_receiver.try_recv() {
            let entry = &mut self.list_entries[result.index];
            entry.abort_handle = None;
            if let Some(elements) = result.elements {
                let objects = elements
                    .into_par_iter()
                    .filter_map(|elements| Object::from_elements(elements).ok())
                    .collect::<Vec<_>>();
                if self.follow_index == Some(result.index) {
                    self.follow_index = None;
                    self.followed_object = objects.first().cloned();
                }
                new_objects.extend(objects);
            } else {
                entry.selected = false;
            }
        }
        new_objects
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
            list_entries: Default::default(),
            list_state: Default::default(),
            inner_area: Default::default(),
            cache_lifetime: Default::default(),
            last_update_instant: Instant::now(),
            update_sender: tx,
            update_receiver: rx,
            follow_index: None,
            followed_object: None,
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
        Block::bordered().title(t!("group-title").to_string().blue())
    }

    fn list(&self) -> List<'static> {
        let items = self.state.list_entries.iter().map(|entry| {
            let icon = if entry.abort_handle.is_some() {
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
    /// Handle to abort the loading task.
    abort_handle: Option<AbortHandle>,
}

impl From<Group> for Entry {
    fn from(group: Group) -> Self {
        Self {
            group,
            selected: false,
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
    let new_objects = state.poll_entry_updates();
    states.shared.objects.extend(new_objects);

    // Select the object of the group followed from the command line
    if let Some(object) = state.take_followed_object() {
        states.shared.selected_object = Some(object);
    }

    let now = Instant::now();
    if now.duration_since(state.last_update_instant) >= state.cache_lifetime {
        states.shared.objects.clear();
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
                states.shared.selected_object = None;

                if was_selected {
                    // Deselecting: cancel if loading
                    state.cancel_entry_loading(index);
                    states.shared.objects.clear();
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

#[cfg(test)]
mod tests {
    use super::*;

    const ISS: usize = 0;
    const CSS: usize = 1;

    fn state() -> SatelliteGroupsState {
        SatelliteGroupsState::with_config(SatelliteGroupsConfig::default())
    }

    fn labels(names: &[&str]) -> Vec<String> {
        names.iter().map(ToString::to_string).collect()
    }

    fn selected_labels(state: &SatelliteGroupsState) -> Vec<&str> {
        state
            .list_entries
            .iter()
            .filter(|entry| entry.selected)
            .map(|entry| entry.group.label())
            .collect()
    }

    /// Orbital elements of a real TLE, under the given name.
    fn elements(name: &str) -> sgp4::Elements {
        sgp4::Elements::from_tle(
            Some(name.to_string()),
            b"1 25544U 98067A   20194.88612269 -.00002218  00000-0 -31515-4 0  9992",
            b"2 25544  51.6461 221.2784 0001413  89.1723 280.4612 15.49507896236008",
        )
        .unwrap()
    }

    fn finish_loading(
        state: &mut SatelliteGroupsState,
        index: usize,
        elements: Option<Vec<sgp4::Elements>>,
    ) {
        state
            .update_sender
            .send(UpdateResult { index, elements })
            .unwrap();
        state.poll_entry_updates();
    }

    #[test]
    fn start_selects_only_the_named_groups() {
        let mut state = state();
        state.start_with(&labels(&["CSS"]), None);
        assert_eq!(selected_labels(&state), ["CSS"]);
    }

    #[test]
    fn start_without_groups_selects_nothing() {
        let mut state = state();
        state.start_with(&[], None);
        assert!(selected_labels(&state).is_empty());
    }

    #[test]
    fn followed_group_gives_its_first_object() {
        let mut state = state();
        state.start_with(&labels(&["ISS", "CSS"]), Some("ISS"));
        finish_loading(
            &mut state,
            ISS,
            Some(vec![elements("FIRST"), elements("SECOND")]),
        );
        let object = state.take_followed_object().unwrap();
        assert_eq!(object.name(), Some("FIRST"));
    }

    #[test]
    fn a_group_that_loads_first_is_not_followed_by_mistake() {
        let mut state = state();
        state.start_with(&labels(&["ISS", "CSS"]), Some("ISS"));

        finish_loading(&mut state, CSS, Some(vec![elements("CSS OBJECT")]));
        assert!(state.take_followed_object().is_none());

        finish_loading(&mut state, ISS, Some(vec![elements("ISS OBJECT")]));
        let object = state.take_followed_object().unwrap();
        assert_eq!(object.name(), Some("ISS OBJECT"));
    }

    #[test]
    fn nothing_is_followed_without_the_flag() {
        let mut state = state();
        state.start_with(&labels(&["ISS"]), None);
        finish_loading(&mut state, ISS, Some(vec![elements("ISS OBJECT")]));
        assert!(state.take_followed_object().is_none());
    }

    #[test]
    fn a_failed_download_gives_no_object() {
        let mut state = state();
        state.start_with(&labels(&["ISS"]), Some("ISS"));
        finish_loading(&mut state, ISS, None);
        assert!(state.take_followed_object().is_none());
    }

    #[test]
    fn a_reload_does_not_select_the_object_again() {
        let mut state = state();
        state.start_with(&labels(&["ISS"]), Some("ISS"));
        finish_loading(&mut state, ISS, Some(vec![elements("ISS OBJECT")]));
        assert!(state.take_followed_object().is_some());

        finish_loading(&mut state, ISS, Some(vec![elements("ISS OBJECT")]));
        assert!(state.take_followed_object().is_none());
    }
}
