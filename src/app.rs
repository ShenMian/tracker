use std::error;

use crate::{
    components::{object_information::ObjectInformation, track_map::TrackMap},
    object::Object,
    satellites::Satellites,
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub objects: Vec<Object>,
    pub selected_object: Option<usize>,

    pub track_map: TrackMap,
    pub object_information: ObjectInformation,
}

impl Default for App {
    fn default() -> Self {
        let mut objects = Vec::new();
        for elements in Satellites::Beidou.get_elements() {
            objects.push(Object::from_elements(elements));
        }
        for elements in Satellites::Css.get_elements() {
            objects.push(Object::from_elements(elements));
        }
        for elements in Satellites::Iss.get_elements() {
            objects.push(Object::from_elements(elements));
        }

        Self {
            running: true,
            objects,
            selected_object: None,
            track_map: TrackMap::default(),
            object_information: ObjectInformation::default(),
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
