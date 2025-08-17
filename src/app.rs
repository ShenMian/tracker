use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;

use crate::{
    config::Config,
    event::{Event, EventHandler},
    tui::Tui,
    widgets::{
        object_information::{self, ObjectInformation, ObjectInformationState},
        satellite_groups::{self, SatelliteGroups, SatelliteGroupsState},
        world_map::{self, WorldMap, WorldMapState},
    },
};

/// Application.
pub struct App {
    /// Indicates if the application is currently active and running. When set
    /// to false, triggers application shutdown.
    pub running: bool,

    pub world_map_state: WorldMapState,
    pub satellite_groups_state: SatelliteGroupsState,
    pub object_information_state: ObjectInformationState,

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
            world_map_state: WorldMapState::with_config(config.world_map)?,
            satellite_groups_state: SatelliteGroupsState::with_config(config.satellite_groups)?,
            object_information_state: Default::default(),
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
            let vertical = Layout::vertical([Constraint::Percentage(60), Constraint::Fill(1)]);
            let [top_right_area, right_bottom_area] = vertical.areas(right_area);

            let world_map = WorldMap {
                satellite_groups_state: &self.satellite_groups_state,
            };
            frame.render_stateful_widget(world_map, left_area, &mut self.world_map_state);

            let object_information = ObjectInformation {
                satellite_groups_state: &self.satellite_groups_state,
                world_map_state: &self.world_map_state,
            };
            frame.render_stateful_widget(
                object_information,
                top_right_area,
                &mut self.object_information_state,
            );

            frame.render_stateful_widget(
                SatelliteGroups,
                right_bottom_area,
                &mut self.satellite_groups_state,
            );
        })?;
        Ok(())
    }

    async fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Render => self.render()?,
            Event::Key(event) => self.handle_key_events(event),
            _ => {}
        }

        world_map::handle_event(event, self).await?;
        object_information::handle_event(event, self).await?;
        satellite_groups::handle_event(event, self).await
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
