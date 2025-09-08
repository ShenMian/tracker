use std::borrow::Cow;

use anyhow::Result;
use arboard::Clipboard;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    style::palette::tailwind,
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarState, StatefulWidget, Table,
        TableState, Wrap,
    },
};
use rust_i18n::t;
use unicode_width::UnicodeWidthStr;

use crate::{
    app::States,
    event::Event,
    object::Object,
    widgets::{timeline::TimelineState, window_to_area},
};

use super::world_map::WorldMapState;

/// A widget that displays information about a selected object.
pub struct Information<'a> {
    pub state: &'a mut InformationState,
    pub world_map_state: &'a WorldMapState,
    pub timeline_state: &'a TimelineState,
}

/// State of a [`Information`] widget.
#[derive(Default)]
pub struct InformationState {
    /// Key-value pairs representing the object information to display in the
    /// table.
    table_entries: Vec<(String, String)>,
    /// The current state of the table widget.
    table_state: TableState,
    /// The inner rendering area of the widget.
    inner_area: Rect,
}

impl InformationState {
    fn scroll_up(&mut self) {
        *self.table_state.offset_mut() = self.table_state.offset().saturating_sub(1);
    }

    fn scroll_down(&mut self) {
        *self.table_state.offset_mut() = (self.table_state.offset() + 1).min(self.max_offset());
    }

    fn max_offset(&self) -> usize {
        self.table_entries
            .len()
            .saturating_sub(self.inner_area.height as usize)
    }
}

impl Widget for Information<'_> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let block = Self::block();
        self.state.inner_area = block.inner(area);
        block.render(area, buf);

        if let Some(object) = &self.world_map_state.selected_object {
            self.render_table(buf, object);
            self.render_scrollbar(area, buf);
        } else {
            Self::centered_paragraph(t!("no_object_selected").dark_gray())
                .render(self.state.inner_area, buf);
        }
    }
}

impl Information<'_> {
    fn block() -> Block<'static> {
        Block::new().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
    }

    fn table(&mut self, object: &Object) -> Table<'static> {
        self.update_table_entries(object);

        let (max_key_width, _max_value_width) = self
            .state
            .table_entries
            .iter()
            .map(|(key, value)| (key.width(), value.width()))
            .fold((0, 0), |acc, (key_width, value_width)| {
                (acc.0.max(key_width), acc.1.max(value_width))
            });

        let widths = [Constraint::Max(max_key_width as u16), Constraint::Fill(1)];
        let [_left, right] = Layout::horizontal(widths)
            .areas(self.state.inner_area)
            .map(|rect| rect.width);
        let right = right.saturating_sub(1) as usize;

        let rows = self
            .state
            .table_entries
            .iter()
            .enumerate()
            .map(|(row_index, (key, value))| {
                let value = truncate(value, right);
                let row_color = if row_index % 2 == 0 {
                    tailwind::SLATE.c950
                } else {
                    tailwind::SLATE.c900
                };
                Row::new([
                    Cell::from(Text::from(key.to_owned().bold())),
                    Cell::from(Text::from(value.to_string())),
                ])
                .bg(row_color)
                .height(1)
            });

        Table::new(rows, widths).row_highlight_style(Style::new().add_modifier(Modifier::REVERSED))
    }

    fn render_table(&mut self, buf: &mut Buffer, object: &Object) {
        StatefulWidget::render(
            self.table(object),
            self.state.inner_area,
            buf,
            &mut self.state.table_state,
        );
    }

    fn render_scrollbar(&self, area: Rect, buf: &mut Buffer) {
        let inner_area = Rect {
            height: area.height.saturating_sub(1),
            ..area
        };
        Scrollbar::default().render(
            inner_area,
            buf,
            &mut ScrollbarState::new(self.state.max_offset())
                .position(self.state.table_state.offset()),
        );
    }

    fn update_table_entries(&mut self, object: &Object) {
        const UNKNOWN: &str = "(Unknown)";

        let state = object.predict(&self.timeline_state.time()).unwrap();
        let (country, city) = state.position.country_city();
        let elements = object.elements();
        self.state.table_entries = vec![
            (
                t!("info.name").into(),
                object.name().unwrap_or(UNKNOWN).into(),
            ),
            (
                "COSPAR ID".into(),
                elements
                    .international_designator
                    .as_deref()
                    .unwrap_or(UNKNOWN)
                    .into(),
            ),
            (t!("info.norad_id").into(), elements.norad_id.to_string()),
            (
                t!("info.longitude").into(),
                format!("{:9.4}°", state.longitude()),
            ),
            (
                t!("info.latitude").into(),
                format!("{:9.4}°", state.latitude()),
            ),
            (
                t!("info.altitude").into(),
                format!("{:8.3} km", state.altitude()),
            ),
            (
                t!("info.speed").into(),
                format!("{:.2} km/s", state.speed()),
            ),
            (
                t!("info.period").into(),
                format!("{:.2} min", object.orbital_period().as_seconds_f64() / 60.0),
            ),
            (t!("info.location").into(), format!("{city}, {country}")),
            (
                t!("info.epoch").into(),
                object.epoch().format("%Y-%m-%d %H:%M:%S").to_string(),
            ),
            (
                t!("info.drag_term").into(),
                format!("{} 1/ER", elements.drag_term),
            ),
            (
                t!("info.inclination").into(),
                format!("{}°", elements.inclination),
            ),
            (
                t!("info.right_ascension").into(),
                format!("{}°", elements.right_ascension),
            ),
            (
                t!("info.eccentricity").into(),
                elements.eccentricity.to_string(),
            ),
            (
                t!("info.mean_anomaly").into(),
                format!("{}°", elements.mean_anomaly),
            ),
            (
                t!("info.mean_motion").into(),
                format!("{} 1/day", elements.mean_motion),
            ),
            (
                t!("info.rev_num").into(),
                elements.revolution_number.to_string(),
            ),
        ];
    }

    fn centered_paragraph<'a>(text: impl Into<Text<'a>>) -> Paragraph<'a> {
        Paragraph::new(text).centered().wrap(Wrap { trim: true })
    }
}

pub async fn handle_event(event: Event, states: &mut States) -> Result<()> {
    match event {
        Event::Mouse(event) => handle_mouse_event(event, states).await,
        _ => Ok(()),
    }
}

async fn handle_mouse_event(event: MouseEvent, states: &mut States) -> Result<()> {
    let state = &mut states.information_state;

    let global_mouse = Position::new(event.column, event.row);
    let Some(local_mouse) = window_to_area(global_mouse, state.inner_area) else {
        *state.table_state.selected_mut() = None;
        return Ok(());
    };

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Copy the clicked value to the clipboard.
            if let Some(index) = state.table_state.selected()
                && let Ok(mut clipboard) = Clipboard::new()
            {
                let value = &state.table_entries[index].1;
                clipboard
                    .set_text(value)
                    .expect("failed to copy to clipboard");
            }
        }
        MouseEventKind::ScrollUp => state.scroll_up(),
        MouseEventKind::ScrollDown => state.scroll_down(),
        _ => {}
    }
    // Highlight the hovered row.
    let row = local_mouse.y as usize + state.table_state.offset();
    let index = if row < state.table_entries.len() {
        Some(row)
    } else {
        None
    };
    state.table_state.select(index);

    Ok(())
}

/// Truncates a string to fit within the specified width, adding an ellipsis if
/// necessary.
fn truncate<'a>(str: &'a str, max_width: usize) -> Cow<'a, str> {
    const ELLIPSIS: &str = "…";
    debug_assert!(max_width >= ELLIPSIS.width());
    if str.width() > max_width {
        let end = str
            .char_indices()
            .map(|(i, _)| i)
            .nth(max_width.saturating_sub(ELLIPSIS.width()))
            .unwrap_or(str.len());
        Cow::Owned(format!("{}{}", &str[..end], ELLIPSIS))
    } else {
        Cow::Borrowed(str)
    }
}
