use std::error;

use crate::components::{
    object_information::ObjectInformation, satellites::Satellites, track_map::TrackMap,
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub track_map: TrackMap,
    pub object_information: ObjectInformation,
    pub satellites: Satellites,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            track_map: TrackMap::default(),
            object_information: ObjectInformation::default(),
            satellites: Satellites::default(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
