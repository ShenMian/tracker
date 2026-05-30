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
use rayon::prelude::*;
use rust_i18n::t;

use crate::{
    app::States, config::WorldMapConfig, event::Event, shared_state::SharedState, utils::*,
    widgets::window_to_area,
};

/// A widget that displays a world map with objects.
pub struct WorldMap<'a> {
    pub state: &'a mut WorldMapState,
    pub shared: &'a SharedState,
}

/// State of a [`WorldMap`] widget.
/// State of a [`WorldMap`] widget.
pub struct WorldMapState {
    /// Center longitude offset for horizontal map scrolling in degrees.
    pub lon_offset: f64,
    /// Center latitude offset for vertical map scrolling in degrees.
    pub lat_offset: f64,
    /// Zoom level of the map (>= 1.0).
    pub zoom: f64,
    /// The amount of longitude (in degrees) to move the map when scrolling.
    lon_delta: f64,

    /// Whether to follow the selected object by adjusting the map offset.
    follow_object: bool,
    /// The smoothing factor for follow mode.
    follow_smoothing: f64,
    /// Whether to display the day-night terminator line.
    show_terminator: bool,
    /// Whether to display the visibility area.
    show_visibility_area: bool,

    map_color: Color,
    trajectory_color: Color,
    visibility_area_color: Color,

    /// The inner rendering area of the widget.
    pub inner_area: Rect,
}

impl Default for WorldMapState {
    fn default() -> Self {
        Self {
            lon_offset: 0.0,
            lat_offset: 0.0,
            zoom: 1.0,
            lon_delta: 10.0,
            follow_object: true,
            follow_smoothing: 0.3,
            show_terminator: true,
            show_visibility_area: true,
            map_color: Color::Gray,
            trajectory_color: Color::LightBlue,
            visibility_area_color: Color::Yellow,
            inner_area: Rect::default(),
        }
    }
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
            visibility_area_color: config.visibility_area_color,
            ..Self::default()
        }
    }

    /// Scrolls the map view to the left.
    fn scroll_left(&mut self) {
        let delta = self.lon_delta / self.zoom;
        self.lon_offset = wrap_longitude_deg(self.lon_offset - delta);
    }

    /// Scrolls the map view to the right.
    fn scroll_right(&mut self) {
        let delta = self.lon_delta / self.zoom;
        self.lon_offset = wrap_longitude_deg(self.lon_offset + delta);
    }

    /// Scrolls the map view up.
    fn scroll_up(&mut self) {
        let delta = self.lon_delta / self.zoom;
        let limit = 90.0 - 90.0 / self.zoom;
        self.lat_offset = (self.lat_offset + delta).clamp(-limit, limit);
    }

    /// Scrolls the map view down.
    fn scroll_down(&mut self) {
        let delta = self.lon_delta / self.zoom;
        let limit = 90.0 - 90.0 / self.zoom;
        self.lat_offset = (self.lat_offset - delta).clamp(-limit, limit);
    }

    /// Zooms in.
    fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.2).min(100.0);
        self.clamp_lat_offset();
    }

    /// Zooms out.
    fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.2).max(1.0);
        self.clamp_lat_offset();
    }

    /// Resets scroll and zoom.
    fn reset_view(&mut self) {
        self.lon_offset = 0.0;
        self.lat_offset = 0.0;
        self.zoom = 1.0;
    }

    /// Clamps `lat_offset` so that the camera doesn't scroll beyond poles.
    fn clamp_lat_offset(&mut self) {
        let limit = 90.0 - 90.0 / self.zoom;
        self.lat_offset = self.lat_offset.clamp(-limit, limit);
    }
}

impl WorldMap<'_> {
    const OBJECT_SYMBOL: &'static str = "+";
    const UNKNOWN_NAME: &'static str = "UNK";

    pub fn render(mut self, area: Rect, buf: &mut Buffer) {
        let block = self.block();
        self.state.inner_area = block.inner(area);
        block.render(area, buf);

        self.render_map(buf);

        // Apply custom cell-shading (e.g. Day/Night cycles)
        self.apply_shading(buf);
    }

    fn block(&self) -> Block<'static> {
        let mut block = Block::bordered()
            .border_set(symbols::border::Set {
                bottom_left: symbols::line::VERTICAL_RIGHT,
                bottom_right: symbols::line::VERTICAL_LEFT,
                ..Default::default()
            })
            .title(t!("map.title").to_string().blue());

        // Show follow mode indicator if enabled
        if self.state.follow_object {
            let style = if self.shared.selected_object.is_none() {
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
        // Follow the coordinates of the selected object
        if self.state.follow_object
            && let Some(selected) = &self.shared.selected_object
        {
            let object_state = selected.predict(&self.shared.time.time()).unwrap();

            self.state.lon_offset +=
                wrap_longitude_deg(object_state.longitude() - self.state.lon_offset)
                    * self.state.follow_smoothing;
            self.state.lon_offset = wrap_longitude_deg(self.state.lon_offset);

            self.state.lat_offset +=
                (object_state.latitude() - self.state.lat_offset) * self.state.follow_smoothing;
            self.state.clamp_lat_offset();
        }

        let lon_span = 360.0 / self.state.zoom;
        let lat_span = 180.0 / self.state.zoom;

        let left = self.state.lon_offset - lon_span / 2.0;
        let right = self.state.lon_offset + lon_span / 2.0;
        let bottom = self.state.lat_offset - lat_span / 2.0;
        let top = self.state.lat_offset + lat_span / 2.0;

        // Find range of offsets k to wrap longitude horizontally
        let k_start = ((left - 180.0) / 360.0).floor() as i32 + 1;
        let k_end = ((right + 180.0) / 360.0).ceil() as i32 - 1;

        let mut bounds_vec = Vec::new();
        for k in k_start..=k_end {
            let offset = k as f64 * 360.0;
            bounds_vec.push([left - offset, right - offset]);
        }

        if bounds_vec.is_empty() {
            bounds_vec.push([left, right]);
        }

        for bounds in &bounds_vec {
            self.render_bottom_layer(buf, *bounds, [bottom, top]);
        }
        for bounds in &bounds_vec {
            self.render_top_layer(buf, *bounds, [bottom, top]);
        }
    }

    /// Renders the bottom layer of the world map, including the map and all
    /// objects.
    fn render_bottom_layer(&self, buf: &mut Buffer, x_bounds: [f64; 2], y_bounds: [f64; 2]) {
        Canvas::default()
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .paint(|ctx| {
                ctx.draw(&Map {
                    color: self.state.map_color,
                    resolution: MapResolution::High,
                });
                ctx.layer();
                self.draw_objects(ctx);
            })
            .render(self.state.inner_area, buf);
    }

    /// Renders the top layer of the world map, including object highlights and
    /// trajectories.
    fn render_top_layer(&self, buf: &mut Buffer, x_bounds: [f64; 2], y_bounds: [f64; 2]) {
        Canvas::default()
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .paint(|ctx| {
                self.draw_object_highlight(ctx);
                if self.state.show_visibility_area {
                    self.draw_visibility_area(ctx);
                }
                self.draw_ground_station(ctx);
            })
            .render(self.state.inner_area, buf);
    }

    /// Draws all objects and their labels.
    fn draw_objects(&self, ctx: &mut Context) {
        let time = self.shared.time.time();

        for (text, state) in self
            .shared
            .objects
            .par_iter()
            .map(|object| {
                let object_name = object.name().unwrap_or(Self::UNKNOWN_NAME);
                let text = if self.shared.selected_object.is_none() {
                    Self::OBJECT_SYMBOL.light_red() + format!(" {object_name}").white()
                } else {
                    Self::OBJECT_SYMBOL.red() + format!(" {object_name}").dark_gray()
                };
                (text, object.predict(&time).unwrap())
            })
            .collect::<Vec<_>>()
        {
            ctx.print(state.longitude(), state.latitude(), text);
        }
    }

    /// Draws the highlight and trajectory for the selected or hovered object.
    fn draw_object_highlight(&self, ctx: &mut Context) {
        if let Some(selected) = &self.shared.selected_object {
            // Draw the trajectory
            Self::draw_lines(
                ctx,
                calculate_ground_track(selected, &self.shared.time.time()),
                self.state.trajectory_color,
            );

            // Highlight the selected object
            let object_name = selected.name().unwrap_or(Self::UNKNOWN_NAME);
            let text =
                Self::OBJECT_SYMBOL.light_green().slow_blink() + format!(" {object_name}").white();
            let object_state = selected.predict(&self.shared.time.time()).unwrap();
            ctx.print(object_state.longitude(), object_state.latitude(), text);
        } else if let Some(hovered) = &self.shared.hovered_object {
            // Highlight the hovered object
            let object_name = hovered.name().unwrap_or(Self::UNKNOWN_NAME);
            let text = Self::OBJECT_SYMBOL.light_red().reversed()
                + " ".into()
                + object_name.to_string().white().reversed();
            let object_state = hovered.predict(&self.shared.time.time()).unwrap();
            ctx.print(object_state.longitude(), object_state.latitude(), text);
        }
    }

    /// Draws the visibility area for the selected object.
    fn draw_visibility_area(&self, ctx: &mut Context) {
        let Some(object) = &self.shared.selected_object else {
            return;
        };
        let object_state = object.predict(&self.shared.time.time()).unwrap();
        let points = calculate_visibility_area(&object_state.position);
        Self::draw_lines(ctx, points, self.state.visibility_area_color);
    }

    fn draw_ground_station(&self, ctx: &mut Context) {
        let Some(ground_station) = &self.shared.ground_station else {
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

    /// Apply custom cell shading mapping day/night cycles onto the map background and foreground.
    fn apply_shading(&self, buf: &mut Buffer) {
        if !self.state.show_terminator {
            return;
        }
        let area = self.state.inner_area;
        if area.width == 0 || area.height == 0 {
            return;
        }

        let time = self.shared.time.time();
        let (sub_lon, sub_lat) = subsolar_point(&time);

        let lon_span = 360.0 / self.state.zoom;
        let lat_span = 180.0 / self.state.zoom;
        let lon_min = self.state.lon_offset - lon_span / 2.0;
        let lat_max = self.state.lat_offset + lat_span / 2.0;

        for y in area.top()..area.bottom() {
            let dy = y - area.y;
            let normalized_y = dy as f64 / area.height as f64;
            let lat = lat_max - normalized_y * lat_span;
            let lat_rad = lat.to_radians();

            for x in area.left()..area.right() {
                let dx = x - area.x;
                let normalized_x = dx as f64 / area.width as f64;
                let lon = wrap_longitude_deg(lon_min + normalized_x * lon_span);
                let lon_rad = lon.to_radians();

                let cos_theta = lat_rad.sin() * sub_lat.sin()
                    + lat_rad.cos() * sub_lat.cos() * (lon_rad - sub_lon).cos();
                let is_day = cos_theta >= 0.0;

                let cell = &mut buf[(x, y)];

                if !is_day {
                    let bg = cell.style().bg.unwrap_or(Color::Reset);
                    let fg = cell.style().fg.unwrap_or(Color::Reset);

                    let shaded_bg = match bg {
                        Color::Reset | Color::Black => Color::Rgb(8, 12, 24),
                        _ => darken_color(bg),
                    };

                    let shaded_fg = match fg {
                        Color::Reset | Color::White | Color::Gray => Color::DarkGray,
                        _ => darken_color(fg),
                    };

                    cell.set_bg(shaded_bg).set_fg(shaded_fg);
                } else {
                    let bg = cell.style().bg.unwrap_or(Color::Reset);
                    if bg == Color::Reset || bg == Color::Black {
                        cell.set_bg(Color::Rgb(15, 25, 45));
                    }
                }
            }
        }
    }
}

fn darken_color(color: Color) -> Color {
    match color {
        Color::Red => Color::DarkGray,
        Color::LightRed => Color::Red,
        Color::Green => Color::Rgb(0, 50, 0),
        Color::LightGreen => Color::Green,
        Color::Yellow => Color::Rgb(100, 100, 0),
        Color::LightYellow => Color::Yellow,
        Color::Blue => Color::Rgb(0, 0, 100),
        Color::LightBlue => Color::Blue,
        Color::Magenta => Color::Rgb(100, 0, 100),
        Color::LightMagenta => Color::Magenta,
        Color::Cyan => Color::Rgb(0, 100, 100),
        Color::LightCyan => Color::Cyan,
        Color::Gray => Color::DarkGray,
        Color::DarkGray => Color::Rgb(30, 30, 30),
        Color::White => Color::Gray,
        Color::Rgb(r, g, b) => Color::Rgb(r / 3, g / 3, b / 3),
        Color::Indexed(idx) => Color::Indexed(idx),
        _ => Color::DarkGray,
    }
}

pub fn handle_event(event: Event, states: &mut States) -> Result<()> {
    match event {
        Event::Key(event) => handle_key_event(event, states),
        Event::Mouse(event) => handle_mouse_event(event, states),
        _ => Ok(()),
    }
}

fn handle_key_event(event: KeyEvent, states: &mut States) -> Result<()> {
    match event.code {
        KeyCode::Char('[') | KeyCode::Char('h') | KeyCode::Left | KeyCode::Char('a') => {
            states.world_map_state.scroll_left()
        }
        KeyCode::Char(']') | KeyCode::Char('l') | KeyCode::Right | KeyCode::Char('d') => {
            states.world_map_state.scroll_right()
        }
        KeyCode::Char('w') | KeyCode::Char('k') | KeyCode::Up => states.world_map_state.scroll_up(),
        KeyCode::Char('s') | KeyCode::Char('j') | KeyCode::Down => {
            states.world_map_state.scroll_down()
        }
        KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('i') => {
            states.world_map_state.zoom_in()
        }
        KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Char('o') => {
            states.world_map_state.zoom_out()
        }
        KeyCode::Char('0') => states.world_map_state.reset_view(),
        KeyCode::Char('f') => {
            states.world_map_state.follow_object = !states.world_map_state.follow_object;
        }
        KeyCode::Char('t') => {
            states.world_map_state.show_terminator = !states.world_map_state.show_terminator;
        }
        _ => {}
    }

    Ok(())
}

fn handle_mouse_event(event: MouseEvent, states: &mut States) -> Result<()> {
    let global_mouse = Position::new(event.column, event.row);
    let inner_area = states.world_map_state.inner_area;
    let Some(local_mouse) = window_to_area(global_mouse, inner_area) else {
        states.shared.hovered_object = None;
        return Ok(());
    };

    let nearest_object_index = get_nearest_object_index(states, local_mouse, inner_area);
    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            states.shared.selected_object =
                nearest_object_index.map(|index| states.shared.objects[index].clone());
        }
        MouseEventKind::Down(MouseButton::Right) => {
            states.shared.selected_object = None;
        }
        MouseEventKind::ScrollUp => {
            states.world_map_state.zoom_in();
        }
        MouseEventKind::ScrollDown => {
            states.world_map_state.zoom_out();
        }
        _ => {}
    }
    states.shared.hovered_object =
        nearest_object_index.map(|index| states.shared.objects[index].clone());

    Ok(())
}

/// Get the index of the nearest object to the given area position.
fn get_nearest_object_index(
    states: &States,
    position: Position,
    inner_area: Rect,
) -> Option<usize> {
    let time = states.shared.time.time();

    states
        .shared
        .objects
        .par_iter()
        .enumerate()
        .min_by_key(|(_, obj)| {
            let state = obj.predict(&time).unwrap();
            let (x, y) = lon_lat_to_area(
                state.longitude(),
                state.latitude(),
                inner_area,
                states.world_map_state.lon_offset,
                states.world_map_state.lat_offset,
                states.world_map_state.zoom,
            );
            (x as i32 - position.x as i32).abs() + (y as i32 - position.y as i32).abs() * 2
        })
        .map(|(index, _)| index)
}

#[expect(dead_code)]
/// Converts area coordinates to lon/lat coordinates.
fn area_to_lon_lat(
    x: u16,
    y: u16,
    area: Rect,
    lon_offset: f64,
    lat_offset: f64,
    zoom: f64,
) -> (f64, f64) {
    debug_assert!(x < area.width && y < area.height);
    debug_assert!(area.width > 0 && area.height > 0);

    let lon_span = 360.0 / zoom;
    let lat_span = 180.0 / zoom;

    let lon_min = lon_offset - lon_span / 2.0;
    let lat_max = lat_offset + lat_span / 2.0;

    let normalized_x = x as f64 / area.width as f64;
    let normalized_y = y as f64 / area.height as f64;

    let lon = wrap_longitude_deg(lon_min + normalized_x * lon_span);
    let lat = lat_max - normalized_y * lat_span;
    (lon, lat)
}

/// Converts lon/lat coordinates to area coordinates.
fn lon_lat_to_area(
    lon: f64,
    lat: f64,
    area: Rect,
    lon_offset: f64,
    lat_offset: f64,
    zoom: f64,
) -> (u16, u16) {
    let lon_span = 360.0 / zoom;
    let lat_span = 180.0 / zoom;

    let lon_max_bound = lat_offset + lat_span / 2.0;

    let d_lon = wrap_longitude_deg(lon - lon_offset);
    let relative_lon = d_lon + lon_span / 2.0;

    let normalized_x = relative_lon / lon_span;
    let normalized_y = (lon_max_bound - lat) / lat_span;

    let x = (normalized_x * area.width as f64).round() as i32;
    let y = (normalized_y * area.height as f64).round() as i32;

    (
        x.clamp(0, area.width as i32 - 1) as u16,
        y.clamp(0, area.height as i32 - 1) as u16,
    )
}
