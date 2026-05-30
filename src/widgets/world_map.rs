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
#[derive(Default)]
pub struct WorldMapState {
    /// Center longitude offset for horizontal map scrolling in degrees.
    lon_offset: f64,
    /// Center latitude offset for vertical map scrolling in degrees.
    lat_offset: f64,
    /// The amount of longitude (in degrees) to move the map when scrolling left
    /// or right.
    lon_delta: f64,
    /// The amount of latitude (in degrees) to move the map when scrolling up
    /// or down.
    lat_delta: f64,
    /// Zoom level. 1.0 is default (360x180).
    zoom: f64,

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
            lat_delta: config.lon_delta_deg,
            zoom: 1.0,
            map_color: config.map_color,
            trajectory_color: config.trajectory_color,
            terminator_color: config.terminator_color,
            visibility_area_color: config.visibility_area_color,
            ..Self::default()
        }
    }

    /// Scrolls the map view to the left.
    fn scroll_map_left(&mut self) {
        self.lon_offset = wrap_longitude_deg(self.lon_offset - self.lon_delta / self.zoom);
    }

    /// Scrolls the map view to the right.
    fn scroll_map_right(&mut self) {
        self.lon_offset = wrap_longitude_deg(self.lon_offset + self.lon_delta / self.zoom);
    }

    /// Scrolls the map view up.
    fn scroll_map_up(&mut self) {
        self.lat_offset = (self.lat_offset + self.lat_delta / self.zoom).min(90.0);
    }

    /// Scrolls the map view down.
    fn scroll_map_down(&mut self) {
        self.lat_offset = (self.lat_offset - self.lat_delta / self.zoom).max(-90.0);
    }

    /// Zooms in the map.
    fn zoom_in(&mut self) {
        self.zoom *= 1.2;
    }

    /// Zooms out the map.
    fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.2).max(1.0);
        // Reset offsets if zoomed all the way out to prevent weird behavior
        if self.zoom <= 1.0 {
            self.lat_offset = 0.0;
        }
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
        // Follow the longitude of the selected object
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
        }

        let lon_span = 360.0 / self.state.zoom;
        let lat_span = 180.0 / self.state.zoom;

        let x_min = self.state.lon_offset - lon_span / 2.0;
        let x_max = self.state.lon_offset + lon_span / 2.0;

        let mut y_min = self.state.lat_offset - lat_span / 2.0;
        let mut y_max = self.state.lat_offset + lat_span / 2.0;

        // Clamp latitude bounds
        if y_min < -90.0 {
            y_max += -90.0 - y_min;
            y_min = -90.0;
        }
        if y_max > 90.0 {
            y_min -= y_max - 90.0;
            y_max = 90.0;
        }
        y_min = y_min.max(-90.0);
        y_max = y_max.min(90.0);

        let y_bounds = [y_min, y_max];

        // Adjust the rendering order to prevent the labels on the left map from being
        // covered by the right map
        let mut bounds_vec = Vec::new();
        if x_min < -180.0 {
            bounds_vec.push([x_min, x_max]);
            bounds_vec.push([x_min + 360.0, x_max + 360.0]);
        } else if x_max > 180.0 {
            bounds_vec.push([x_min, x_max]);
            bounds_vec.push([x_min - 360.0, x_max - 360.0]);
        } else {
            bounds_vec.push([x_min, x_max]);
        }

        for bounds in &bounds_vec {
            self.render_bottom_layer(buf, *bounds, y_bounds);
        }
        for bounds in &bounds_vec {
            self.render_top_layer(buf, *bounds, y_bounds);
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
                if self.state.show_terminator {
                    self.draw_terminator(ctx);
                }
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

    /// Draws the day-night terminator and subsolar point.
    fn draw_terminator(&self, ctx: &mut Context) {
        // Draw the terminator line
        Self::draw_lines(
            ctx,
            calculate_terminator(&self.shared.time.time()),
            self.state.terminator_color,
        );

        // Mark the subsolar point
        let (sub_lon, sub_lat) = subsolar_point(&self.shared.time.time());
        Self::draw_text(
            ctx,
            sub_lon.to_degrees(),
            sub_lat.to_degrees(),
            Self::SUBSOLAR_SYMBOL.yellow().bold(),
        );
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
            Self::draw_text(ctx, state.longitude(), state.latitude(), text);
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
            Self::draw_text(ctx, object_state.longitude(), object_state.latitude(), text);
        } else if let Some(hovered) = &self.shared.hovered_object {
            // Highlight the hovered object
            let object_name = hovered.name().unwrap_or(Self::UNKNOWN_NAME);
            let text = Self::OBJECT_SYMBOL.light_red().reversed()
                + " ".into()
                + object_name.to_string().white().reversed();
            let object_state = hovered.predict(&self.shared.time.time()).unwrap();
            Self::draw_text(ctx, object_state.longitude(), object_state.latitude(), text);
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
        Self::draw_text(
            ctx,
            ground_station.position.lon,
            ground_station.position.lat,
            "*".light_cyan().bold() + format!(" {}", ground_station.name).light_cyan(),
        );
    }

    /// Draws text at multiple offsets to handle map wrapping.
    fn draw_text<'a, T>(ctx: &mut Context<'a>, x: f64, y: f64, text: T)
    where
        T: Into<Line<'a>> + Clone,
    {
        for offset in [-360.0, 0.0, 360.0] {
            ctx.print(x + offset, y, text.clone());
        }
    }

    /// Draws lines between points.
    fn draw_lines(ctx: &mut Context, points: Vec<(f64, f64)>, color: Color) {
        if points.is_empty() {
            return;
        }

        let mut p1 = points[0];
        for &p2 in &points[1..] {
            let mut p2_adj = p2;
            let lon_diff = p2_adj.0 - p1.0;

            if lon_diff.abs() > 180.0 {
                // Handle IDL crossing by making the longitude continuous
                p2_adj.0 -= 360.0 * lon_diff.signum();
                Self::draw_line(ctx, p1, p2_adj, color);
            } else if lon_diff.abs() > 90.0 && (p1.1.abs() > 70.0 || p2.1.abs() > 70.0) {
                // Handle pole crossing: split the line at the pole to avoid a straight
                // horizontal line across the map.
                let y_pole = if p1.1 > 0.0 { 90.0 } else { -90.0 };
                Self::draw_line(ctx, p1, (p1.0, y_pole), color);
                Self::draw_line(ctx, (p2.0, y_pole), p2, color);
            } else {
                Self::draw_line(ctx, p1, p2, color);
            }
            p1 = p2_adj;
        }
    }

    /// Draws a line between two points, repeating it at 360-degree offsets to
    /// handle map wrapping and zooming correctly.
    fn draw_line(ctx: &mut Context, (x1, y1): (f64, f64), (x2, y2): (f64, f64), color: Color) {
        for offset in [-360.0, 0.0, 360.0] {
            ctx.draw(&canvas::Line::new(x1 + offset, y1, x2 + offset, y2, color));
        }
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
        KeyCode::Char('a') => states.world_map_state.scroll_map_left(),
        KeyCode::Char('d') => states.world_map_state.scroll_map_right(),
        KeyCode::Char('w') => states.world_map_state.scroll_map_up(),
        KeyCode::Char('s') => states.world_map_state.scroll_map_down(),
        KeyCode::Char('e') | KeyCode::Char('+') | KeyCode::Char('=') => {
            states.world_map_state.zoom_in()
        }
        KeyCode::Char('q') | KeyCode::Char('-') | KeyCode::Char('_') => {
            states.world_map_state.zoom_out()
        }
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
            // Convert to area position
            let (x, y) = lon_lat_to_area(
                wrap_longitude_deg(state.longitude() - states.world_map_state.lon_offset),
                state.latitude(),
                inner_area,
                &states.world_map_state,
            );
            ((x - position.x as f64).abs() + (y - position.y as f64).abs() * 2.0) as i32
        })
        .map(|(index, _)| index)
}

#[expect(dead_code)]
/// Converts area coordinates to lon/lat coordinates.
fn area_to_lon_lat(x: u16, y: u16, area: Rect, state: &WorldMapState) -> (f64, f64) {
    debug_assert!(x < area.width && y < area.height);
    debug_assert!(area.width > 0 && area.height > 0);

    let lon_span = 360.0 / state.zoom;
    let lat_span = 180.0 / state.zoom;

    let normalized_x = x as f64 / area.width as f64;
    let normalized_y = y as f64 / area.height as f64;
    let lon = state.lon_offset - lon_span / 2.0 + normalized_x * lon_span;
    let lat = state.lat_offset + lat_span / 2.0 - normalized_y * lat_span;
    (lon, lat)
}

/// Converts lon/lat coordinates to area coordinates.
fn lon_lat_to_area(lon_diff: f64, lat: f64, area: Rect, state: &WorldMapState) -> (f64, f64) {
    let lon_span = 360.0 / state.zoom;
    let lat_span = 180.0 / state.zoom;

    let x = (lon_diff / lon_span + 0.5) * area.width as f64;

    let mut y_min = state.lat_offset - lat_span / 2.0;
    let mut y_max = state.lat_offset + lat_span / 2.0;

    if y_min < -90.0 {
        y_max += -90.0 - y_min;
        y_min = -90.0;
    }
    if y_max > 90.0 {
        y_min -= y_max - 90.0;
        y_max = 90.0;
    }
    y_min = y_min.max(-90.0);
    y_max = y_max.min(90.0);

    let y_span = y_max - y_min;
    let y = if y_span > 0.0 {
        ((y_max - lat) / y_span) * area.height as f64
    } else {
        0.0
    };

    (x, y)
}
