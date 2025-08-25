use anyhow::Result;
use chrono::{DateTime, Duration, Local, Timelike, Utc};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders,
        canvas::{self, Canvas, Context},
    },
};

use crate::app::App;
use crate::event::Event;
use crate::widgets::window_to_area;
use crate::widgets::world_map::WorldMapState;

pub struct Timeline<'a> {
    pub world_map_state: &'a WorldMapState,
}

#[derive(Default)]
pub struct TimelineState {
    inner_area: Rect,
    mouse_position: Option<Position>,
}

impl Timeline<'_> {
    const HOURS_WINDOW: i64 = 8;

    fn block(&self, state: &TimelineState) -> Block<'static> {
        let mut block = Block::new().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM);
        if let Some(duration) = self.mouse_time(state) {
            let time = self.world_map_state.time().with_timezone(&Local) + duration;
            let label = time.format("%Y-%m-%d %H:%M:%S").to_string();
            block = block.title_bottom(Line::from(label).right_aligned());
        }
        block
    }

    fn draw_axis(ctx: &mut Context) {
        ctx.draw(&canvas::Line {
            x1: 0.0,
            y1: 0.5,
            x2: Self::HOURS_WINDOW as f64,
            y2: 0.5,
            color: Color::DarkGray,
        });
    }

    fn draw_hour_marks(ctx: &mut Context, time: DateTime<Utc>) {
        for offset in (-Self::HOURS_WINDOW / 2)..=(Self::HOURS_WINDOW / 2) {
            let time = time.with_timezone(&Local) + Duration::hours(offset);
            let minutes = time.minute() % 60;
            let hours = time.hour() % 24;

            let x_offset = (Self::HOURS_WINDOW / 2) as f64 + offset as f64 - minutes as f64 / 60.0;
            ctx.draw(&canvas::Line {
                x1: x_offset,
                y1: 0.5,
                x2: x_offset,
                y2: 0.5,
                color: Color::White,
            });
            ctx.print(x_offset, 0.0, format!("{:02}", hours).fg(Color::DarkGray));
        }
    }

    fn draw_current_time_marker(ctx: &mut Context) {
        ctx.draw(&canvas::Line {
            x1: Self::HOURS_WINDOW as f64 / 2.0,
            y1: 0.0,
            x2: Self::HOURS_WINDOW as f64 / 2.0,
            y2: 1.0,
            color: Color::LightRed,
        });
    }

    fn render_canvas(&self, buf: &mut Buffer, state: &mut TimelineState) {
        Canvas::default()
            .x_bounds([0.0, Self::HOURS_WINDOW as f64])
            .y_bounds([0.0, 1.0])
            .paint(|ctx| {
                Self::draw_axis(ctx);
                ctx.layer();
                Self::draw_hour_marks(ctx, self.world_map_state.time());
                ctx.layer();
                Self::draw_current_time_marker(ctx);
            })
            .render(state.inner_area, buf);
    }

    fn mouse_time(&self, state: &TimelineState) -> Option<Duration> {
        let mouse = state.mouse_position?;
        let duration = canvas_offset_to_duration(area_to_canvas_offset(state.inner_area, mouse));
        Some(duration)
    }
}

impl StatefulWidget for Timeline<'_> {
    type State = TimelineState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = self.block(state);
        state.inner_area = block.inner(area);
        block.render(area, buf);

        self.render_canvas(buf, state);
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
    let inner_area = app.timeline_state.inner_area;
    let Some(local_mouse) = window_to_area(global_mouse, inner_area) else {
        app.timeline_state.mouse_position = None;
        return Ok(());
    };
    app.timeline_state.mouse_position = Some(local_mouse);

    let duration = canvas_offset_to_duration(area_to_canvas_offset(inner_area, local_mouse));
    let time = app.world_map_state.time() + duration;

    if let MouseEventKind::Down(MouseButton::Left) = event.kind {
        app.world_map_state.set_time(time);
    }

    Ok(())
}

fn area_to_canvas_offset(area: Rect, position: Position) -> f64 {
    (position.x as f64 + 0.5) / area.width as f64 * Timeline::HOURS_WINDOW as f64
}

fn canvas_offset_to_duration(offset: f64) -> Duration {
    const SECS_PER_HOUR: f64 = 3600.0;
    let hours_offset = offset - Timeline::HOURS_WINDOW as f64 / 2.0;
    Duration::seconds((hours_offset * SECS_PER_HOUR).round() as i64)
}
