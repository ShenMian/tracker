use std::cell::Cell;

use anyhow::Result;
use chrono::Utc;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Position, Rect},
    style::{Color, Stylize},
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Map, MapResolution},
        Block,
    },
    Frame,
};

use crate::app::App;

use super::Component;

#[derive(Default)]
pub struct TrackMap {
    pub selected_object: Option<usize>,
    area: Cell<Rect>,
}

impl TrackMap {
    pub fn area(&self) -> Rect {
        self.area.get()
    }
}

impl Component for TrackMap {
    fn render(&self, app: &App, frame: &mut Frame, area: Rect) -> Result<()> {
        self.area.set(area);

        let canvas = Canvas::default()
            .block(Block::bordered().title("Satellite ground track".cyan()))
            .marker(Marker::Braille)
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: Color::Gray,
                    resolution: MapResolution::High,
                });

                for object in &app.satellites.objects {
                    let line = if self.selected_object.is_none() {
                        "+".light_red() + format!(" {}", object.name()).white()
                    } else {
                        "+".red() + format!(" {}", object.name()).dark_gray()
                    };
                    let state = object.predict(Utc::now()).unwrap();
                    ctx.print(state.position[0], state.position[1], line);
                }

                if let Some(selected_object_index) = self.selected_object {
                    let selected = &app.satellites.objects[selected_object_index];
                    for minutes in 1..selected.orbital_period().num_minutes() {
                        let time = Utc::now() + chrono::Duration::minutes(minutes);
                        let state = selected.predict(time).unwrap();
                        ctx.print(state.position[0], state.position[1], ".".light_blue());
                    }

                    let state = selected.predict(Utc::now()).unwrap();
                    ctx.print(
                        state.position[0],
                        state.position[1],
                        "+".light_green().rapid_blink() + format!(" {}", selected.name()).white(),
                    );
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        frame.render_widget(canvas, area);

        Ok(())
    }
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let area = app.track_map.area();
    if !area.contains(Position::new(event.column, event.row)) {
        return Ok(());
    }

    if let MouseEventKind::Down(buttom) = event.kind {
        match buttom {
            MouseButton::Left => {
                // Convert mouse coordinates to latitude and longitude
                let x = (event.column as f64 - area.left() as f64) / area.width as f64;
                let y = (event.row as f64 - area.top() as f64) / area.height as f64;
                let lon = -180.0 + x * 360.0;
                let lat = 90.0 - y * 180.0;

                // Find the nearest object
                if let Some((index, _)) =
                    app.satellites
                        .objects
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, obj)| {
                            let state = obj.predict(Utc::now()).unwrap();
                            let dx = state.longitude() - lon;
                            let dy = state.latitude() - lat;
                            ((dx * dx + dy * dy) * 1000.0) as i32
                        })
                {
                    app.track_map.selected_object = Some(index);
                }
            }
            MouseButton::Right => {
                app.track_map.selected_object = None;
            }
            _ => {}
        }
    }

    Ok(())
}
