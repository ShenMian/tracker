use anyhow::Result;
use chrono::Utc;
use ratatui::{
    layout::{Margin, Rect},
    style::{palette::tailwind, Style, Stylize},
    text::Text,
    widgets::{Block, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};
use reverse_geocoder::ReverseGeocoder;

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
                ("ID", object.id().clone()),
                ("Latitude", format!("{:.2}°", state.latitude())),
                ("Longitude", format!("{:.2}°", state.longitude())),
                ("Altitude", format!("{:.2} km", state.altitude())),
                ("Speed", format!("{:.2} km/s", state.speed())),
                ("Location", format!("{}, {}", city, country)),
                (
                    "Orbital period",
                    format!(
                        "{} hr {} min",
                        object.orbital_period().num_hours(),
                        object.orbital_period().num_minutes() % 60
                    ),
                ),
            ];

            let inner_area = area.inner(Margin::new(1, 1));

            let rows = items.iter().enumerate().map(|(i, item)| {
                let color = match i % 2 {
                    0 => tailwind::SLATE.c950,
                    _ => tailwind::SLATE.c900,
                };
                let (property, value) = item;

                // Split the value into multiple lines if it's too long
                let value = value
                    .as_bytes()
                    .chunks(((inner_area.width - 1) / 2) as usize)
                    .map(String::from_utf8_lossy)
                    .map(|s| s.trim_start().to_string())
                    .collect::<Vec<_>>()
                    .join("\n");

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

            let table = Table::default().block(block).rows(rows);

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
