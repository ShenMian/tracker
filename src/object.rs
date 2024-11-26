use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct Object {
    name: String,
    id: String,

    epoch: DateTime<Utc>,

    constants: sgp4::Constants,
    mean_motion: f64,
}

impl Object {
    pub fn new(
        name: String,
        id: String,
        epoch: DateTime<Utc>,
        constants: sgp4::Constants,
        mean_motion: f64,
    ) -> Self {
        Self {
            name,
            id,
            epoch,
            constants,
            mean_motion,
        }
    }

    pub fn from_elements(elements: sgp4::Elements) -> Self {
        Self::new(
            elements.object_name.as_ref().unwrap().clone(),
            elements.international_designator.as_ref().unwrap().clone(),
            DateTime::from_naive_utc_and_offset(elements.datetime, Utc),
            sgp4::Constants::from_elements(&elements).unwrap(),
            elements.mean_motion,
        )
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn predict(&self, time: DateTime<Utc>) -> Result<State, sgp4::Error> {
        let time_since_epoch = time - self.epoch;
        let prediction = self.constants.propagate(sgp4::MinutesSinceEpoch(
            time_since_epoch.num_seconds() as f64 / 60.0,
        ))?;
        let [lon, lat, alt] = ecef_to_geodetic(prediction.position);
        assert!((-90.0..=90.0).contains(&lat));
        assert!((-180.0..=180.0).contains(&lon));
        Ok(State {
            position: [lon, lat, alt],
            velocity: prediction.velocity,
        })
    }

    pub fn orbital_period(&self) -> chrono::Duration {
        const SECONDS_PER_DAY: f64 = 24.0 * 60.0 * 60.0;
        chrono::Duration::seconds((SECONDS_PER_DAY / self.mean_motion) as i64)
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

/// Converts ECEF coordinates to geodetic coordinates.
fn ecef_to_geodetic(position: [f64; 3]) -> [f64; 3] {
    let [x, y, z] = position;

    let a = 6378.137; // Earth's semi-major axis (km)
    let e2 = 0.00669437999014; // Square of Earth's ellipsoid eccentricity

    let b = (x * x + y * y).sqrt();
    let mut lat = (z / b).atan(); // Initial latitude

    // Iterate to calculate latitude
    for _ in 0..5 {
        let sin_lat = lat.sin();
        let n = a / (1.0 - e2 * sin_lat * sin_lat).sqrt();
        lat = ((z + e2 * n * sin_lat) / b).atan();
    }

    let lon = y.atan2(x);

    // Calculate altitude
    let n = a / (1.0 - e2 * lat.sin().powi(2)).sqrt();
    let alt = b / lat.cos() - n;

    [lon.to_degrees(), lat.to_degrees(), alt]
}
