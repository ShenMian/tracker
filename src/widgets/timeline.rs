use anyhow::Result;
use chrono::{DateTime, Duration, Local, Timelike, Utc};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders,
        canvas::{self, Canvas, Context},
    },
};

use crate::{
    app::App,
    config::TimelineConfig,
    event::Event,
    utils::calculate_pass_times,
    widgets::{
        satellite_groups::SatelliteGroupsState, sky::SkyState, window_to_area,
        world_map::WorldMapState,
    },
};

const SECS_PER_HOUR: f64 = 3600.0;

pub struct Timeline<'a> {
    pub world_map_state: &'a WorldMapState,
    pub satellite_groups_state: &'a SatelliteGroupsState,
    pub sky_state: &'a SkyState,
}

#[derive(Default)]
pub struct TimelineState {
    mouse_position: Option<Position>,
    /// Time offset from the current UTC time for time simulation.
    time_offset: Duration,
    /// The time step to advance or rewind when scrolling time.
    time_delta: Duration,
    /// The inner rendering area of the widget.
    inner_area: Rect,
}

impl TimelineState {
    /// Creates a new `TimelineState` with the given configuration.
    pub fn with_config(config: TimelineConfig) -> Self {
        Self {
            time_delta: Duration::minutes(config.time_delta_min),
            ..Default::default()
        }
    }

    /// Returns the current simulation time.
    pub fn time(&self) -> DateTime<Utc> {
        Utc::now() + self.time_offset
    }

    /// Sets the current simulation time.
    pub fn set_time(&mut self, time: DateTime<Utc>) {
        self.time_offset = time - Utc::now();
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

impl Timeline<'_> {
    const HOURS_WINDOW: i64 = 8;

    fn block(&self, state: &TimelineState) -> Block<'static> {
        let mut block = Block::new()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .title_bottom(
                format!(
                    "{} ({:+}m)",
                    state
                        .time()
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S"),
                    state.time_offset.num_minutes()
                )
                .white(),
            );

        if let Some(duration) = self.mouse_time(state) {
            let time = state.time().with_timezone(&Local) + duration;
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

    fn draw_pass_times(&self, ctx: &mut Context, state: &TimelineState) {
        let Some(selected_object) = self
            .world_map_state
            .selected_object(self.satellite_groups_state)
        else {
            return;
        };
        let Some(ground_station) = &self.sky_state.ground_station else {
            return;
        };

        let current_time = state.time();
        let pass_segments = calculate_pass_times(
            selected_object,
            &ground_station.position,
            &(current_time - Duration::hours(Self::HOURS_WINDOW) / 2),
            &(current_time + Duration::hours(Self::HOURS_WINDOW) / 2),
        );

        for (start_time, end_time) in pass_segments {
            let start_offset_hours = (start_time - current_time).as_seconds_f64() / SECS_PER_HOUR;
            let end_offset_hours = (end_time - current_time).as_seconds_f64() / SECS_PER_HOUR;

            // Convert to canvas coordinates
            let x1 = (Self::HOURS_WINDOW as f64 / 2.0) + start_offset_hours;
            let x2 = (Self::HOURS_WINDOW as f64 / 2.0) + end_offset_hours;

            debug_assert!(x2 >= 0.0 && x1 <= Self::HOURS_WINDOW as f64);
            ctx.draw(&canvas::Line {
                x1: x1.max(0.0),
                y1: 0.5,
                x2: x2.min(Self::HOURS_WINDOW as f64),
                y2: 0.5,
                color: Color::LightYellow,
            });
        }
    }

    fn render_canvas(&self, buf: &mut Buffer, state: &mut TimelineState) {
        Canvas::default()
            .x_bounds([0.0, Self::HOURS_WINDOW as f64])
            .y_bounds([0.0, 1.0])
            .paint(|ctx| {
                Self::draw_axis(ctx);
                ctx.layer();
                self.draw_pass_times(ctx, state);
                ctx.layer();
                Self::draw_hour_marks(ctx, state.time());
                ctx.layer();
                Self::draw_current_time_marker(ctx);
            })
            .render(state.inner_area, buf);
    }

    fn mouse_time(&self, state: &TimelineState) -> Option<Duration> {
        let mouse = state.mouse_position?;
        let duration = canvas_x_to_duration(area_to_canvas_x(state.inner_area, mouse));
        Some(duration)
    }
}

impl<'a> StatefulWidget for Timeline<'a> {
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
        Event::Key(event) => handle_key_event(event, app).await,
        Event::Mouse(event) => handle_mouse_event(event, app).await,
        _ => Ok(()),
    }
}

async fn handle_key_event(event: KeyEvent, app: &mut App) -> Result<()> {
    if let KeyCode::Char('r') = event.code {
        app.timeline_state.time_offset = chrono::Duration::zero()
    }

    Ok(())
}

async fn handle_mouse_event(event: MouseEvent, app: &mut App) -> Result<()> {
    let global_mouse = Position::new(event.column, event.row);
    let inner_area = app.timeline_state.inner_area;
    let Some(local_mouse) = window_to_area(global_mouse, inner_area) else {
        app.timeline_state.mouse_position = None;
        return Ok(());
    };
    app.timeline_state.mouse_position = Some(local_mouse);

    let duration = canvas_x_to_duration(area_to_canvas_x(inner_area, local_mouse));
    let time = app.timeline_state.time() + duration;

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            app.timeline_state.set_time(time);
        }
        MouseEventKind::ScrollUp => {
            app.timeline_state.rewind_time();
        }
        MouseEventKind::ScrollDown => {
            app.timeline_state.advance_time();
        }
        _ => {}
    }

    Ok(())
}

fn area_to_canvas_x(area: Rect, position: Position) -> f64 {
    (position.x as f64 + 0.5) / area.width as f64 * Timeline::HOURS_WINDOW as f64
}

fn canvas_x_to_duration(offset: f64) -> Duration {
    let hours_offset = offset - Timeline::HOURS_WINDOW as f64 / 2.0;
    Duration::seconds((hours_offset * SECS_PER_HOUR).round() as i64)
}
