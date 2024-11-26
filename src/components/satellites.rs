use std::cell::{Cell, RefCell};

use anyhow::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Margin, Position, Rect},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, List, ListItem, ListState},
    Frame,
};

use crate::satellite::Satellite;
use crate::{app::App, object::Object};

use super::Component;

pub struct Satellites {
    pub objects: Vec<Object>,

    pub items: Vec<Item>,
    pub state: RefCell<ListState>,

    area: Cell<Rect>,
}

pub struct Item {
    pub satellite: Satellite,
    selected: bool,
}

impl Satellites {
    fn update_objects(&mut self) {
        let mut objects = Vec::new();
        for item in &self.items {
            if !item.selected {
                continue;
            }
            for elements in item.satellite.get_elements() {
                objects.push(Object::from_elements(elements));
            }
        }
        self.objects = objects;
    }
}

impl Satellites {
    pub fn area(&self) -> Rect {
        self.area.get()
    }
}

impl Component for Satellites {
    fn render(&self, _app: &App, frame: &mut Frame, area: Rect) -> Result<()> {
        self.area.set(area);

        let items = self.items.iter().map(|item| {
            let style = match item.selected {
                true => Style::default().fg(Color::White),
                false => Style::default(),
            };
            let text: String = match item.selected {
                true => format!("✓ {}", item.satellite),
                false => format!("☐ {}", item.satellite),
            };
            ListItem::new(Text::styled(text, style))
        });

        let list = List::new(items)
            .block(Block::bordered().title("Satellites".blue()))
            .highlight_style(Style::default().bold().bg(Color::Blue));

        frame.render_stateful_widget(list, area, &mut self.state.borrow_mut());

        Ok(())
    }
}

impl Default for Satellites {
    fn default() -> Self {
        let mut items = Vec::new();
        for satellite in [
            Satellite::Css,
            Satellite::Iss,
            Satellite::Weather,
            Satellite::NOAA,
            Satellite::GOES,
            Satellite::EarthResources,
            Satellite::SearchRescue,
            Satellite::DisasterMonitoring,
            Satellite::Gps,
            Satellite::Glonass,
            Satellite::Galileo,
            Satellite::Beidou,
            Satellite::SpaceEarthScience,
            Satellite::Geodetic,
            Satellite::Engineering,
            Satellite::Education,
            Satellite::Dfh1,
            Satellite::Military,
            Satellite::RadarCalibration,
            Satellite::CubeSats,
        ] {
            items.push(Item {
                satellite,
                selected: false,
            });
        }
        let mut instance = Self {
            objects: Vec::new(),
            items,
            state: Default::default(),
            area: Default::default(),
        };
        instance.update_objects();
        instance
    }
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.satellites.area().inner(Margin::new(1, 1));
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.satellites.state.get_mut().select(None);
        return Ok(());
    }

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let index = app.satellites.state.get_mut().selected().unwrap();
            app.satellites.items[index].selected = !app.satellites.items[index].selected;
            app.track_map.selected_object = None;
            app.satellites.update_objects();
        }
        MouseEventKind::ScrollDown => {
            let max_offset = app
                .satellites
                .items
                .len()
                .saturating_sub(inner_area.height as usize);
            *app.satellites.state.get_mut().offset_mut() =
                (app.satellites.state.get_mut().offset() + 1).min(max_offset);
        }
        MouseEventKind::ScrollUp => {
            *app.satellites.state.get_mut().offset_mut() =
                app.satellites.state.get_mut().offset().saturating_sub(1);
        }
        _ => {}
    }
    let index = (event.row - inner_area.y) as usize + app.satellites.state.get_mut().offset();
    app.satellites.state.get_mut().select(Some(index));

    Ok(())
}
