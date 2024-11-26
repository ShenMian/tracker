use std::cell::RefCell;

use anyhow::Result;
use chrono::Utc;
use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{palette::tailwind, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, Cell, Paragraph, Row, Scrollbar, ScrollbarState, Table, TableState, Wrap},
    Frame,
};
use reverse_geocoder::ReverseGeocoder;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::App;

use super::Component;

pub struct ObjectInformation {
    pub table_state: RefCell<TableState>,
    pub height: std::cell::Cell<u16>,

    area: std::cell::Cell<Rect>,
    geocoder: ReverseGeocoder,
}

impl ObjectInformation {
    pub fn area(&self) -> Rect {
        self.area.get()
    }
}

impl Default for ObjectInformation {
    fn default() -> Self {
        Self {
            table_state: RefCell::new(TableState::default().with_selected(0)),
            height: Default::default(),
            area: Default::default(),
            geocoder: ReverseGeocoder::new(),
        }
    }
}

impl Component for ObjectInformation {
    fn render(&self, app: &App, frame: &mut Frame, area: Rect) -> Result<()> {
        self.area.set(area);

        let block = Block::bordered().title("Object information".blue());
        if let Some(index) = app.track_map.selected_object {
            let object = &app.satellites.objects[index];
            let state = object.predict(Utc::now()).unwrap();

            let result = self.geocoder.search((state.latitude(), state.longitude()));
            let city = result.record.name.clone();
            let country = isocountry::CountryCode::for_alpha2(&result.record.cc)
                .unwrap()
                .name();

            let mut items: Vec<(&str, String)> = vec![
                ("Name", object.name().clone()),
                ("COSPAR ID", object.cospar_id().clone()),
                ("NORAD ID", object.norad_id().to_string()),
                ("Longitude", format!("{:10.5}", state.longitude())),
                ("Latitude", format!("{:10.5}", state.latitude())),
                ("Altitude", format!("{:.5} km", state.altitude())),
                ("Speed", format!("{:.2} km/s", state.speed())),
                ("Location", format!("{}, {}", city, country)),
                ("Epoch", object.epoch().to_string()),
                (
                    "Period",
                    format!(
                        "{} hr {} min {} ({:.2} min)",
                        object.orbital_period().num_hours(),
                        object.orbital_period().num_minutes() % 60,
                        object.orbital_period().num_seconds() % 60,
                        object.orbital_period().num_seconds() as f64 / 60.0
                    ),
                ),
                ("Inc", object.inclination().to_string()),
                ("R.A.", object.right_ascension().to_string()),
                ("Ecc", object.eccentricity().to_string()),
                ("M. anomaly", object.mean_anomaly().to_string()),
                ("M. motion", object.mean_motion().to_string()),
                ("Rev. #", object.revolution_number().to_string()),
            ];

            let inner_area = area.inner(Margin::new(1, 1));

            let (max_key_width, _max_value_width) = items
                .iter()
                .map(|(key, value)| (key.width() as u16, value.width() as u16))
                .fold((0, 0), |acc, (key_width, value_width)| {
                    (acc.0.max(key_width), acc.1.max(value_width))
                });

            let widths = [Constraint::Max(max_key_width), Constraint::Fill(1)];
            let [_left, right] = Layout::horizontal(widths)
                .areas(inner_area)
                .map(|rect| rect.width);

            let mut total_height = 0;
            for (_, value) in &mut items {
                *value = split_string_by_width(value, right as usize - 1);
                total_height += calculate_row_height(value);
            }
            self.height.set(total_height);

            let rows = items.iter().enumerate().map(|(i, (key, value))| {
                let color = match i % 2 {
                    0 => tailwind::SLATE.c950,
                    _ => tailwind::SLATE.c900,
                };
                Row::new([
                    Cell::from(Text::from(key.bold())),
                    Cell::from(Text::from(value.to_string())),
                ])
                .style(Style::new().bg(color))
                .height(calculate_row_height(value))
            });

            let table = Table::new(rows, widths)
                .block(block)
                .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            frame.render_stateful_widget(table, area, &mut self.table_state.borrow_mut());

            let inner_area = area.inner(Margin::new(0, 1));
            frame.render_stateful_widget(
                Scrollbar::default(),
                inner_area,
                &mut ScrollbarState::new(
                    self.height.get().saturating_sub(inner_area.height) as usize
                )
                .position(self.table_state.borrow().offset()),
            );
        } else {
            let paragraph = Paragraph::new("No object selected".dark_gray())
                .block(block)
                .centered()
                .wrap(Wrap { trim: true });

            frame.render_widget(paragraph, area);
        }

        Ok(())
    }
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.object_information.area().inner(Margin::new(1, 1));
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.object_information.table_state.get_mut().select(None);
        return Ok(());
    }

    match event.kind {
        MouseEventKind::ScrollDown => {
            let max_offset = app
                .object_information
                .height
                .get()
                .saturating_sub(inner_area.height) as usize;
            *app.object_information.table_state.get_mut().offset_mut() =
                (*app.object_information.table_state.get_mut().offset_mut() + 1).min(max_offset);
        }
        MouseEventKind::ScrollUp => {
            *app.object_information.table_state.get_mut().offset_mut() = app
                .object_information
                .table_state
                .get_mut()
                .offset()
                .saturating_sub(1);
        }
        _ => {}
    }

    Ok(())
}

fn split_string_by_width(input: &str, max_width: usize) -> String {
    let mut lines = Vec::new();

    let mut line = String::new();
    let mut line_width = 0;
    for ch in input.chars() {
        let char_width = ch.width().unwrap_or(0);
        if line_width + char_width > max_width {
            lines.push(line.clone());
            line.clear();
            line_width = 0;
        }
        line.push(ch);
        line_width += char_width;
    }
    if !line.is_empty() {
        lines.push(line);
    }

    lines.join("\n")
}

fn calculate_row_height(text: &str) -> u16 {
    text.chars().filter(|c| *c == '\n').count() as u16 + 1
}

#[allow(dead_code)]
fn format_longitude(longitude: f64) -> String {
    if longitude >= 0.0 {
        format!("{:.5}°E", longitude)
    } else {
        format!("{:.5}°W", longitude.abs())
    }
}

#[allow(dead_code)]
fn format_latitude(latitude: f64) -> String {
    if latitude >= 0.0 {
        format!("{:.5}°N", latitude)
    } else {
        format!("{:.5}°S", latitude.abs())
    }
}
