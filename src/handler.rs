use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::{
    app::{App, AppResult},
    components::{object_information, satellites, track_map},
};

pub fn handle_key_events(event: KeyEvent, app: &mut App) -> AppResult<()> {
    match event.code {
        // Exit application on `ESC`
        KeyCode::Esc => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') => {
            if event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> AppResult<()> {
    track_map::handle_mouse_events(event, app).unwrap();
    object_information::handle_mouse_events(event, app).unwrap();
    satellites::handle_mouse_events(event, app).unwrap();
    Ok(())
}
