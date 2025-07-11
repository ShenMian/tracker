use std::f64::consts::PI;

use chrono::{DateTime, Datelike, Timelike, Utc};

/// A satellite object with orbital elements.
#[derive(Clone, Debug)]
pub struct Object {
    epoch: DateTime<Utc>,
    orbital_period: chrono::Duration,
    elements: sgp4::Elements,
    constants: sgp4::Constants,
}

impl Object {
    /// Creates a new `Object` from SGP4 elements.
    pub fn from_elements(elements: sgp4::Elements) -> Self {
        const SECONDS_PER_DAY: f64 = 24.0 * 60.0 * 60.0;
        let orbital_period =
            chrono::Duration::seconds((SECONDS_PER_DAY / elements.mean_motion) as i64);

        Self {
            epoch: DateTime::from_naive_utc_and_offset(elements.datetime, Utc),
            orbital_period,
            constants: sgp4::Constants::from_elements(&elements).unwrap(),
            elements,
        }
    }

    /// Returns the name of the object.
    pub fn name(&self) -> Option<&str> {
        self.elements.object_name.as_deref()
    }

    /// Returns the UTC timestamp of the elements.
    pub fn epoch(&self) -> DateTime<Utc> {
        self.epoch
    }

    /// Returns the orbital period of the object.
    pub fn orbital_period(&self) -> &chrono::Duration {
        &self.orbital_period
    }

    /// Returns the SGP4 elements of the object.
    pub fn elements(&self) -> &sgp4::Elements {
        &self.elements
    }

    /// Predicts the state of the object at the given time.
    pub fn predict(&self, time: DateTime<Utc>) -> Result<State, sgp4::Error> {
        let minutes_since_epoch = (time - self.epoch).num_seconds() as f64 / 60.0;

        let prediction = self
            .constants
            .propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch))?;

        let gmst = gmst_from_julian_days(julian_days_from_utc(time));
        let [lat, lon, alt] = ecef_to_lat_lon_alt(teme_to_ecef(prediction.position, gmst));

        debug_assert!((-90.0..=90.0).contains(&lat), "latitude out of range");
        debug_assert!((-180.0..=180.0).contains(&lon), "longitude out of range");

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

/// Returns the Julian days for the given UTC datetime.
fn julian_days_from_utc(datetime: DateTime<Utc>) -> f64 {
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
    (365.25 * (y as f64 + 4716.0)).floor()
        + (30.6001 * (m as f64 + 1.0)).floor()
        + day as f64
        + hour / 24.0
        - 1524.5
        + b
}

/// Calculates the Greenwich Mean Sidereal Time (GMST) in radians.
///
/// # Arguments
/// * `julian_days` - The Julian days for which to calculate GMST
///
/// # Returns
///
/// The GMST in radians, normalized to [0, 2π]
fn gmst_from_julian_days(julian_days: f64) -> f64 {
    // Constants
    const J2000_EPOCH: f64 = 2451545.0; // Julian Date for J2000.0 epoch
    const JULIAN_CENTURY: f64 = 36525.0; // Days in a Julian century

    // GMST formula coefficients (in degrees)
    const GMST_MEAN: f64 = 280.46061837;
    const GMST_ADVANCE: f64 = 360.98564736629;
    const T2_COEFF: f64 = 0.000387933;
    const T3_COEFF: f64 = -1.0 / 38710000.0;

    // Calculate time in Julian centuries since J2000.0
    let t = (julian_days - J2000_EPOCH) / JULIAN_CENTURY;

    // Calculate GMST in degrees
    let gmst = GMST_MEAN
        + GMST_ADVANCE * (julian_days - J2000_EPOCH)
        + T2_COEFF * t.powi(2)
        + T3_COEFF * t.powi(3);

    // Convert to radians and normalize to [0, 2π]
    (gmst % 360.0).to_radians().rem_euclid(2.0 * PI)
}

/// Converts a position vector from True Equator Mean Equinox (TEME) frame to Earth-Centered Earth-Fixed (ECEF) frame
///
/// # Arguments
/// * `position` - A 3D position vector [x, y, z] in the TEME frame (typically in kilometers)
/// * `gmst` - Greenwich Mean Sidereal Time in radians
///
/// # Returns
/// A 3D position vector [x, y, z] in the ECEF frame (same units as input)
fn teme_to_ecef(position: [f64; 3], gmst: f64) -> [f64; 3] {
    let [x, y, z] = position;
    let cos_gmst = gmst.cos();
    let sin_gmst = gmst.sin();

    let x_ecef = cos_gmst * x + sin_gmst * y;
    let y_ecef = -sin_gmst * x + cos_gmst * y;
    [x_ecef, y_ecef, z]
}

/// Converts a position vector from Earth-Centered Earth-Fixed (ECEF) frame to geodetic coordinates (latitude, longitude, altitude)
///
/// # Arguments
/// * `position` - A 3D position vector [x, y, z] in the ECEF frame (kilometers)
///
/// # Returns
/// * A tuple (latitude, longitude, altitude) where:
///   - latitude: Geodetic latitude in degrees (-90° to +90°)
///   - longitude: Geodetic longitude in degrees (-180° to +180°)
///   - altitude: Height above WGS84 ellipsoid in kilometers
fn ecef_to_lat_lon_alt(position: [f64; 3]) -> [f64; 3] {
    const A: f64 = 6378.137; // WGS84 Earth semi-major axis (km)
    const F: f64 = 1.0 / 298.257223563; // Flattening
    const B: f64 = A * (1.0 - F); // Semi-minor axis (km)

    let [x, y, z] = position;

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

    [latitude, longitude, altitude]
}
