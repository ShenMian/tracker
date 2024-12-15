use std::time::{Duration, Instant};

use anyhow::{Ok, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use ratatui::{
    layout::{Constraint, Layout},
    prelude::CrosstermBackend,
    style::Color,
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
    /// Constructs a new instance of [`App`].
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
                Event::Key(event) => handle_key_events(event, self).await?,
                Event::Mouse(event) => handle_mouse_events(event, self).await?,
            }
        }

        self.tui.deinit()
    }

    /// Renders the terminal interface.
    pub fn render(&mut self) -> Result<()> {
        self.tui.terminal.draw(|frame| {
            let horizontal = Layout::horizontal([Constraint::Percentage(80), Constraint::Min(25)]);
            let [left, right] = horizontal.areas(frame.area());
            let vertical = Layout::vertical([Constraint::Percentage(60), Constraint::Fill(1)]);
            let [top_right, bottom_right] = vertical.areas(right);

            let world_map = WorldMap {
                satellites_state: &self.satellites_state,
                satellit_symbol: "+".to_string(),
                trajectory_color: Color::LightBlue,
            };
            frame.render_stateful_widget(world_map, left, &mut self.world_map_state);

            let object_information = ObjectInformation {
                satellites_state: &self.satellites_state,
                world_map_state: &self.world_map_state,
            };
            frame.render_stateful_widget(
                object_information,
                top_right,
                &mut self.object_information_state,
            );

            frame.render_stateful_widget(Satellites, bottom_right, &mut self.satellites_state);
        })?;
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub async fn update(&mut self) {
        const OBJECT_UPDATE_INTERVAL: Duration = Duration::from_secs(2 * 60);

        let now = Instant::now();
        if now.duration_since(self.satellites_state.last_object_update) >= OBJECT_UPDATE_INTERVAL {
            self.satellites_state.refresh_objects().await;
            self.satellites_state.last_object_update = now;
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

async fn handle_key_events(event: KeyEvent, app: &mut App) -> Result<()> {
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

async fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    world_map::handle_mouse_events(event, app).await?;
    object_information::handle_mouse_events(event, app).await?;
    satellites::handle_mouse_events(event, app).await?;
    Ok(())
}
