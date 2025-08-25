use anyhow::Result;
use crossterm::event::MouseEvent;
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders, Paragraph, Wrap,
        canvas::{self, Canvas, Circle, Context},
    },
};
use rust_i18n::t;

use crate::{
    app::App,
    config::SkyConfig,
    event::Event,
    utils::*,
    widgets::{window_to_area, world_map::WorldMapState},
};

use super::satellite_groups::SatelliteGroupsState;

/// A widget that displays the sky track on a polar plot.
pub struct Sky<'a> {
    pub world_map_state: &'a WorldMapState,
    pub satellite_groups_state: &'a SatelliteGroupsState,
}

/// State of a [`Sky`] widget.
pub struct SkyState {
    pub ground_station: Option<Station>,
    canvas_area: Rect,

    mouse_position: Option<(f64, f64)>,

    /// The inner rendering area of the widget.
    inner_area: Rect,
}

pub struct Station {
    pub name: String,
    pub position: Lla,
}

impl SkyState {
    pub fn with_config(config: SkyConfig) -> Self {
        let ground_station = config.ground_station.map(|config| Station {
            name: config
                .name
                .unwrap_or_else(|| config.position.country_city().1),
            position: config.position,
        });
        Self {
            ground_station,
            canvas_area: Default::default(),
            mouse_position: None,
            inner_area: Default::default(),
        }
    }
}

impl Sky<'_> {
    fn block(state: &mut SkyState) -> Block<'static> {
        let mut block = Block::new().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM);
        if let Some((x, y)) = state.mouse_position {
            let (az, el) = canvas_to_az_el(x, y);
            block = block
                .title_bottom(Line::from(format!("Az {:.1}°, El {:.1}°", az, el)).right_aligned());
        }
        block
    }

    fn render_graph(&self, buf: &mut Buffer, state: &mut SkyState) {
        Canvas::default()
            .x_bounds([-1.0, 1.0])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| {
                Self::draw_grid(ctx);
                ctx.layer();
                self.draw_sky_track(ctx, &state.ground_station.as_ref().unwrap().position);
            })
            .render(state.canvas_area, buf);
    }

    fn draw_grid(ctx: &mut Context) {
        for radius in [1.0, 0.67, 0.33] {
            ctx.draw(&Circle {
                x: 0.0,
                y: 0.0,
                radius,
                color: Color::DarkGray,
            });
        }
        ctx.draw(&canvas::Line {
            x1: -1.0,
            y1: 0.0,
            x2: 1.0,
            y2: 0.0,
            color: Color::DarkGray,
        });
        ctx.draw(&canvas::Line {
            x1: 0.0,
            y1: -1.0,
            x2: 0.0,
            y2: 1.0,
            color: Color::DarkGray,
        });
        ctx.print(0.0, 1.0, "N".green());
        ctx.print(1.0, 0.0, "E".green());
        ctx.print(0.0, -1.0, "S".green());
        ctx.print(-1.0, 0.0, "W".green());
    }

    /// Draw the sky track on the polar plot.
    fn draw_sky_track(&self, ctx: &mut Context, station_position: &Lla) {
        const UNKNOWN_NAME: &str = "UNK";

        let Some(object) = self
            .world_map_state
            .selected_object(self.satellite_groups_state)
        else {
            return;
        };
        let time = self.world_map_state.time();

        let points = calculate_sky_track(object, station_position, &time);
        Self::draw_lines(ctx, points, Color::LightBlue);

        // Draw current satellite position if visible
        let object_state = object.predict(&time).unwrap();
        let (az, el) = object_state.position.az_el(station_position);
        if el >= 0.0 {
            let (x, y) = az_el_to_canvas(az, el);
            let object_name = object.name().unwrap_or(UNKNOWN_NAME);
            ctx.print(
                x,
                y,
                "+".light_red().slow_blink() + format!(" {object_name}").white(),
            );
        }
    }

    /// Draws lines between points.
    fn draw_lines(ctx: &mut Context, points: Vec<(f64, f64)>, color: Color) {
        for window in points.windows(2) {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];
            ctx.draw(&canvas::Line {
                x1,
                y1,
                x2,
                y2,
                color,
            });
        }
    }

    fn centered_paragraph<'a>(text: impl Into<Text<'a>>) -> Paragraph<'a> {
        Paragraph::new(text).centered().wrap(Wrap { trim: true })
    }
}

fn centered_square(area: Rect) -> Rect {
    let width = area.width.min(area.height * 2);
    let height = width / 2;
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

impl StatefulWidget for Sky<'_> {
    type State = SkyState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Self::block(state);
        state.inner_area = block.inner(area);
        block.render(area, buf);
        state.canvas_area = centered_square(state.inner_area);

        if state.canvas_area.width.min(state.canvas_area.height) < 5 {
            Self::centered_paragraph(t!("no_enough_space").dark_gray())
                .render(state.inner_area, buf);
            return;
        }

        if state.ground_station.is_none() {
            Self::centered_paragraph(t!("sky.no_ground_station").dark_gray())
                .render(state.inner_area, buf);
            return;
        }

        if self.world_map_state.selected_object_index.is_none() {
            Self::centered_paragraph(t!("no_object_selected").dark_gray())
                .render(state.inner_area, buf);
            return;
        }

        self.render_graph(buf, state);
    }
}

pub async fn handle_event(event: Event, app: &mut App) -> Result<()> {
    match event {
        Event::Mouse(event) => handle_mouse_event(event, app).await,
        _ => Ok(()),
    }
}

async fn handle_mouse_event(event: MouseEvent, app: &mut App) -> Result<()> {
    let global_mouse = Position::new(event.column, event.row);
    let canvas_area = app.sky_state.canvas_area;
    let Some(local_mouse) = window_to_area(global_mouse, canvas_area) else {
        app.sky_state.mouse_position = None;
        return Ok(());
    };

    // Convert window coordinates to canvas coordinates in [-1.0, 1.0].
    let local_x = (local_mouse.x as f64 + 0.5) / (canvas_area.width) as f64;
    let local_y = (local_mouse.y as f64 + 0.5) / (canvas_area.height) as f64;
    let canvas_x = local_x * 2.0 - 1.0;
    let canvas_y = 1.0 - (local_y * 2.0);
    if (canvas_x.powi(2) + canvas_y.powi(2)).sqrt() > 1.0 {
        app.sky_state.mouse_position = None;
        return Ok(());
    }
    app.sky_state.mouse_position = Some((canvas_x, canvas_y));

    Ok(())
}
