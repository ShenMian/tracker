use ratatui::{
    prelude::*,
    widgets::{
        Block, Paragraph, Wrap,
        canvas::{Canvas, Circle, Context, Line},
    },
};
use rust_i18n::t;

use crate::{config::PolarConfig, utils::*, widgets::world_map::WorldMapState};

use super::satellite_groups::SatelliteGroupsState;

/// A widget to display the sky track.
pub struct Polar<'a> {
    pub world_map_state: &'a WorldMapState,
    pub satellite_groups_state: &'a SatelliteGroupsState,
}

/// State of a [`Polar`] widget.
pub struct PolarState {
    pub ground_station: Option<Lla>,

    /// The inner rendering area of the widget.
    pub inner_area: Rect,
}

impl PolarState {
    pub fn with_config(config: PolarConfig) -> Self {
        Self {
            ground_station: config.ground_station,
            inner_area: Default::default(),
        }
    }
}

impl Polar<'_> {
    fn render_block(&self, area: Rect, buf: &mut Buffer, state: &mut PolarState) {
        let block = Block::bordered().title(t!("polar.title").to_string().blue());
        state.inner_area = block.inner(area);
        block.render(area, buf);
    }

    fn render_graph(&self, buf: &mut Buffer, state: &mut PolarState) {
        let Rect {
            x,
            y,
            width,
            height,
        } = state.inner_area;

        let canvas_width = width.min(height * 2);
        let canvas_height = canvas_width / 2;
        let area = Rect {
            x: x + (width - canvas_width) / 2,
            y: y + (height - canvas_height) / 2,
            width: canvas_width,
            height: canvas_height,
        };

        Canvas::default()
            .x_bounds([-1.0, 1.0])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| {
                self.draw_grid(ctx);
                ctx.layer();
                self.draw_sky_track(ctx, state.ground_station.as_ref().unwrap());
            })
            .render(area, buf);
    }

    fn render_paragraph<'a>(
        &self,
        text: impl Into<Text<'a>>,
        buf: &mut Buffer,
        state: &mut PolarState,
    ) {
        Paragraph::new(text)
            .centered()
            .wrap(Wrap { trim: true })
            .render(state.inner_area, buf);
    }

    fn draw_grid(&self, ctx: &mut Context) {
        for radius in [0.9, 0.6, 0.3] {
            ctx.draw(&Circle {
                x: 0.0,
                y: 0.0,
                radius,
                color: Color::DarkGray,
            });
        }
        ctx.draw(&Line {
            x1: -1.0,
            y1: 0.0,
            x2: 1.0,
            y2: 0.0,
            color: Color::DarkGray,
        });
        ctx.draw(&Line {
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

    fn draw_sky_track(&self, ctx: &mut Context, ground_station: &Lla) {
        const UNKNOWN_NAME: &str = "UNK";

        let Some(object) = self
            .world_map_state
            .selected_object(self.satellite_groups_state)
        else {
            return;
        };
        let time = self.world_map_state.time();

        let points = calculate_sky_track(object, ground_station, &time);
        self.draw_lines(ctx, points, Color::LightBlue);

        // Draw current satellite position if visible
        let object_state = object.predict(&time).unwrap();
        let (az_deg, el_deg) = object_state.position.az_el(ground_station);
        if el_deg >= 0.0 {
            let r = (1.0 - (el_deg / 90.0)).clamp(0.0, 1.0);
            let az_rad = az_deg.to_radians();
            let x = r * az_rad.sin();
            let y = r * az_rad.cos();
            let object_name = object.name().unwrap_or(UNKNOWN_NAME);
            ctx.print(
                x,
                y,
                "+".light_red().slow_blink() + format!(" {object_name}").white(),
            );
        }
    }

    /// Draws lines between points.
    fn draw_lines(&self, ctx: &mut Context, points: Vec<(f64, f64)>, color: Color) {
        for window in points.windows(2) {
            let (x1, y1) = window[0];
            let (x2, y2) = window[1];
            ctx.draw(&Line {
                x1,
                y1,
                x2,
                y2,
                color,
            });
        }
    }
}

impl StatefulWidget for Polar<'_> {
    type State = PolarState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_block(area, buf, state);
        if state.ground_station.is_none() {
            self.render_paragraph(t!("polar.no_ground_station").dark_gray(), buf, state);
            return;
        }
        if self
            .world_map_state
            .selected_object(self.satellite_groups_state)
            .is_some()
        {
            self.render_graph(buf, state);
        } else {
            self.render_paragraph(t!("no_object_selected").dark_gray(), buf, state);
        }
    }
}
