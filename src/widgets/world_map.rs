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
    /// The amount of longitude (in degrees) to move the map when scrolling left
    /// or right.
    lon_delta: f64,

    /// Whether to follow the selected object by adjusting the map longitude.
    follow_object: bool,
    /// The smoothing factor for follow mode.
    follow_smoothing: f64,
    /// Whether to mark the subsolar point.
    show_subsolar_point: bool,
    /// Whether to shade the night side of the Earth.
    show_night_shading: bool,
    /// Whether to display the visibility area.
    show_visibility_area: bool,
    /// Whether to display the day-night terminator line.
    show_terminator: bool,

    coast_color: Color,
    /// Background colour of the lit (day) side of the Earth.
    day_color: Color,
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
            show_subsolar_point: config.show_subsolar_point,
            show_night_shading: config.show_night_shading,
            show_visibility_area: config.show_visibility_area,
            show_terminator: config.show_terminator,
            lon_delta: config.lon_delta_deg,
            coast_color: config.coast_color,
            day_color: config.day_color,
            trajectory_color: config.trajectory_color,
            terminator_color: config.terminator_color,
            visibility_area_color: config.visibility_area_color,
            ..Self::default()
        }
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
        let mut block = Block::bordered()
            .border_set(symbols::border::Set {
                bottom_left: symbols::line::VERTICAL_RIGHT,
                bottom_right: symbols::line::VERTICAL_LEFT,
                ..Default::default()
            })
            .title(t!("map-title").to_string().blue());

        // Show follow mode indicator if enabled
        if self.state.follow_object {
            let style = if self.shared.selected_object.is_none() {
                Style::new().dark_gray()
            } else {
                Style::new().green().slow_blink()
            };
            block = block.title_bottom(
                Line::from(format!("({})", t!("map-follow")).set_style(style)).right_aligned(),
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
        }

        let x_min = self.state.lon_offset - 180.0;
        let x_max = self.state.lon_offset + 180.0;

        // Adjust the rendering order to prevent the labels on the left map from
        // being covered by the right map
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

        // Shade last, once every layer has been painted: setting only the
        // background leaves the glyphs and their foreground colour untouched,
        // so the map, objects and labels all stay legible on top of the tint.
        if self.state.show_night_shading {
            self.shade_night(buf);
        }
    }

    /// Tints the background of every cell with its side of the terminator and
    /// recolours the coastline on the night side.
    fn shade_night(&self, buf: &mut Buffer) {
        let area = self.state.inner_area;
        if area.width == 0 || area.height == 0 {
            return;
        }

        let subsolar = subsolar_point(&self.shared.time.time());
        let lon_offset = self.state.lon_offset;
        let day_color = self.state.day_color;
        let night_color = dim_color(day_color);
        let coast_color = self.state.coast_color;
        let coast_night_color = dim_color(coast_color);

        for row in 0..area.height {
            for col in 0..area.width {
                let (lon, lat) = cell_center_to_lon_lat(col, row, area, lon_offset);
                let Some(cell) = buf.cell_mut((area.x + col, area.y + row)) else {
                    continue;
                };
                let is_day = is_daylight(subsolar, lon, lat);
                cell.set_bg(if is_day { day_color } else { night_color });
                if !is_day && cell.fg == coast_color && is_braille(cell.symbol()) {
                    cell.set_fg(coast_night_color);
                }
            }
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
                    color: self.state.coast_color,
                    resolution: MapResolution::High,
                });
                ctx.layer();
                if self.state.show_terminator {
                    self.draw_terminator(ctx);
                }
                if self.state.show_subsolar_point {
                    self.draw_subsolar_point(ctx);
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

    /// Draws the day-night terminator line.
    fn draw_terminator(&self, ctx: &mut Context) {
        Self::draw_lines(
            ctx,
            calculate_terminator(&self.shared.time.time()),
            self.state.terminator_color,
        );
    }

    /// Marks the subsolar point: the spot on the globe where the sun is
    /// directly overhead, i.e. the centre of the day side.
    fn draw_subsolar_point(&self, ctx: &mut Context) {
        let (sub_lon, sub_lat) = subsolar_point(&self.shared.time.time());
        ctx.print(
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
            .filter_map(|object| {
                let object_name = object.name().unwrap_or(Self::UNKNOWN_NAME);
                let text = if self.shared.selected_object.is_none() {
                    Self::OBJECT_SYMBOL.light_red() + format!(" {object_name}").white()
                } else {
                    Self::OBJECT_SYMBOL.red() + format!(" {object_name}").dark_gray()
                };
                Some((text, object.predict(&time).ok()?))
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
        KeyCode::Char('[') => states.world_map_state.scroll_map_left(),
        KeyCode::Char(']') => states.world_map_state.scroll_map_right(),
        KeyCode::Char('f') => {
            states.world_map_state.follow_object = !states.world_map_state.follow_object;
        }
        KeyCode::Char('t') => {
            states.world_map_state.show_terminator = !states.world_map_state.show_terminator;
        }
        KeyCode::Char('n') => {
            states.world_map_state.show_night_shading = !states.world_map_state.show_night_shading;
        }
        KeyCode::Char('s') => {
            states.world_map_state.show_subsolar_point =
                !states.world_map_state.show_subsolar_point;
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
            states.world_map_state.scroll_map_left();
        }
        MouseEventKind::ScrollDown => {
            states.world_map_state.scroll_map_right();
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

/// Converts area coordinates to the geographic coordinate at the cell centre.
fn cell_center_to_lon_lat(col: u16, row: u16, area: Rect, lon_offset: f64) -> (f64, f64) {
    debug_assert!(col < area.width && row < area.height);
    debug_assert!(area.width > 0 && area.height > 0);

    let normalized_x = (f64::from(col) + 0.5) / f64::from(area.width);
    let normalized_y = (f64::from(row) + 0.5) / f64::from(area.height);
    let lon = wrap_longitude_deg(lon_offset - 180.0 + normalized_x * 360.0);
    let lat = 90.0 - normalized_y * 180.0;
    (lon, lat)
}

/// How far night-side colours are scaled towards black.
const NIGHT_DIM_FACTOR: f64 = 0.7;

/// Returns `color` scaled towards black.
fn dim_color(color: Color) -> Color {
    match color {
        Color::Gray => Color::DarkGray,
        Color::White => Color::Gray,
        Color::LightRed => Color::Red,
        Color::LightGreen => Color::Green,
        Color::LightYellow => Color::Yellow,
        Color::LightBlue => Color::Blue,
        Color::LightMagenta => Color::Magenta,
        Color::LightCyan => Color::Cyan,
        Color::Rgb(r, g, b) => {
            let dim = |c: u8| (f64::from(c) * NIGHT_DIM_FACTOR) as u8;
            Color::Rgb(dim(r), dim(g), dim(b))
        }
        other => other,
    }
}

/// Returns true when `symbol` is a single Braille character.
fn is_braille(symbol: &str) -> bool {
    symbol
        .chars()
        .next()
        .is_some_and(|c| ('\u{2800}'..='\u{28FF}').contains(&c))
}

/// Converts lon/lat coordinates to area coordinates.
fn lon_lat_to_area(lon: f64, lat: f64, area: Rect) -> (u16, u16) {
    debug_assert!((-180.0..=180.0).contains(&lon));
    debug_assert!((-90.0..=90.0).contains(&lat));

    let x = ((lon + 180.0) * area.width as f64 / 360.0) - 1.0;
    let y = ((90.0 - lat) * area.height as f64 / 180.0) - 1.0;
    (x.round() as u16, y.round() as u16)
}
