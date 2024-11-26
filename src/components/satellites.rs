use std::cell::RefCell;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

use crate::satellite::Satellite;
use crate::{app::App, object::Object};

use super::Component;

pub struct Satellites {
    pub items: Vec<Item>,
    pub state: RefCell<ListState>,
}

pub struct Item {
    pub satellite: Satellite,
    selected: bool,
}

impl Default for Satellites {
    fn default() -> Self {
        let mut items = Vec::new();
        for satellite in [
            Satellite::Beidou,
            Satellite::Galileo,
            Satellite::Glonass,
            Satellite::Gps,
            Satellite::Css,
            Satellite::Iss,
            Satellite::Dfh1,
        ] {
            items.push(Item {
                satellite,
                selected: false,
            });
        }
        Self {
            items,
            state: Default::default(),
        }
    }
}

impl Component for Satellites {
    fn render(&self, _app: &App, frame: &mut Frame, area: Rect) -> Result<()> {
        let items = self.items.iter().map(|item| {
            let style = match item.selected {
                true => Style::default().fg(Color::White),
                false => Style::default(),
            };
            let text: String = match item.selected {
                true => format!("☑ {}", item.satellite),
                false => format!("☐ {}", item.satellite),
            };
            ListItem::new(Text::styled(text, style))
        });

        let list = List::new(items)
            .block(Block::bordered().title("Satellites".cyan()))
            .highlight_style(Style::default().bg(Color::Blue))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut self.state.borrow_mut());
        Ok(())
    }
}

pub fn handle_key_events(event: KeyEvent, app: &mut App) -> Result<()> {
    match event.code {
        KeyCode::Up | KeyCode::Char('k') => app.satellites.state.get_mut().select_previous(),
        KeyCode::Down | KeyCode::Char('j') => app.satellites.state.get_mut().select_next(),
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            if let Some(index) = app.satellites.state.get_mut().selected() {
                app.satellites.items[index].selected = !app.satellites.items[index].selected;
                update_objects(app);
            }
        }
        KeyCode::Left | KeyCode::Char('h') => app.satellites.state.get_mut().select(None),
        _ => {}
    }
    Ok(())
}

fn update_objects(app: &mut App) {
    let mut objects = Vec::new();
    for item in &app.satellites.items {
        if !item.selected {
            continue;
        }
        for elements in item.satellite.get_elements() {
            objects.push(Object::from_elements(elements));
        }
    }
    app.selected_object = None;
    app.objects = objects;
}
