use anyhow::Result;
use chrono::Utc;
use ratatui::{
    layout::{Constraint, Layout, Margin, Rect},
    style::{palette::tailwind, Style, Stylize},
    text::Text,
    widgets::{Block, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};
use reverse_geocoder::ReverseGeocoder;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::App;

use super::Component;

pub struct ObjectInformation {
    geocoder: ReverseGeocoder,
}

impl Default for ObjectInformation {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectInformation {
    pub fn new() -> Self {
        Self {
            geocoder: ReverseGeocoder::new(),
        }
    }
}

impl Component for ObjectInformation {
    fn render(&self, app: &App, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::bordered().title("Object information".blue());
        if let Some(index) = app.track_map.selected_object {
            let object = &app.satellites.objects[index];
            let state = object.predict(Utc::now()).unwrap();

            let result = self.geocoder.search((state.latitude(), state.longitude()));
            let city = result.record.name.clone();
            let country = isocountry::CountryCode::for_alpha2(&result.record.cc)
                .unwrap()
                .name();

            let items: Vec<(&str, String)> = vec![
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

            let rows = items.iter().enumerate().map(|(i, item)| {
                let color = match i % 2 {
                    0 => tailwind::SLATE.c950,
                    _ => tailwind::SLATE.c900,
                };

                let (property, value) = item;

                // Split the value into multiple lines if it's too long
                let value = split_string_by_width(value, right as usize - 1);

                // Calculate the height of the row based on the number of lines
                let height = (property.chars().filter(|c| *c == '\n').count() as u16)
                    .max(value.chars().filter(|c| *c == '\n').count() as u16)
                    + 1;

                Row::new([
                    Cell::from(Text::from(property.to_string())),
                    Cell::from(Text::from(value.to_string())),
                ])
                .style(Style::new().bg(color))
                .height(height)
            });

            let table = Table::new(rows, widths).block(block);

            frame.render_widget(table, area);
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

fn split_string_by_width(input: &str, max_width: usize) -> String {
    let mut lines = Vec::new();

    let mut line = String::new();
    let mut line_width = 0;
    for ch in input.chars() {
        let char_width = ch.width().unwrap();
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
