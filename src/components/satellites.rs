use std::cell::{Cell, RefCell};

use anyhow::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Margin, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarState},
    Frame,
};

use crate::satellite::Satellite;
use crate::{app::App, object::Object};

use super::Component;

pub struct Item {
    pub satellite: Satellite,
    selected: bool,
}

impl From<Satellite> for Item {
    fn from(satellite: Satellite) -> Self {
        Self {
            satellite,
            selected: false,
        }
    }
}

pub struct Satellites {
    pub objects: Vec<Object>,

    pub items: Vec<Item>,
    pub list_state: RefCell<ListState>,

    area: Cell<Rect>,
}

impl Satellites {
    pub fn update_objects(&mut self) {
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
            let style = if item.selected {
                Style::default().fg(Color::White)
            } else {
                Style::default()
            };
            let text: String = if item.selected {
                format!("✓ {}", item.satellite)
            } else {
                format!("☐ {}", item.satellite)
            };
            ListItem::new(Text::styled(text, style))
        });

        let list = List::new(items)
            .block(Block::bordered().title("Satellites".blue()))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_stateful_widget(list, area, &mut self.list_state.borrow_mut());

        let inner_area = area.inner(Margin::new(0, 1));
        frame.render_stateful_widget(
            Scrollbar::default(),
            inner_area,
            &mut ScrollbarState::new(self.items.len() - inner_area.height as usize)
                .position(self.list_state.borrow().offset()),
        );

        Ok(())
    }
}

impl Default for Satellites {
    fn default() -> Self {
        let satellites = [
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
        ];
        Self {
            objects: Vec::new(),
            items: satellites.into_iter().map(Item::from).collect(),
            list_state: Default::default(),
            area: Default::default(),
        }
    }
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.satellites.area().inner(Margin::new(1, 1));
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.satellites.list_state.get_mut().select(None);
        return Ok(());
    }

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(index) = app.satellites.list_state.get_mut().selected() {
                app.satellites.items[index].selected = !app.satellites.items[index].selected;
                app.track_map.selected_object = None;
                app.satellites.update_objects();
            }
        }
        MouseEventKind::ScrollDown => {
            let max_offset = app
                .satellites
                .items
                .len()
                .saturating_sub(inner_area.height as usize);
            *app.satellites.list_state.get_mut().offset_mut() =
                (app.satellites.list_state.get_mut().offset() + 1).min(max_offset);
        }
        MouseEventKind::ScrollUp => {
            *app.satellites.list_state.get_mut().offset_mut() = app
                .satellites
                .list_state
                .get_mut()
                .offset()
                .saturating_sub(1);
        }
        _ => {}
    }
    let index = (event.row - inner_area.y) as usize + app.satellites.list_state.get_mut().offset();
    app.satellites.list_state.get_mut().select(Some(index));

    Ok(())
}
