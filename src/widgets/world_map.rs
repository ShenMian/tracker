use anyhow::Result;
use chrono::{Duration, Utc};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    widgets::{
        Block,
        canvas::{Canvas, Context, Line, Map, MapResolution},
    },
};

use crate::{app::App, object::Object};

use super::satellite_groups::SatelliteGroupsState;

/// A widget to display a world map with satellites.
pub struct WorldMap<'a> {
    pub satellite_groups_state: &'a SatelliteGroupsState,
}

/// State of a [`WorldMapState`] widget
#[derive(Default)]
pub struct WorldMapState {
    pub selected_object_index: Option<usize>,
    pub hovered_object_index: Option<usize>,
    pub inner_area: Rect,
}

impl WorldMap<'_> {
    const MAP_COLOR: Color = Color::Gray;
    const TRAJECTORY_COLOR: Color = Color::LightBlue;
    const SATELLITE_SYMBOL: &'static str = "+";
    const UNKNOWN_NAME: &'static str = "UNK";

    fn render_block(&self, area: Rect, buf: &mut Buffer, state: &mut WorldMapState) {
        let block = Block::bordered().title("World map".blue());
        state.inner_area = block.inner(area);
        block.render(area, buf);
    }

    /// Render world map and satellites
    fn render_bottom_layer(&self, buf: &mut Buffer, state: &mut WorldMapState) {
        let bottom_layer = Canvas::default()
            .paint(|ctx| {
                // Draw the world map
                ctx.draw(&Map {
                    color: Self::MAP_COLOR,
                    resolution: MapResolution::High,
                });

                // Draw satellites
                for object in self.satellite_groups_state.objects.iter() {
                    let object_name = object.name().unwrap_or(Self::UNKNOWN_NAME);
                    let text = if state.selected_object_index.is_none() {
                        Self::SATELLITE_SYMBOL.light_red() + format!(" {object_name}").white()
                    } else {
                        Self::SATELLITE_SYMBOL.red() + format!(" {object_name}").dark_gray()
                    };
                    let state = object.predict(Utc::now()).unwrap();
                    ctx.print(state.position[0], state.position[1], text);
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        bottom_layer.render(state.inner_area, buf);
    }

    /// Render selected satellite and its trajectory
    fn render_top_layer(&self, buf: &mut Buffer, state: &mut WorldMapState) {
        let top_layer = Canvas::default()
            .paint(|ctx| {
                if let Some(selected_object_index) = state.selected_object_index {
                    let selected = &self.satellite_groups_state.objects[selected_object_index];

                    self.render_trajectory(ctx, selected);

                    // Highlight the selected satellite
                    let object_name = selected.name().unwrap_or(Self::UNKNOWN_NAME);
                    let text = Self::SATELLITE_SYMBOL.light_green().slow_blink()
                        + format!(" {object_name}").white();
                    let state = selected.predict(Utc::now()).unwrap();
                    ctx.print(state.position[0], state.position[1], text);
                } else if let Some(hovered_object_index) = state.hovered_object_index {
                    let hovered = &self.satellite_groups_state.objects[hovered_object_index];

                    // Highlight the hovered satellite
                    let object_name = hovered.name().unwrap_or(Self::UNKNOWN_NAME);
                    let text = Self::SATELLITE_SYMBOL.light_red().reversed()
                        + " ".into()
                        + object_name.to_string().white().reversed();
                    let state = hovered.predict(Utc::now()).unwrap();
                    ctx.print(state.position[0], state.position[1], text);
                }
            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);

        top_layer.render(state.inner_area, buf);
    }

    /// Render the trajectory of the object
    fn render_trajectory(&self, ctx: &mut Context, object: &Object) {
        let trajectory = self.calculate_trajectory(object);

        // Draw the lines between predicted points
        for window in trajectory.windows(2) {
            self.render_line(ctx, window[0], window[1]);
        }
    }

    /// Calculate the trajectory of the object
    fn calculate_trajectory(&self, object: &Object) -> Vec<(f64, f64)> {
        // Calculate future positions along the trajectory
        let mut points = Vec::new();
        for minutes in 1..object.orbital_period().num_minutes() {
            let time = Utc::now() + Duration::minutes(minutes);
            let state = object.predict(time).unwrap();
            points.push((state.position[0], state.position[1]));
        }
        points
    }

    fn render_line(&self, ctx: &mut Context, (x1, y1): (f64, f64), (x2, y2): (f64, f64)) {
        // Handle trajectory crossing the international date line
        if (x1 - x2).abs() >= 180.0 {
            let x_edge = if x1 > 0.0 { 180.0 } else { -180.0 };
            let y_midpoint = (y1 + y2) / 2.0;
            ctx.draw(&Line::new(
                x1,
                y1,
                x_edge,
                y_midpoint,
                Self::TRAJECTORY_COLOR,
            ));
            ctx.draw(&Line::new(
                -x_edge,
                y_midpoint,
                x2,
                y2,
                Self::TRAJECTORY_COLOR,
            ));
            return;
        }
        ctx.draw(&Line::new(x1, y1, x2, y2, Self::TRAJECTORY_COLOR));
    }
}

impl StatefulWidget for WorldMap<'_> {
    type State = WorldMapState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_block(area, buf, state);
        self.render_bottom_layer(buf, state);
        self.render_top_layer(buf, state);
    }
}

pub async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.world_map_state.inner_area;
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.world_map_state.hovered_object_index = None;
        return Ok(());
    }

    // Convert window coordinates to area coordinates
    let mouse = Position::new(event.column - inner_area.x, event.row - inner_area.y);

    let (lon, lat) = area_to_lon_lat(mouse.x, mouse.y, app.world_map_state.inner_area);
    let nearest_object_index =
        app.satellite_groups_state
            .get_nearest_object_index(Utc::now(), lon, lat);
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            app.world_map_state.selected_object_index = nearest_object_index
        }
        MouseEventKind::Down(MouseButton::Right) => {
            app.world_map_state.selected_object_index = None
        }
        _ => {}
    }
    app.world_map_state.hovered_object_index = nearest_object_index;

    Ok(())
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

#[expect(dead_code)]
/// Convert lon/lat coordinates to area coordinates
fn lon_lat_to_area(lon: f64, lat: f64, area: Rect) -> (u16, u16) {
    debug_assert!((-180.0..=180.0).contains(&lon));
    debug_assert!((-90.0..=90.0).contains(&lat));

    let x = ((lon + 180.0) * area.width as f64 / 360.0) - 1.0;
    let y = ((90.0 - lat) * area.height as f64 / 180.0) - 1.0;
    (x.round() as u16, y.round() as u16)
}
