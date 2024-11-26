use std::f64::consts::PI;

use chrono::{DateTime, Datelike, Timelike, Utc};

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

        let gmst = calculate_gmst(calculate_julian_date(time));
        let (lat, lon, alt) = ecef_to_lat_lon_alt(teme_to_ecef(prediction.position, gmst));

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

/// Returns the Julian date for the given UTC datetime.
fn calculate_julian_date(datetime: DateTime<Utc>) -> f64 {
    let year = datetime.year();
    let month = datetime.month() as i32;
    let day = datetime.day() as i32;
    let hour = datetime.hour() as f64
        + datetime.minute() as f64 / 60.0
        + datetime.second() as f64 / 3600.0;

    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };

    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    let jd = (365.25 * (y as f64 + 4716.0)).floor()
        + (30.6001 * (m as f64 + 1.0)).floor()
        + day as f64
        + hour / 24.0
        - 1524.5
        + b;
    jd
}

/// Calculates the Greenwich Mean Sidereal Time (GMST) in radians.
///
/// # Arguments
///
/// * `julian_date` - The Julian date for which to calculate GMST
///
/// # Returns
///
/// The GMST in radians, normalized to [0, 2π]
///
/// # Formula
///
/// Uses the algorithm from the Astronomical Algorithms by Jean Meeus:
/// GMST = 280.46061837° + 360.98564736629° * (JD - 2451545.0)
///        + 0.000387933° * T² - T³/38710000°
/// where T is the number of Julian centuries since J2000.0
fn calculate_gmst(julian_date: f64) -> f64 {
    let jd2000 = 2451545.0;
    let t = (julian_date - jd2000) / 36525.0;

    let gmst = 280.46061837 + 360.98564736629 * (julian_date - jd2000) + 0.000387933 * t * t
        - t * t * t / 38710000.0;

    // Normalize to [0, 2π]
    (gmst % 360.0).to_radians().rem_euclid(2.0 * PI)
}

/// Converts a position vector in the True Equator Mean Equinox (TEME) frame to the Earth-Centered
fn teme_to_ecef(position: [f64; 3], gmst: f64) -> [f64; 3] {
    let (x, y, z) = (position[0], position[1], position[2]);
    let cos_gmst = gmst.cos();
    let sin_gmst = gmst.sin();

    let x_ecef = cos_gmst * x + sin_gmst * y;
    let y_ecef = -sin_gmst * x + cos_gmst * y;
    [x_ecef, y_ecef, z]
}

/// Converts a position vector in the Earth-Centered Earth-Fixed (ECEF) frame to geodetic coordinates
fn ecef_to_lat_lon_alt(position: [f64; 3]) -> (f64, f64, f64) {
    const A: f64 = 6378.137; // WGS84 Earth semi-major axis (km)
    const F: f64 = 1.0 / 298.257223563; // Flattening
    const B: f64 = A * (1.0 - F); // Semi-minor axis (km)

    let (x, y, z) = (position[0], position[1], position[2]);

    // Calculate longitude
    let longitude = y.atan2(x).to_degrees();

    // Calculate latitude
    let e2 = 1.0 - (B * B) / (A * A); // Square of first eccentricity
    let p = (x.powi(2) + y.powi(2)).sqrt();
    let theta = (z * A) / (p * B);
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();
    let latitude = ((z + e2 * B * sin_theta.powi(3)) / (p - e2 * A * cos_theta.powi(3)))
        .atan()
        .to_degrees();

    // Calculate altitude
    let n = A / (1.0 - e2 * latitude.to_radians().sin().powi(2)).sqrt();
    let altitude = p / latitude.to_radians().cos() - n;

    (latitude, longitude, altitude)
}
