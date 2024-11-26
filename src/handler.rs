use crossterm::event::{self, KeyCode, KeyEvent, MouseEvent};
use ratatui::layout::Position;

use crate::app::{App, AppResult};

pub fn handle_key_events(event: KeyEvent, app: &mut App) -> AppResult<()> {
    if event.code == KeyCode::Esc {
        app.quit();
    }
    Ok(())
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> AppResult<()> {
    if let event::MouseEventKind::Down(buttom) = event.kind {
        let area = app.track_map.area();
        if !area.contains(Position::new(event.column, event.row)) {
            return Ok(());
        }
        match buttom {
            event::MouseButton::Left => {
                let area = app.track_map.area();
                if !area.contains(Position::new(event.column, event.row)) {
                    return Ok(());
                }

                // Convert mouse coordinates to latitude and longitude
                let x = (event.column as f64 - area.left() as f64) / area.width as f64;
                let y = (event.row as f64 - area.top() as f64) / area.height as f64;
                let lon = -180.0 + x * 360.0;
                let lat = 90.0 - y * 180.0;

                // Find the nearest object
                if let Some((index, _)) = app.objects.iter().enumerate().min_by_key(|(_, obj)| {
                    let state = obj.predict(0.0).unwrap();
                    let dx = state.longitude() - lon;
                    let dy = state.latitude() - lat;
                    ((dx * dx + dy * dy) * 1000.0) as i32
                }) {
                    app.selected_object = Some(index);
                }
            }
            event::MouseButton::Right => {
                app.selected_object = None;
            }
            _ => {}
        }
    }

    Ok(())
}
