use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    style::Styled,
    widgets::{
        Block,
        canvas::{self, Canvas, Context, Map, MapResolution},
    },
};
use rust_i18n::t;

use crate::{
    app::App,
    config::WorldMapConfig,
    event::Event,
    object::Object,
    utils::*,
    widgets::{sky::SkyState, timeline::TimelineState, window_to_area},
};

use super::satellite_groups::SatelliteGroupsState;

/// A widget that displays a world map with objects.
pub struct WorldMap<'a> {
    pub state: &'a mut WorldMapState,
    pub satellite_groups_state: &'a SatelliteGroupsState,
    pub sky_state: &'a SkyState,
    pub timeline_state: &'a TimelineState,
}

/// State of a [`WorldMap`] widget.
#[derive(Default)]
pub struct WorldMapState {
    /// Index of the selected object.
    pub selected_object_index: Option<usize>,
    /// Index of the hovered object.
    hovered_object_index: Option<usize>,

    /// Center longitude offset for horizontal map scrolling in degrees.
    lon_offset: f64,
    /// The amount of longitude (in degrees) to move the map when scrolling left
    /// or right.
    lon_delta: f64,

    /// Whether to follow the selected object by adjusting the map longitude.
    follow_object: bool,
    /// The smoothing factor for follow mode.
    follow_smoothing: f64,
    /// Whether to display the day-night terminator line.
    show_terminator: bool,
    /// Whether to display the visibility area.
    show_visibility_area: bool,

    map_color: Color,
    trajectory_color: Color,
    terminator_color: Color,
    visibility_area_color: Color,

    /// The inner rendering area of the widget.
    inner_area: Rect,
}

impl WorldMapState {
    /// Creates a new `WorldMapState` with the given configuration.
    pub fn with_config(config: WorldMapConfig) -> Self {
        Self {
            follow_object: config.follow_object,
            follow_smoothing: config.follow_smoothing,
            show_terminator: config.show_terminator,
            show_visibility_area: config.show_visibility_area,
            lon_delta: config.lon_delta_deg,
            map_color: config.map_color,
            trajectory_color: config.trajectory_color,
            terminator_color: config.terminator_color,
            visibility_area_color: config.visibility_area_color,
            ..Self::default()
        }
    }

    /// Returns a reference to the selected object.
    pub fn selected_object<'a>(
        &self,
        satellite_groups_state: &'a SatelliteGroupsState,
    ) -> Option<&'a Object> {
        satellite_groups_state
            .objects
            .get(self.selected_object_index?)
    }

    /// Returns a reference to the hovered object.
    fn hovered_object<'a>(
        &self,
        satellite_groups_state: &'a SatelliteGroupsState,
    ) -> Option<&'a Object> {
        satellite_groups_state
            .objects
            .get(self.hovered_object_index?)
    }

    /// Scrolls the map view to the left.
    fn scroll_map_left(&mut self) {
        self.lon_offset = wrap_longitude_deg(self.lon_offset - self.lon_delta);
    }

    /// Scrolls the map view to the right.
    fn scroll_map_right(&mut self) {
        self.lon_offset = wrap_longitude_deg(self.lon_offset + self.lon_delta);
    }
}

impl WorldMap<'_> {
    const OBJECT_SYMBOL: &'static str = "+";
    const SUBSOLAR_SYMBOL: &'static str = "*";
    const UNKNOWN_NAME: &'static str = "UNK";

    pub fn render(mut self, area: Rect, buf: &mut Buffer) {
        let block = self.block();
        self.state.inner_area = block.inner(area);
        block.render(area, buf);

        self.render_map(buf);
    }

    fn block(&self) -> Block<'static> {
        let mut block = Block::bordered().title(t!("map.title").to_string().blue());

        // Show follow mode indicator if enabled
        if self.state.follow_object {
            let style = if self.state.selected_object_index.is_none() {
                Style::new().dark_gray()
            } else {
                Style::new().green().slow_blink()
            };
            block = block.title_bottom(
                Line::from(format!("({})", t!("map.follow")).set_style(style)).right_aligned(),
            );
        }

        block
    }

    /// Renders the world map.
    fn render_map(&mut self, buf: &mut Buffer) {
        // Follow the longitude of the selected object
        if self.state.follow_object
            && let Some(selected) = self.state.selected_object(self.satellite_groups_state)
        {
            let object_state = selected.predict(&self.timeline_state.time()).unwrap();

            self.state.lon_offset +=
                wrap_longitude_deg(object_state.longitude() - self.state.lon_offset)
                    * self.state.follow_smoothing;
            self.state.lon_offset = wrap_longitude_deg(self.state.lon_offset);
        }

        let x_min = self.state.lon_offset - 180.0;
        let x_max = self.state.lon_offset + 180.0;

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
            self.render_bottom_layer(buf, *bounds);
        }
        for bounds in &bounds_vec {
            self.render_top_layer(buf, *bounds);
        }
    }

    /// Renders the bottom layer of the world map, including the map and all
    /// objects.
    fn render_bottom_layer(&self, buf: &mut Buffer, x_bounds: [f64; 2]) {
        Canvas::default()
            .x_bounds(x_bounds)
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: self.state.map_color,
                    resolution: MapResolution::High,
                });
                ctx.layer();
                if self.state.show_terminator {
                    self.draw_terminator(ctx);
                }
                self.draw_objects(ctx);
            })
            .render(self.state.inner_area, buf);
    }

    /// Renders the top layer of the world map, including object highlights and
    /// trajectories.
    fn render_top_layer(&self, buf: &mut Buffer, x_bounds: [f64; 2]) {
        Canvas::default()
            .x_bounds(x_bounds)
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                self.draw_object_highlight(ctx);
                if self.state.show_visibility_area {
                    self.draw_visibility_area(ctx);
                }
                self.draw_ground_station(ctx);
            })
            .render(self.state.inner_area, buf);
    }

    /// Draws the day-night terminator and subsolar point.
    fn draw_terminator(&self, ctx: &mut Context) {
        // Draw the terminator line
        Self::draw_lines(
            ctx,
            calculate_terminator(&self.timeline_state.time()),
            self.state.terminator_color,
        );

        // Mark the subsolar point
        let (sub_lon, sub_lat) = subsolar_point(&self.timeline_state.time());
        ctx.print(
            sub_lon.to_degrees(),
            sub_lat.to_degrees(),
            Self::SUBSOLAR_SYMBOL.yellow().bold(),
        );
    }

    /// Draws all objects and their labels.
    fn draw_objects(&self, ctx: &mut Context) {
        for object in self.satellite_groups_state.objects.iter() {
            let object_name = object.name().unwrap_or(Self::UNKNOWN_NAME);
            let text = if self.state.selected_object_index.is_none() {
                Self::OBJECT_SYMBOL.light_red() + format!(" {object_name}").white()
            } else {
                Self::OBJECT_SYMBOL.red() + format!(" {object_name}").dark_gray()
            };
            let object_state = object.predict(&self.timeline_state.time()).unwrap();
            ctx.print(object_state.longitude(), object_state.latitude(), text);
        }
    }

    /// Draws the highlight and trajectory for the selected or hovered object.
    fn draw_object_highlight(&self, ctx: &mut Context) {
        if let Some(selected) = self.state.selected_object(self.satellite_groups_state) {
            // Draw the trajectory
            Self::draw_lines(
                ctx,
                calculate_ground_track(selected, &self.timeline_state.time()),
                self.state.trajectory_color,
            );

            // Highlight the selected object
            let object_name = selected.name().unwrap_or(Self::UNKNOWN_NAME);
            let text =
                Self::OBJECT_SYMBOL.light_green().slow_blink() + format!(" {object_name}").white();
            let object_state = selected.predict(&self.timeline_state.time()).unwrap();
            ctx.print(object_state.longitude(), object_state.latitude(), text);
        } else if let Some(hovered) = self.state.hovered_object(self.satellite_groups_state) {
            // Highlight the hovered object
            let object_name = hovered.name().unwrap_or(Self::UNKNOWN_NAME);
            let text = Self::OBJECT_SYMBOL.light_red().reversed()
                + " ".into()
                + object_name.to_string().white().reversed();
            let object_state = hovered.predict(&self.timeline_state.time()).unwrap();
            ctx.print(object_state.longitude(), object_state.latitude(), text);
        }
    }

    /// Draws the visibility area for the selected object.
    fn draw_visibility_area(&self, ctx: &mut Context) {
        let Some(object) = self.state.selected_object(self.satellite_groups_state) else {
            return;
        };
        let object_state = object.predict(&self.timeline_state.time()).unwrap();
        let points = calculate_visibility_area(&object_state.position);
        Self::draw_lines(ctx, points, self.state.visibility_area_color);
    }

    fn draw_ground_station(&self, ctx: &mut Context) {
        let Some(ground_station) = &self.sky_state.ground_station else {
            return;
        };
        ctx.print(
            ground_station.position.lon,
            ground_station.position.lat,
            "*".light_cyan().bold() + format!(" {}", ground_station.name).light_cyan(),
        );
    }

    /// Draws lines between points.
    fn draw_lines(ctx: &mut Context, points: Vec<(f64, f64)>, color: Color) {
        for window in points.windows(2) {
            Self::draw_line(ctx, window[0], window[1], color);
        }
    }

    /// Draws a line between two points.
    fn draw_line(ctx: &mut Context, (x1, y1): (f64, f64), (x2, y2): (f64, f64), color: Color) {
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
        KeyCode::Char('t') => {
            app.world_map_state.show_terminator = !app.world_map_state.show_terminator;
        }
        _ => {}
    }

    Ok(())
}

async fn handle_mouse_event(event: MouseEvent, app: &mut App) -> Result<()> {
    let global_mouse = Position::new(event.column, event.row);
    let inner_area = app.world_map_state.inner_area;
    let Some(local_mouse) = window_to_area(global_mouse, inner_area) else {
        app.world_map_state.hovered_object_index = None;
        return Ok(());
    };

    let nearest_object_index = get_nearest_object_index(app, local_mouse, inner_area);
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            app.world_map_state.selected_object_index = nearest_object_index
        }
        MouseEventKind::Down(MouseButton::Right) => {
            app.world_map_state.selected_object_index = None;
        }
        MouseEventKind::ScrollUp => {
            app.world_map_state.scroll_map_left();
        }
        MouseEventKind::ScrollDown => {
            app.world_map_state.scroll_map_right();
        }
        _ => {}
    }
    app.world_map_state.hovered_object_index = nearest_object_index;

    Ok(())
}

/// Get the index of the nearest object to the given area position
fn get_nearest_object_index(app: &App, position: Position, inner_area: Rect) -> Option<usize> {
    app.satellite_groups_state
        .objects
        .iter()
        .enumerate()
        .min_by_key(|(_, obj)| {
            let state = obj.predict(&app.timeline_state.time()).unwrap();
            // Convert to area position
            let (x, y) = lon_lat_to_area(
                wrap_longitude_deg(state.longitude() - app.world_map_state.lon_offset),
                state.latitude(),
                inner_area,
            );
            (x as i32 - position.x as i32).abs() + (y as i32 - position.y as i32).abs() * 2
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
