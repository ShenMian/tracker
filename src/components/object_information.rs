use anyhow::Result;
use chrono::Utc;
use ratatui::{
    layout::Rect,
    style::Stylize,
    text::Text,
    widgets::{Block, Paragraph, Wrap},
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
        let paragraph = if let Some(index) = app.track_map.selected_object {
            let object = &app.satellites.objects[index];
            let state = object.predict(Utc::now()).unwrap();

            let result = self.geocoder.search((state.latitude(), state.longitude()));
            let city = result.record.name.clone();
            let country = isocountry::CountryCode::for_alpha2(&result.record.cc)
                .unwrap()
                .name();

            let string = format!(
                r#"Name: {}
                ID  : {}

                LAT: {:8.2}°
                LON: {:8.2}°
                ALT: {:8.2} km

                Speed   : {:.2} km/s
                Location: {}, {}

                Orbital period: {} hr {} min"#,
                object.name(),
                object.id(),
                state.latitude(),
                state.longitude(),
                state.altitude(),
                state.speed(),
                city,
                country,
                object.orbital_period().num_hours(),
                object.orbital_period().num_minutes() % 60
            );

            let text = Text::raw(string);
            Paragraph::new(text)
                .block(Block::bordered().title("Object information".cyan()))
                .wrap(Wrap { trim: true })
        } else {
            Paragraph::new("No object selected")
                .block(Block::bordered().title("Object information".cyan()))
                .centered()
                .wrap(Wrap { trim: true })
        };

        frame.render_widget(paragraph, area);

        Ok(())
    }
}
