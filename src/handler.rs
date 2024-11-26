use crossterm::event::{KeyCode, KeyEvent, MouseEvent};

use crate::{
    app::{App, AppResult},
    components::{satellites, track_map},
};

pub fn handle_key_events(event: KeyEvent, app: &mut App) -> AppResult<()> {
    if event.code == KeyCode::Esc {
        app.quit();
    }
    Ok(())
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> AppResult<()> {
    track_map::handle_mouse_events(event, app).unwrap();
    satellites::handle_mouse_events(event, app).unwrap();
    Ok(())
}
