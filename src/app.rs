use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
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
    /// Indicates if the application is currently active and running. When set to false, triggers application shutdown.
    pub running: bool,

    pub world_map_state: WorldMapState,
    pub satellite_groups_state: SatelliteGroupsState,
    pub object_information_state: ObjectInformationState,

    tui: Tui<CrosstermBackend<std::io::Stdout>>,
}

impl App {
    /// Creates a new `App` with the configuration.
    pub fn with_config(config: Config) -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new();
        let tui = Tui::new(terminal, events);
        Ok(Self {
            running: true,
            world_map_state: WorldMapState::with_config(config.world_map),
            satellite_groups_state: SatelliteGroupsState::with_config(config.satellite_groups),
            object_information_state: Default::default(),
            tui,
        })
    }

    /// Runs the main loop of the application.
    pub async fn run(&mut self) -> Result<()> {
        self.tui.init()?;

        // Start the main loop.
        while self.running {
            // Handle events.
            match self.tui.events.next().await? {
                Event::Update => self.update().await,
                Event::Render => self.render()?,
                Event::Key(event) => self.handle_key_events(event).await?,
                Event::Mouse(event) => self.handle_mouse_events(event).await?,
            }
        }

        self.tui.deinit()
    }

    /// Renders the terminal interface.
    pub fn render(&mut self) -> Result<()> {
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

    /// Handle update events.
    pub async fn update(&mut self) {
        // Refresh satellite data every 2 minutes.
        const OBJECT_UPDATE_INTERVAL: Duration = Duration::from_secs(2 * 60);
        let now = Instant::now();
        if now.duration_since(self.satellite_groups_state.last_object_update)
            >= OBJECT_UPDATE_INTERVAL
        {
            self.satellite_groups_state.refresh_objects().await;
            self.satellite_groups_state.last_object_update = now;
        }
    }

    /// Set running to false to quit the application.
    pub fn request_exit(&mut self) {
        self.running = false;
    }

    async fn handle_key_events(&mut self, event: KeyEvent) -> Result<()> {
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
        world_map::handle_key_events(event, self).await?;
        Ok(())
    }

    async fn handle_mouse_events(&mut self, event: MouseEvent) -> Result<()> {
        world_map::handle_mouse_events(event, self).await?;
        object_information::handle_mouse_events(event, self).await?;
        satellite_groups::handle_mouse_events(event, self).await?;
        Ok(())
    }
}
