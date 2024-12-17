use std::time::Instant;

use anyhow::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, List, ListItem, ListState, Scrollbar, ScrollbarState, StatefulWidget, Widget,
    },
};
use strum::IntoEnumIterator;

use crate::{app::App, object::Object, satellite::Satellite};

#[derive(Default)]
pub struct Satellites;

pub struct SatellitesState {
    pub objects: Vec<Object>,

    pub items: Vec<Item>,
    pub list_state: ListState,

    pub inner_area: Rect,

    pub last_object_update: Instant,
}

impl SatellitesState {
    /// Updates the orbital elements for selected satellites.
    pub async fn refresh_objects(&mut self) {
        self.objects.clear();
        for item in &mut self.items {
            if !item.selected {
                continue;
            }
            if let Some(elements) = item.satellite.get_elements().await {
                self.objects
                    .extend(elements.into_iter().map(Object::from_elements));
            } else {
                item.selected = false;
            }
        }
    }

    pub fn scroll_up(&mut self) {
        *self.list_state.offset_mut() = self.list_state.offset().saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max_offset = self
            .items
            .len()
            .saturating_sub(self.inner_area.height as usize);
        *self.list_state.offset_mut() = (self.list_state.offset() + 1).min(max_offset);
    }
}

impl Default for SatellitesState {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            items: Satellite::iter().map(Item::from).collect(),
            list_state: Default::default(),
            inner_area: Default::default(),
            last_object_update: Instant::now(),
        }
    }
}

impl Satellites {
    fn render_block(&self, area: Rect, buf: &mut Buffer, state: &mut SatellitesState) {
        let block = Block::bordered().title("Satellites".blue());
        state.inner_area = block.inner(area);
        block.render(area, buf);
    }

    fn render_list(&self, buf: &mut Buffer, state: &mut SatellitesState) {
        let items = state.items.iter().map(|item| {
            let style = if item.selected {
                Style::default().fg(Color::White)
            } else {
                Style::default()
            };
            let text = if item.selected {
                format!("✓ {}", item.satellite)
            } else {
                format!("☐ {}", item.satellite)
            };
            ListItem::new(Text::styled(text, style))
        });

        let list =
            List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        StatefulWidget::render(list, state.inner_area, buf, &mut state.list_state);
    }

    fn render_scrollbar(&self, area: Rect, buf: &mut Buffer, state: &mut SatellitesState) {
        let inner_area = area.inner(Margin::new(0, 1));
        let mut scrollbar_state =
            ScrollbarState::new(state.items.len().saturating_sub(inner_area.height as usize))
                .position(state.list_state.offset());
        Scrollbar::default().render(inner_area, buf, &mut scrollbar_state);
    }
}

impl StatefulWidget for Satellites {
    type State = SatellitesState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_block(area, buf, state);
        self.render_list(buf, state);
        self.render_scrollbar(area, buf, state);
    }
}

pub struct Item {
    pub satellite: Satellite,
    selected: bool,
}

impl From<Satellite> for Item {
    fn from(satellite: Satellite) -> Self {
        Self {
            satellite,
            selected: false,
        }
    }
}

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.satellites_state.inner_area;
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.satellites_state.list_state.select(None);
        return Ok(());
    }

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Select the clicked item.
            if let Some(index) = app.satellites_state.list_state.selected() {
                app.satellites_state.items[index].selected =
                    !app.satellites_state.items[index].selected;
                app.world_map_state.selected_object = None;
                app.world_map_state.hovered_object = None;
                app.satellites_state.refresh_objects().await;
            }
        }
        MouseEventKind::ScrollUp => app.satellites_state.scroll_up(),
        MouseEventKind::ScrollDown => app.satellites_state.scroll_down(),
        _ => {}
    }
    // Highlight the hovered item.
    let row = (event.row - inner_area.y) as usize + app.satellites_state.list_state.offset();
    let index = if row < app.satellites_state.items.len() {
        Some(row)
    } else {
        None
    };
    app.satellites_state.list_state.select(index);

    Ok(())
}
