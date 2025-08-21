use std::fmt::{Display, Formatter};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::Block};
use rust_i18n::t;

use crate::{
    app::App,
    event::Event,
    widgets::{
        object_information::{ObjectInformation, ObjectInformationState},
        satellite_groups::SatelliteGroupsState,
        sky::{Sky, SkyState},
        world_map::WorldMapState,
    },
};

/// Tabs enum for the right-side panel.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Info,
    Sky,
}

impl Tab {
    /// Returns an iterator over all tabs.
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Info, Self::Sky].into_iter()
    }

    /// Returns the next tab.
    fn next(&self) -> Self {
        match self {
            Tab::Sky => Tab::Info,
            Tab::Info => Tab::Sky,
        }
    }

    /// Returns the previous tab.
    fn previous(&self) -> Self {
        self.next()
    }
}

impl Display for Tab {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Info => write!(f, "{}", t!("oi.title")),
            Tab::Sky => write!(f, "{}", t!("sky.title")),
        }
    }
}

pub struct Tabs<'a> {
    pub world_map_state: &'a WorldMapState,
    pub satellite_groups_state: &'a SatelliteGroupsState,
    pub sky_state: &'a mut SkyState,
    pub object_information_state: &'a mut ObjectInformationState,
}

#[derive(Default)]
pub struct TabsState {
    pub selected: Tab,
}

impl Tabs<'_> {
    fn block(state: &TabsState) -> Block<'static> {
        let mut block = Block::bordered();
        for tab in Tab::iter() {
            if tab == state.selected {
                block = block.title(tab.to_string().blue());
            } else {
                block = block.title(tab.to_string());
            }
        }
        block
    }

    fn render_tab(self, area: Rect, buf: &mut Buffer, state: &TabsState) {
        match state.selected {
            Tab::Sky => {
                let sky = Sky {
                    world_map_state: self.world_map_state,
                    satellite_groups_state: self.satellite_groups_state,
                };
                sky.render(area, buf, self.sky_state);
            }
            Tab::Info => {
                let object_information = ObjectInformation {
                    satellite_groups_state: self.satellite_groups_state,
                    world_map_state: self.world_map_state,
                };
                object_information.render(area, buf, self.object_information_state);
            }
        }
    }
}

impl StatefulWidget for Tabs<'_> {
    type State = TabsState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Self::block(state);
        let inner_area = block.inner(area);
        block.render(area, buf);

        self.render_tab(inner_area, buf, state);
    }
}

pub async fn handle_event(event: Event, app: &mut App) -> Result<()> {
    match event {
        Event::Key(event) => handle_key_event(event, app).await,
        _ => Ok(()),
    }
}

async fn handle_key_event(event: KeyEvent, app: &mut App) -> Result<()> {
    let state = &mut app.tab_state;

    if event.code == KeyCode::Tab {
        if event.modifiers == KeyModifiers::SHIFT {
            state.selected = state.selected.previous();
        } else {
            state.selected = state.selected.next();
        }
    }

    Ok(())
}
