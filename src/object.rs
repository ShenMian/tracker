use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct Object {
    name: String,
    cospar_id: String,
    norad_id: u64,

    epoch: DateTime<Utc>,
    inclination: f64,
    right_ascension: f64,
    eccentricity: f64,
    argument_of_perigee: f64,
    mean_anomaly: f64,
    mean_motion: f64,
    revolution_number: u64,

    constants: sgp4::Constants,
}

impl Object {
    pub fn from_elements(elements: sgp4::Elements) -> Self {
        Self {
            name: elements.object_name.as_ref().unwrap().clone(),
            cospar_id: elements.international_designator.as_ref().unwrap().clone(),
            norad_id: elements.norad_id,
            epoch: DateTime::from_naive_utc_and_offset(elements.datetime, Utc),
            inclination: elements.inclination,
            right_ascension: elements.right_ascension,
            eccentricity: elements.eccentricity,
            argument_of_perigee: elements.argument_of_perigee,
            mean_anomaly: elements.mean_anomaly,
            mean_motion: elements.mean_motion,
            revolution_number: elements.revolution_number,
            constants: sgp4::Constants::from_elements(&elements).unwrap(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn cospar_id(&self) -> &String {
        &self.cospar_id
    }

    pub fn norad_id(&self) -> u64 {
        self.norad_id
    }

    pub fn epoch(&self) -> DateTime<Utc> {
        self.epoch
    }

    pub fn inclination(&self) -> f64 {
        self.inclination
    }

    pub fn right_ascension(&self) -> f64 {
        self.right_ascension
    }

    pub fn eccentricity(&self) -> f64 {
        self.eccentricity
    }

    pub fn argument_of_perigee(&self) -> f64 {
        self.argument_of_perigee
    }

    pub fn mean_anomaly(&self) -> f64 {
        self.mean_anomaly
    }

    pub fn mean_motion(&self) -> f64 {
        self.mean_motion
    }

    pub fn revolution_number(&self) -> u64 {
        self.revolution_number
    }

    pub fn orbital_period(&self) -> chrono::Duration {
        const SECONDS_PER_DAY: f64 = 24.0 * 60.0 * 60.0;
        chrono::Duration::seconds((SECONDS_PER_DAY / self.mean_motion) as i64)
    }

    pub fn predict(&self, time: DateTime<Utc>) -> Result<State, sgp4::Error> {
        let minutes_since_epoch = (time - self.epoch).num_seconds() as f64 / 60.0;

        let prediction = self
            .constants
            .propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch))?;

        let [lon, lat, alt] = ecef_to_geodetic(prediction.position);

        assert!((-90.0..=90.0).contains(&lat), "Latitude out of range");
        assert!((-180.0..=180.0).contains(&lon), "Longitude out of range");

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
