use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;

use crate::{
    config::Config,
    event::{Event, EventHandler},
    tui::Tui,
    widgets::{
        information::{self, InformationState},
        satellite_groups::{self, SatelliteGroups, SatelliteGroupsState},
        sky::{self, SkyState},
        tabs::{self, Tabs, TabsState},
        timeline::{self, Timeline, TimelineState},
        world_map::{self, WorldMap, WorldMapState},
    },
};

/// Application.
pub struct App {
    /// Indicates if the application is currently active and running. When set
    /// to false, triggers application shutdown.
    pub running: bool,
    pub states: States,
    tui: Tui<CrosstermBackend<std::io::Stdout>>,
}

impl App {
    /// Creates a new `App` with the given configuration.
    pub fn with_config(config: Config) -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new();
        let tui = Tui::new(terminal, events);
        Ok(Self {
            running: true,
            states: States::with_config(config),
            tui,
        })
    }

    /// Runs the main loop of the application.
    pub async fn run(&mut self) -> Result<()> {
        self.tui.init()?;

        // The main loop.
        while self.running {
            let event = self.tui.events.next().await?;
            self.handle_event(event).await?;
        }

        self.tui.deinit()
    }

    /// Set running to false to quit the application.
    fn request_exit(&mut self) {
        self.running = false;
    }

    /// Renders the terminal interface.
    fn render(&mut self) -> Result<()> {
        self.tui.terminal.draw(|frame| {
            let horizontal = Layout::horizontal([Constraint::Percentage(80), Constraint::Min(25)]);
            let [left_area, right_area] = horizontal.areas(frame.area());

            let left_vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(3)]);
            let [left_top_area, left_bottom_area] = left_vertical.areas(left_area);

            WorldMap {
                state: &mut self.states.world_map_state,
                satellite_groups_state: &self.states.satellite_groups_state,
                sky_state: &self.states.sky_state,
                timeline_state: &self.states.timeline_state,
            }
            .render(left_top_area, frame.buffer_mut());

            Timeline {
                state: &mut self.states.timeline_state,
                world_map_state: &self.states.world_map_state,
                sky_state: &self.states.sky_state,
            }
            .render(left_bottom_area, frame.buffer_mut());

            let vertical = Layout::vertical([Constraint::Percentage(60), Constraint::Fill(1)]);
            let [right_top_area, right_bottom_area] = vertical.areas(right_area);

            Tabs {
                state: &mut self.states.tab_state,
                world_map_state: &self.states.world_map_state,
                sky_state: &mut self.states.sky_state,
                information_state: &mut self.states.information_state,
                timeline_state: &self.states.timeline_state,
            }
            .render(right_top_area, frame.buffer_mut());

            SatelliteGroups {
                state: &mut self.states.satellite_groups_state,
            }
            .render(right_bottom_area, frame.buffer_mut());
        })?;
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Render => self.render()?,
            Event::Key(event) => self.handle_key_events(event),
            _ => {}
        }

        world_map::handle_event(event, &mut self.states).await?;
        satellite_groups::handle_event(event, &mut self.states).await?;
        tabs::handle_event(event, &mut self.states).await?;
        information::handle_event(event, &mut self.states).await?;
        sky::handle_event(event, &mut self.states).await?;
        timeline::handle_event(event, &mut self.states).await
    }

    fn handle_key_events(&mut self, event: KeyEvent) {
        match event.code {
            // Exit application on `Q` or `ESC`.
            KeyCode::Char('q') | KeyCode::Esc => {
                self.request_exit();
            }
            // Exit application on `Ctrl-C`.
            KeyCode::Char('c') => {
                if event.modifiers == KeyModifiers::CONTROL {
                    self.request_exit();
                }
            }
            _ => {}
        }
    }
}

pub struct States {
    pub world_map_state: WorldMapState,
    pub satellite_groups_state: SatelliteGroupsState,
    pub tab_state: TabsState,
    pub information_state: InformationState,
    pub sky_state: SkyState,
    pub timeline_state: TimelineState,
}

impl States {
    pub fn with_config(config: Config) -> Self {
        Self {
            world_map_state: WorldMapState::with_config(config.world_map),
            satellite_groups_state: SatelliteGroupsState::with_config(config.satellite_groups),
            tab_state: Default::default(),
            information_state: Default::default(),
            sky_state: SkyState::with_config(config.sky),
            timeline_state: TimelineState::with_config(config.timeline),
        }
    }
}
