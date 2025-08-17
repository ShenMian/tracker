use anyhow::{Context as _, Result};
use chrono::{DateTime, Duration, Local, Utc};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    style::Styled,
    widgets::{
        Block,
        canvas::{self, Canvas, Context, Map, MapResolution},
    },
};
use rust_i18n::t;

use crate::{app::App, config::WorldMapConfig, event::Event, utils::*};

use super::satellite_groups::SatelliteGroupsState;

/// A widget to display a world map with objects.
pub struct WorldMap<'a> {
    pub satellite_groups_state: &'a SatelliteGroupsState,
}

/// State of a [`WorldMap`] widget.
#[derive(Default)]
pub struct WorldMapState {
    /// Index of the selected object.
    pub selected_object_index: Option<usize>,
    /// Index of the hovered object.
    hovered_object_index: Option<usize>,

    /// Time offset from the current UTC time for time simulation.
    time_offset: Duration,
    /// Center longitude offset for horizontal map scrolling in degrees.
    lon_offset: f64,
    /// The time step to advance or rewind when scrolling time.
    time_delta: Duration,
    /// The amount of longitude (in degrees) to move the map when scrolling left
    /// or right.
    lon_delta: f64,

    /// Whether to follow the selected object by adjusting the map longitude.
    follow_object: bool,
    /// The smoothing factor for follow mode.
    follow_smoothing: f64,
    /// Whether to display the day-night terminator line.
    show_terminator: bool,

    map_color: Color,
    trajectory_color: Color,
    terminator_color: Color,

    /// The inner rendering area of the widget.
    inner_area: Rect,
}

impl WorldMapState {
    /// Creates a new `WorldMapState` with the given configuration.
    pub fn with_config(config: WorldMapConfig) -> Result<Self> {
        let map_color = config
            .map_color
            .parse()
            .context("invalid `world_map.map_color` configuration")?;
        let trajectory_color = config
            .trajectory_color
            .parse()
            .context("invalid `world_map.trajectory_color` configuration")?;
        let terminator_color = config
            .terminator_color
            .parse()
            .context("invalid `world_map.terminator_color` configuration")?;

        Ok(Self {
            follow_object: config.follow_object,
            follow_smoothing: config.follow_smoothing,
            show_terminator: config.show_terminator,
            time_delta: Duration::minutes(config.time_delta_min),
            lon_delta: config.lon_delta_deg,
            map_color,
            trajectory_color,
            terminator_color,
            ..Self::default()
        })
    }

    /// Returns the current simulation time.
    pub fn time(&self) -> DateTime<Utc> {
        Utc::now() + self.time_offset
    }

    /// Scrolls the map view to the left.
    fn scroll_map_left(&mut self) {
        self.lon_offset = wrap_longitude_deg(self.lon_offset - self.lon_delta);
    }

    /// Scrolls the map view to the right.
    fn scroll_map_right(&mut self) {
        self.lon_offset = wrap_longitude_deg(self.lon_offset + self.lon_delta);
    }

    /// Advances the simulation time.
    fn advance_time(&mut self) {
        self.time_offset += self.time_delta;
    }

    /// Rewinds the simulation time.
    fn rewind_time(&mut self) {
        self.time_offset -= self.time_delta;
    }
}

impl WorldMap<'_> {
    const OBJECT_SYMBOL: &'static str = "+";
    const SUBSOLAR_SYMBOL: &'static str = "*";
    const UNKNOWN_NAME: &'static str = "UNK";

    fn render_block(&self, area: Rect, buf: &mut Buffer, state: &mut WorldMapState) {
        let mut block = Block::bordered()
            .title(t!("wm.title").to_string().blue())
            .title_bottom(
                format!(
                    "{} ({:+} mins)",
                    state
                        .time()
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S"),
                    state.time_offset.num_minutes()
                )
                .white(),
            );

        // Show follow mode indicator if enabled
        if state.follow_object {
            let style = if state.selected_object_index.is_none() {
                Style::default().dark_gray()
            } else {
                Style::default().green().slow_blink()
            };
            block = block.title_bottom(
                Line::from(format!("({})", t!("wm.follow")).set_style(style)).right_aligned(),
            );
        }

        state.inner_area = block.inner(area);
        block.render(area, buf);
    }

    /// Renders the world map.
    fn render_map(&self, buf: &mut Buffer, state: &mut WorldMapState) {
        // Follow the longitude of the selected object
        if state.follow_object
            && let Some(index) = state.selected_object_index
        {
            let selected = &self.satellite_groups_state.objects[index];
            let object_state = selected.predict(&state.time()).unwrap();
            // state.lon_offset = object_state.longitude();

            state.lon_offset += wrap_longitude_deg(object_state.longitude() - state.lon_offset)
                * state.follow_smoothing;
            state.lon_offset = wrap_longitude_deg(state.lon_offset);
        }

        let x_min = state.lon_offset - 180.0;
        let x_max = state.lon_offset + 180.0;

        // Adjust the rendering order to prevent the labels on the left mapfrom being
        // covered by the right map
        let mut bounds_vec = Vec::new();
        if x_min < -180.0 {
            bounds_vec.push([x_min, x_max]); // Left side
            bounds_vec.push([x_max, x_max + 360.0]); // Right side
        } else if x_max > 180.0 {
            bounds_vec.push([-360.0 + x_min, x_min]); // Left side
            bounds_vec.push([x_min, x_max]); // Right side
        } else {
            bounds_vec.push([x_min, x_max]);
        }

        for bounds in &bounds_vec {
            self.render_bottom_layer(buf, *bounds, state);
        }
        for bounds in &bounds_vec {
            self.render_top_layer(buf, *bounds, state);
        }
    }

    /// Renders the bottom layer of the world map, including the map and all
    /// objects.
    fn render_bottom_layer(&self, buf: &mut Buffer, x_bounds: [f64; 2], state: &mut WorldMapState) {
        Canvas::default()
            .x_bounds(x_bounds)
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: state.map_color,
                    resolution: MapResolution::High,
                });
                ctx.layer();
                if state.show_terminator {
                    self.draw_terminator(ctx, state);
                }
                self.draw_objects(ctx, state);
            })
            .render(state.inner_area, buf);
    }

    /// Renders the top layer of the world map, including object highlights and
    /// trajectories.
    fn render_top_layer(&self, buf: &mut Buffer, x_bounds: [f64; 2], state: &mut WorldMapState) {
        Canvas::default()
            .x_bounds(x_bounds)
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                self.draw_object_highlight(ctx, state);
            })
            .render(state.inner_area, buf);
    }

    /// Draws the day-night terminator and subsolar point.
    fn draw_terminator(&self, ctx: &mut Context, state: &WorldMapState) {
        // Draw the terminator line
        self.draw_lines(
            ctx,
            calculate_terminator(&state.time()),
            state.terminator_color,
        );

        // Mark the subsolar point
        let (sub_lon, sub_lat) = subsolar_point(&state.time());
        ctx.print(
            sub_lon.to_degrees(),
            sub_lat.to_degrees(),
            Self::SUBSOLAR_SYMBOL.yellow().bold(),
        );
    }

    /// Draws all objects and their labels.
    fn draw_objects(&self, ctx: &mut Context, state: &WorldMapState) {
        for object in self.satellite_groups_state.objects.iter() {
            let object_name = object.name().unwrap_or(Self::UNKNOWN_NAME);
            let text = if state.selected_object_index.is_none() {
                Self::OBJECT_SYMBOL.light_red() + format!(" {object_name}").white()
            } else {
                Self::OBJECT_SYMBOL.red() + format!(" {object_name}").dark_gray()
            };
            let object_state = object.predict(&state.time()).unwrap();
            ctx.print(
                object_state.position.longitude,
                object_state.position.latitude,
                text,
            );
        }
    }

    /// Draws the highlight and trajectory for the selected or hovered object.
    fn draw_object_highlight(&self, ctx: &mut Context, state: &WorldMapState) {
        if let Some(selected_object_index) = state.selected_object_index {
            let selected = &self.satellite_groups_state.objects[selected_object_index];

            // Draw the trajectory
            self.draw_lines(
                ctx,
                calculate_trajectory(selected, &state.time()),
                state.trajectory_color,
            );

            // Highlight the selected object
            let object_name = selected.name().unwrap_or(Self::UNKNOWN_NAME);
            let text =
                Self::OBJECT_SYMBOL.light_green().slow_blink() + format!(" {object_name}").white();
            let object_state = selected.predict(&state.time()).unwrap();
            ctx.print(
                object_state.position.longitude,
                object_state.position.latitude,
                text,
            );
        } else if let Some(hovered_object_index) = state.hovered_object_index {
            let hovered = &self.satellite_groups_state.objects[hovered_object_index];

            // Highlight the hovered object
            let object_name = hovered.name().unwrap_or(Self::UNKNOWN_NAME);
            let text = Self::OBJECT_SYMBOL.light_red().reversed()
                + " ".into()
                + object_name.to_string().white().reversed();
            let object_state = hovered.predict(&state.time()).unwrap();
            ctx.print(
                object_state.position.longitude,
                object_state.position.latitude,
                text,
            );
        }
    }

    /// Draws lines between points.
    fn draw_lines(&self, ctx: &mut Context, points: Vec<(f64, f64)>, color: Color) {
        for window in points.windows(2) {
            self.draw_line(ctx, window[0], window[1], color);
        }
    }

    /// Draws a line between two points.
    fn draw_line(
        &self,
        ctx: &mut Context,
        (x1, y1): (f64, f64),
        (x2, y2): (f64, f64),
        color: Color,
    ) {
        // Handle trajectory crossing the international date line
        if (x1 - x2).abs() >= 180.0 {
            let x_edge = if x1 > 0.0 { 180.0 } else { -180.0 };
            let y_midpoint = (y1 + y2) / 2.0;
            ctx.draw(&canvas::Line::new(x1, y1, x_edge, y_midpoint, color));
            ctx.draw(&canvas::Line::new(-x_edge, y_midpoint, x2, y2, color));
            return;
        }
        ctx.draw(&canvas::Line::new(x1, y1, x2, y2, color));
    }
}

impl StatefulWidget for WorldMap<'_> {
    type State = WorldMapState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_block(area, buf, state);
        self.render_map(buf, state);
    }
}

pub async fn handle_event(event: Event, app: &mut App) -> Result<()> {
    match event {
        Event::Key(event) => handle_key_event(event, app).await,
        Event::Mouse(event) => handle_mouse_event(event, app).await,
        _ => Ok(()),
    }
}

async fn handle_key_event(event: KeyEvent, app: &mut App) -> Result<()> {
    match event.code {
        KeyCode::Char('[') => app.world_map_state.scroll_map_left(),
        KeyCode::Char(']') => app.world_map_state.scroll_map_right(),
        KeyCode::Char('f') => {
            app.world_map_state.follow_object = !app.world_map_state.follow_object;
        }
        KeyCode::Char('r') => app.world_map_state.time_offset = chrono::Duration::zero(),
        KeyCode::Char('t') => {
            app.world_map_state.show_terminator = !app.world_map_state.show_terminator;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_mouse_event(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.world_map_state.inner_area;
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.world_map_state.hovered_object_index = None;
        return Ok(());
    }

    // Convert window coordinates to area coordinates
    let mouse = Position::new(event.column - inner_area.x, event.row - inner_area.y);

    let nearest_object_index = get_nearest_object_index(app, mouse, inner_area);
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            app.world_map_state.selected_object_index = nearest_object_index
        }
        MouseEventKind::Down(MouseButton::Right) => {
            app.world_map_state.selected_object_index = None;
        }
        MouseEventKind::ScrollUp => {
            if event.modifiers == KeyModifiers::SHIFT {
                app.world_map_state.scroll_map_left();
            } else {
                app.world_map_state.rewind_time();
            }
        }
        MouseEventKind::ScrollDown => {
            if event.modifiers == KeyModifiers::SHIFT {
                app.world_map_state.scroll_map_right();
            } else {
                app.world_map_state.advance_time();
            }
        }
        _ => {}
    }
    app.world_map_state.hovered_object_index = nearest_object_index;

    Ok(())
}

/// Get the index of the nearest object to the given area position
fn get_nearest_object_index(app: &mut App, position: Position, inner_area: Rect) -> Option<usize> {
    app.satellite_groups_state
        .objects
        .iter()
        .enumerate()
        .min_by_key(|(_, obj)| {
            let state = obj.predict(&app.world_map_state.time()).unwrap();
            // Convert to area position
            let (x, y) = lon_lat_to_area(
                wrap_longitude_deg(state.longitude() - app.world_map_state.lon_offset),
                state.latitude(),
                inner_area,
            );
            (x as i32 - position.x as i32).abs() + (y as i32 - position.y as i32).abs()
        })
        .map(|(index, _)| index)
}

#[expect(dead_code)]
/// Converts area coordinates to lon/lat coordinates.
fn area_to_lon_lat(x: u16, y: u16, area: Rect) -> (f64, f64) {
    debug_assert!(x < area.width && y < area.height);
    debug_assert!(area.width > 0 && area.height > 0);

    let normalized_x = x as f64 / area.width as f64;
    let normalized_y = y as f64 / area.height as f64;
    let lon = -180.0 + normalized_x * 360.0;
    let lat = 90.0 - normalized_y * 180.0;
    (lon, lat)
}

/// Converts lon/lat coordinates to area coordinates.
fn lon_lat_to_area(lon: f64, lat: f64, area: Rect) -> (u16, u16) {
    debug_assert!((-180.0..=180.0).contains(&lon));
    debug_assert!((-90.0..=90.0).contains(&lat));

    let x = ((lon + 180.0) * area.width as f64 / 360.0) - 1.0;
    let y = ((90.0 - lat) * area.height as f64 / 180.0) - 1.0;
    (x.round() as u16, y.round() as u16)
}
