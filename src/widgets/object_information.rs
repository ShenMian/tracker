use anyhow::Result;
use arboard::Clipboard;
use chrono::Utc;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{palette::tailwind, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, Cell, Paragraph, Row, Scrollbar, ScrollbarState, StatefulWidget, Table, TableState,
        Widget, Wrap,
    },
};
use reverse_geocoder::ReverseGeocoder;
use unicode_width::UnicodeWidthStr;

use crate::app::App;

use super::{satellites::SatellitesState, world_map::WorldMapState};

pub struct ObjectInformation<'a> {
    pub satellites_state: &'a SatellitesState,
    pub world_map_state: &'a WorldMapState,
}

pub struct ObjectInformationState {
    pub items: Vec<(&'static str, String)>,
    pub table_state: TableState,
    pub inner_area: Rect,
    geocoder: ReverseGeocoder,
}

impl ObjectInformationState {
    pub fn scroll_up(&mut self) {
        *self.table_state.offset_mut() = self.table_state.offset().saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        let max_offset = self
            .items
            .len()
            .saturating_sub(self.inner_area.height as usize);
        *self.table_state.offset_mut() = (self.table_state.offset() + 1).min(max_offset);
    }
}

impl Default for ObjectInformationState {
    fn default() -> Self {
        Self {
            items: Default::default(),
            table_state: Default::default(),
            inner_area: Default::default(),
            geocoder: ReverseGeocoder::new(),
        }
    }
}

impl ObjectInformation<'_> {
    fn render_block(&self, area: Rect, buf: &mut Buffer, state: &mut ObjectInformationState) {
        let block = Block::bordered().title("Object information".blue());
        state.inner_area = block.inner(area);
        block.render(area, buf);
    }

    fn render_table(&self, buf: &mut Buffer, state: &mut ObjectInformationState, index: usize) {
        const UNKNOWN_NAME: &str = "Unknown";

        let object = &self.satellites_state.objects[index];
        let object_state = object.predict(Utc::now()).unwrap();

        let result = state
            .geocoder
            .search((object_state.latitude(), object_state.longitude()));
        let city = result.record.name.clone();
        let country = isocountry::CountryCode::for_alpha2(&result.record.cc)
            .unwrap()
            .name();

        state.items = Vec::from([
            (
                "Name",
                object.name().unwrap_or(&UNKNOWN_NAME.to_string()).clone(),
            ),
            (
                "COSPAR ID",
                object
                    .cospar_id()
                    .unwrap_or(&UNKNOWN_NAME.to_string())
                    .clone(),
            ),
            ("NORAD ID", object.norad_id().to_string()),
            ("Longitude", format!("{:9.4}°", object_state.longitude())),
            ("Latitude", format!("{:9.4}°", object_state.latitude())),
            ("Altitude", format!("{:.3} km", object_state.altitude())),
            ("Speed", format!("{:.2} km/s", object_state.speed())),
            (
                "Period",
                format!(
                    "{:.2} min",
                    object.orbital_period().num_seconds() as f64 / 60.0
                ),
            ),
            ("Location", format!("{}, {}", city, country)),
            (
                "Epoch",
                object.epoch().format("%Y-%m-%d %H:%M:%S").to_string(),
            ),
            ("Drag term", format!("{} 1/ER", object.drag_term())),
            ("Inc", format!("{}°", object.inclination())),
            ("Right asc.", format!("{}°", object.right_ascension())),
            ("Ecc", object.eccentricity().to_string()),
            ("M. anomaly", format!("{}°", object.mean_anomaly())),
            ("M. motion", format!("{} 1/day", object.mean_motion())),
            ("Rev. #", object.revolution_number().to_string()),
        ]);

        let (max_key_width, _max_value_width) = state
            .items
            .iter()
            .map(|(key, value)| (key.width(), value.width()))
            .fold((0, 0), |acc, (key_width, value_width)| {
                (acc.0.max(key_width), acc.1.max(value_width))
            });

        let widths = [Constraint::Max(max_key_width as u16), Constraint::Fill(1)];
        let [_left, right] = Layout::horizontal(widths)
            .areas(state.inner_area)
            .map(|rect| rect.width);
        let right = right.saturating_sub(1) as usize;

        let rows = state.items.iter().enumerate().map(|(i, (key, value))| {
            let color = match i % 2 {
                0 => tailwind::SLATE.c950,
                _ => tailwind::SLATE.c900,
            };
            let value = if value.width() > right {
                let etc = "…";
                let end = value
                    .char_indices()
                    .map(|(i, _)| i)
                    .nth(right.saturating_sub(etc.width()))
                    .unwrap();
                value[..end].to_string() + etc
            } else {
                value.to_string()
            };
            Row::new([
                Cell::from(Text::from(key.bold())),
                Cell::from(Text::from(value)),
            ])
            .style(Style::new().bg(color))
            .height(1)
        });

        let table = Table::new(rows, widths)
            .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        StatefulWidget::render(table, state.inner_area, buf, &mut state.table_state);
    }

    fn render_scrollbar(&self, area: Rect, buf: &mut Buffer, state: &mut ObjectInformationState) {
        let inner_area = area.inner(Margin::new(0, 1));
        let mut scrollbar_state =
            ScrollbarState::new(state.items.len().saturating_sub(inner_area.height as usize))
                .position(state.table_state.offset());
        Scrollbar::default().render(inner_area, buf, &mut scrollbar_state);
    }

    fn render_no_object_selected(&self, buf: &mut Buffer, state: &mut ObjectInformationState) {
        let paragraph = Paragraph::new("No object selected".dark_gray())
            .centered()
            .wrap(Wrap { trim: true });

        paragraph.render(state.inner_area, buf);
    }
}

impl StatefulWidget for ObjectInformation<'_> {
    type State = ObjectInformationState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_block(area, buf, state);
        if let Some(index) = self.world_map_state.selected_object {
            self.render_table(buf, state, index);
            self.render_scrollbar(area, buf, state);
        } else {
            self.render_no_object_selected(buf, state);
        }
    }
}

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.object_information_state.inner_area;
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.object_information_state.table_state.select(None);
        return Ok(());
    }

    // Convert window coordinates to area coordinates
    let mouse = Position::new(event.column - inner_area.x, event.row - inner_area.y);

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            // Copy the clicked value to the clipboard.
            if let Some(index) = app.object_information_state.table_state.selected() {
                if let Ok(mut clipboard) = Clipboard::new() {
                    let value = app.object_information_state.items[index].1.clone();
                    clipboard.set_text(value).unwrap();
                }
            }
        }
        MouseEventKind::ScrollUp => app.object_information_state.scroll_up(),
        MouseEventKind::ScrollDown => app.object_information_state.scroll_down(),
        _ => {}
    }
    // Highlight the hovered row.
    let row = mouse.y as usize + app.object_information_state.table_state.offset();
    let index = if row < app.object_information_state.items.len() {
        Some(row)
    } else {
        None
    };
    app.object_information_state.table_state.select(index);

    Ok(())
}
