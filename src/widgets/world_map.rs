use anyhow::Result;
use chrono::{Duration, Utc};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Position, Rect},
    style::{Color, Stylize},
    widgets::{
        canvas::{Canvas, Line, Map, MapResolution},
        Block, StatefulWidget, Widget,
    },
};

use crate::app::App;

use super::satellites::SatellitesState;

pub struct WorldMap<'a> {
    pub satellites_state: &'a SatellitesState,
    pub satellit_symbol: String,
    pub trajectory_color: Color,
}

#[derive(Default)]
pub struct WorldMapState {
    pub selected_object: Option<usize>,
    pub hovered_object: Option<usize>,
    pub inner_area: Rect,
}

impl WorldMap<'_> {
    fn block(&self) -> Block<'static> {
        Block::bordered().title("World map".blue())
    }

    fn render_bottom_layer(&self, area: Rect, buf: &mut Buffer, state: &mut WorldMapState) {
        let bottom_layer = Canvas::default()
            .block(self.block())
            .paint(|ctx| {
                // Draw the world map
                ctx.draw(&Map {
                    color: Color::Gray,
                    resolution: MapResolution::High,
                });

                // Draw satellites
                for object in self.satellites_state.objects.iter() {
                    let line = if state.selected_object.is_none() {
                        self.satellit_symbol.clone().light_red()
                            + format!(" {}", object.name()).white()
                    } else {
                        self.satellit_symbol.clone().red()
                            + format!(" {}", object.name()).dark_gray()
                    };
                    let state = object.predict(Utc::now()).unwrap();
                    ctx.print(state.position[0], state.position[1], line);
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        bottom_layer.render(area, buf);
    }

    fn render_top_layer(&self, buf: &mut Buffer, state: &mut WorldMapState) {
        let top_layer = Canvas::default()
            .paint(|ctx| {
                if let Some(selected_object_index) = state.selected_object {
                    let selected = &self.satellites_state.objects[selected_object_index];
                    let state = selected.predict(Utc::now()).unwrap();

                    // Calculate future positions along the trajectory
                    let mut points = Vec::new();
                    for minutes in 1..selected.orbital_period().num_minutes() {
                        let time = Utc::now() + Duration::minutes(minutes);
                        let state = selected.predict(time).unwrap();
                        points.push((state.position[0], state.position[1]));
                    }

                    // Draw the lines between predicted points
                    for window in points.windows(2) {
                        let (x1, y1) = window[0];
                        let (x2, y2) = window[1];
                        // Handle trajectory crossing the international date line
                        if (x1 - x2).abs() >= 180.0 {
                            let x_edge = if x1 > 0.0 { 180.0 } else { -180.0 };
                            ctx.draw(&Line::new(x1, y1, x_edge, y2, self.trajectory_color));
                            ctx.draw(&Line::new(-x_edge, y1, x2, y2, self.trajectory_color));
                            continue;
                        }
                        if (y1 - y2).abs() >= 90.0 {
                            // TEMPSAT 1 (1512), CALSPHERE 4A (1520)
                            continue;
                        }
                        ctx.draw(&Line::new(x1, y1, x2, y2, self.trajectory_color));
                    }

                    // Highlight the selected satellite
                    ctx.print(
                        state.position[0],
                        state.position[1],
                        self.satellit_symbol.clone().light_green().slow_blink()
                            + format!(" {}", selected.name()).white(),
                    );
                } else if let Some(hovered_object_index) = state.hovered_object {
                    let hovered = &self.satellites_state.objects[hovered_object_index];
                    let state = hovered.predict(Utc::now()).unwrap();

                    // Highlight the hovered satellite
                    ctx.print(
                        state.position[0],
                        state.position[1],
                        self.satellit_symbol.clone().light_red().reversed()
                            + " ".into()
                            + hovered.name().clone().white().reversed(),
                    );
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        top_layer.render(state.inner_area, buf);
    }
}

impl StatefulWidget for WorldMap<'_> {
    type State = WorldMapState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.inner_area = area.inner(Margin::new(1, 1));

        self.render_bottom_layer(area, buf, state);
        self.render_top_layer(buf, state);
    }
}

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.world_map_state.inner_area;
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.world_map_state.hovered_object = None;
        return Ok(());
    }

    // Convert window coordinates to area coordinates
    let mouse = Position::new(event.column - inner_area.x, event.row - inner_area.y);

    if let MouseEventKind::Down(buttom) = event.kind {
        match buttom {
            MouseButton::Left => {
                app.world_map_state.selected_object = get_nearest_object(app, mouse.x, mouse.y);
            }
            MouseButton::Right => {
                app.world_map_state.selected_object = None;
            }
            _ => {}
        }
    }
    app.world_map_state.hovered_object = get_nearest_object(app, mouse.x, mouse.y);

    Ok(())
}

/// Get the index of the nearest object to the given area coordinates
fn get_nearest_object(app: &mut App, x: u16, y: u16) -> Option<usize> {
    app.satellites_state
        .objects
        .iter()
        .enumerate()
        .min_by_key(|(_, obj)| {
            let state = obj.predict(Utc::now()).unwrap();
            let (lon, lat) = area_to_lon_lat(x, y, app.world_map_state.inner_area);
            let dx = state.longitude() - lon;
            let dy = state.latitude() - lat;
            ((dx * dx + dy * dy) * 1000.0) as i32
        })
        .map(|(index, _)| index)
}

/// Convert area coordinates to lon/lat coordinates
fn area_to_lon_lat(x: u16, y: u16, area: Rect) -> (f64, f64) {
    debug_assert!(x < area.width && y < area.height);

    let normalized_x = (x + 1) as f64 / area.width as f64;
    let normalized_y = (y + 1) as f64 / area.height as f64;
    let lon = -180.0 + normalized_x * 360.0;
    let lat = 90.0 - normalized_y * 180.0;
    (lon, lat)
}

#[allow(dead_code)]
/// Convert lon/lat coordinates to area coordinates
fn lon_lat_to_area(lon: f64, lat: f64, area: Rect) -> (u16, u16) {
    debug_assert!((-180.0..=180.0).contains(&lon));
    debug_assert!((-90.0..=90.0).contains(&lat));

    let x = ((lon + 180.0) * area.width as f64 / 360.0) - 1.0;
    let y = ((90.0 - lat) * area.height as f64 / 180.0) - 1.0;
    (x.round() as u16, y.round() as u16)
}