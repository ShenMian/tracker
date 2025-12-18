use anyhow::Result;
use chrono::{DateTime, Duration, Local, TimeZone, Timelike, Utc};
use crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    prelude::*,
    widgets::{
        Block, Borders,
        canvas::{self, Canvas, Context},
    },
};

use crate::{
    app::States, config::TimelineConfig, event::Event, shared_state::SharedState,
    utils::calculate_pass_times, widgets::window_to_area,
};

const SECS_PER_HOUR: f64 = 3600.0;

pub struct Timeline<'a> {
    pub state: &'a mut TimelineState,
    pub shared: &'a SharedState,
}

#[derive(Default)]
pub struct TimelineState {
    /// Current mouse position within the widget's area.
    mouse_position: Option<Position>,
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

    fn hovered_time(&self, current_time: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let mouse = self.mouse_position?;
        Some(canvas_x_to_time(
            area_to_canvas_x(self.inner_area, mouse),
            current_time,
        ))
    }
}

impl Widget for Timeline<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = self.block();
        self.state.inner_area = block.inner(area);
        block.render(area, buf);

        self.render_canvas(buf);
    }
}

impl Timeline<'_> {
    const HOURS_WINDOW: i64 = 8;

    fn block(&self) -> Block<'static> {
        let current_time = self.shared.time.time();
        let mut block = Block::new()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
            .title_bottom(
                format!(
                    "{} ({:+}m)",
                    current_time
                        .with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S"),
                    self.shared.time.time_offset().num_minutes()
                )
                .white(),
            );

        if let Some(time) = self.state.hovered_time(current_time) {
            let label = time
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            block = block.title_bottom(Line::from(label).right_aligned());
        }

        block
    }

    fn render_canvas(&self, buf: &mut Buffer) {
        Canvas::default()
            .x_bounds([0.0, Self::HOURS_WINDOW as f64])
            .y_bounds([0.0, 1.0])
            .paint(|ctx| {
                Self::draw_axis(ctx);
                ctx.layer();
                self.draw_pass_times(ctx);
                ctx.layer();
                self.draw_hour_marks(ctx);
                ctx.layer();
                Self::draw_current_time_marker(ctx);
            })
            .render(self.state.inner_area, buf);
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

    fn draw_hour_marks(&self, ctx: &mut Context) {
        let current_time = self.shared.time.time();
        let minutes = Duration::minutes(current_time.minute() as i64);
        for hour_offset in
            ((-Self::HOURS_WINDOW / 2)..=(Self::HOURS_WINDOW / 2)).map(Duration::hours)
        {
            let mark_time = current_time + hour_offset - minutes;
            let x = time_to_canvas_x(mark_time, current_time);

            ctx.draw(&canvas::Line {
                x1: x,
                y1: 0.5,
                x2: x,
                y2: 0.5,
                color: Color::White,
            });

            let hours = mark_time.with_timezone(&Local).hour() % 24;
            ctx.print(x, 0.0, format!("{hours:02}").fg(Color::DarkGray));
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

    fn draw_pass_times(&self, ctx: &mut Context) {
        let Some(selected_object) = &self.shared.selected_object else {
            return;
        };
        let Some(ground_station) = &self.shared.ground_station else {
            return;
        };

        let current_time = self.shared.time.time();
        let pass_segments = calculate_pass_times(
            selected_object,
            &ground_station.position,
            &(current_time - Duration::hours(Self::HOURS_WINDOW) / 2),
            &(current_time + Duration::hours(Self::HOURS_WINDOW) / 2),
        );

        for (start_time, end_time) in pass_segments {
            let x1 = time_to_canvas_x(start_time, current_time).max(0.0);
            let x2 = time_to_canvas_x(end_time, current_time).min(Self::HOURS_WINDOW as f64);

            debug_assert!(x2 >= 0.0 && x1 <= Self::HOURS_WINDOW as f64);
            ctx.draw(&canvas::Line {
                x1,
                y1: 0.5,
                x2,
                y2: 0.5,
                color: Color::LightYellow,
            });
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
    if let KeyCode::Char('r') = event.code {
        states.shared.time.set_time_offset(chrono::Duration::zero())
    }

    Ok(())
}

fn handle_mouse_event(event: MouseEvent, states: &mut States) -> Result<()> {
    let state = &mut states.timeline_state;
    let shared = &mut states.shared;

    let global_mouse = Position::new(event.column, event.row);
    let inner_area = state.inner_area;
    let Some(local_mouse) = window_to_area(global_mouse, inner_area) else {
        state.mouse_position = None;
        return Ok(());
    };
    state.mouse_position = Some(local_mouse);

    let time = canvas_x_to_time(
        area_to_canvas_x(inner_area, local_mouse),
        shared.time.time(),
    );

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            shared.time.set_time(time);
        }
        MouseEventKind::ScrollUp => {
            shared.time.rewind_time(state.time_delta);
        }
        MouseEventKind::ScrollDown => {
            shared.time.advance_time(state.time_delta);
        }
        _ => {}
    }

    Ok(())
}

/// Converts a position within a rectangular area to a canvas x-coordinate.
fn area_to_canvas_x(area: Rect, position: Position) -> f64 {
    (position.x as f64 + 0.5) / area.width as f64 * Timeline::HOURS_WINDOW as f64
}

/// Converts a time to a canvas x-coordinate relative to a reference time.
fn time_to_canvas_x<Tz: TimeZone>(time: DateTime<Tz>, reference: DateTime<Tz>) -> f64 {
    let hours_offset = (time - reference).as_seconds_f64() / SECS_PER_HOUR;
    Timeline::HOURS_WINDOW as f64 / 2.0 + hours_offset
}

/// Converts a canvas x-coordinate to a time relative to a reference time.
fn canvas_x_to_time<Tz: TimeZone>(x: f64, reference: DateTime<Tz>) -> DateTime<Tz> {
    let hours_offset = x - Timeline::HOURS_WINDOW as f64 / 2.0;
    reference + Duration::seconds((hours_offset * SECS_PER_HOUR).round() as i64)
}
