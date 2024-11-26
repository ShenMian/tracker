use std::cell::Cell;

use anyhow::Result;
use ratatui::{
    layout::Rect,
    style::{Color, Stylize},
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Map, MapResolution},
        Block,
    },
    Frame,
};

use crate::app::App;

use super::Components;

#[derive(Default)]
pub struct TrackMap {
    area: Cell<Rect>,
}

impl TrackMap {
    pub fn area(&self) -> Rect {
        self.area.get()
    }
}

impl Components for TrackMap {
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

                for object in &app.objects {
                    let line = if app.selected_object.is_none() {
                        "+".light_red() + format!(" {}", object.name()).white()
                    } else {
                        "+".red() + format!(" {}", object.name()).dark_gray()
                    };
                    let state = object.predict(0.0).unwrap();
                    ctx.print(state.position[0], state.position[1], line);
                }

                if let Some(selected_object_index) = app.selected_object {
                    let selected = &app.objects[selected_object_index];
                    for minutes in 10..24 * 60 {
                        let state = selected.predict(minutes as f64).unwrap();
                        ctx.print(state.position[0], state.position[1], ".".light_blue());
                    }

                    let state = selected.predict(0.0).unwrap();
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
