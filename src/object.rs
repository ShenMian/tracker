use chrono::{NaiveDateTime, Utc};

#[derive(Clone, Debug)]
pub struct Object {
    name: String,
    id: String,

    datetime: NaiveDateTime,
    constants: sgp4::Constants,
}

impl Object {
    pub fn new(
        name: String,
        id: String,
        datetime: NaiveDateTime,
        constants: sgp4::Constants,
    ) -> Self {
        Self {
            name,
            id,
            datetime,
            constants,
        }
    }

    pub fn from_elements(elements: sgp4::Elements) -> Self {
        Self::new(
            elements.object_name.as_ref().unwrap().clone(),
            elements.international_designator.as_ref().unwrap().clone(),
            elements.datetime,
            sgp4::Constants::from_elements(&elements).unwrap(),
        )
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn predict(&self, minutes: f64) -> Result<State, sgp4::Error> {
        let current_time = Utc::now().naive_utc();
        let time_since_epoch = (current_time - self.datetime).num_seconds() as f64;
        let prediction = self.constants.propagate(sgp4::MinutesSinceEpoch(
            (time_since_epoch + minutes * 60.0) / 60.0,
        ))?;
        let [lat, lon, alt] = ecef_to_geodetic(prediction.position);
        Ok(State {
            position: [lon, lat, alt],
            velocity: prediction.velocity,
        })
    }
}

#[derive(Clone, Debug)]
pub struct State {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
}

impl State {
    pub fn latitude(&self) -> f64 {
        self.position[1]
    }

    pub fn longitude(&self) -> f64 {
        self.position[0]
    }

    pub fn altitude(&self) -> f64 {
        self.position[2]
    }

    pub fn speed(&self) -> f64 {
        (self.velocity[0].powi(2) + self.velocity[1].powi(2) + self.velocity[2].powi(2)).sqrt()
    }
}

fn ecef_to_geodetic(position: [f64; 3]) -> [f64; 3] {
    let [x, y, z] = position;

    let a = 6378.137; // Earth's semi-major axis (unit: km)
    let e2 = 0.00669437999014; // Square of Earth's ellipsoid eccentricity

    let b = (x * x + y * y).sqrt();
    let mut lat = (z / b).atan(); // Initial latitude

    // Iterate to calculate latitude
    for _ in 0..5 {
        let sin_lat = lat.sin();
        let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();
        lat = ((z + e2 * n * sin_lat) / b).atan();
    }

    let lon = y.atan2(x); // Longitude

    // Calculate altitude
    let n = a / (1.0 - e2 * lat.sin().powi(2)).sqrt();
    let alt = b / lat.cos() - n;

    [lat.to_degrees(), lon.to_degrees(), alt]
}
