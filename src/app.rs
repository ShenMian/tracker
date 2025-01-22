use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::CrosstermBackend,
    Terminal,
};

use crate::{
    event::{Event, EventHandler},
    tui::Tui,
    widgets::{
        object_information::{self, ObjectInformation, ObjectInformationState},
        satellites::{self, Satellites, SatellitesState},
        world_map::{self, WorldMap, WorldMapState},
    },
};

/// Application.
pub struct App {
    /// Indicates if the application is currently active and running. When set to false, triggers application shutdown.
    pub running: bool,

    pub world_map_state: WorldMapState,
    pub satellites_state: SatellitesState,
    pub object_information_state: ObjectInformationState,

    tui: Tui<CrosstermBackend<std::io::Stdout>>,
}

impl App {
    /// Creates a new `App`.
    pub fn new() -> Result<Self> {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new();
        let tui = Tui::new(terminal, events);
        Ok(Self {
            running: true,
            world_map_state: Default::default(),
            satellites_state: Default::default(),
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
                satellites_state: &self.satellites_state,
            };
            frame.render_stateful_widget(world_map, left_area, &mut self.world_map_state);

            let object_information = ObjectInformation {
                satellites_state: &self.satellites_state,
                world_map_state: &self.world_map_state,
            };
            frame.render_stateful_widget(
                object_information,
                top_right_area,
                &mut self.object_information_state,
            );

            frame.render_stateful_widget(Satellites, right_bottom_area, &mut self.satellites_state);
        })?;
        Ok(())
    }

    /// Handle update events.
    pub async fn update(&mut self) {
        // Refresh satellite data every 2 minutes.
        const OBJECT_UPDATE_INTERVAL: Duration = Duration::from_secs(2 * 60);
        let now = Instant::now();
        if now.duration_since(self.satellites_state.last_object_update) >= OBJECT_UPDATE_INTERVAL {
            self.satellites_state.refresh_objects().await;
            self.satellites_state.last_object_update = now;
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
        Ok(())
    }

    async fn handle_mouse_events(&mut self, event: MouseEvent) -> Result<()> {
        world_map::handle_mouse_events(event, self).await?;
        object_information::handle_mouse_events(event, self).await?;
        satellites::handle_mouse_events(event, self).await?;
        Ok(())
    }
}
