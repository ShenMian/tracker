use chrono::{DateTime, Datelike, Duration, Timelike, Utc};

const SECONDS_PER_DAY: f64 = 24.0 * 60.0 * 60.0;

/// A satellite object with orbital elements.
#[derive(Clone, Debug)]
pub struct Object {
    epoch: DateTime<Utc>,
    orbital_period: Duration,
    elements: sgp4::Elements,
    constants: sgp4::Constants,
}

impl Object {
    /// Creates a new `Object` from SGP4 elements.
    pub fn from_elements(elements: sgp4::Elements) -> Self {
        let orbital_period = Duration::seconds((SECONDS_PER_DAY / elements.mean_motion) as i64);

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
    pub fn orbital_period(&self) -> &Duration {
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

        let [lat, lon, alt] = teme_to_lla(prediction.position, time);

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

/// Converts a position vector from TEME frame to LLA.
fn teme_to_lla(teme: [f64; 3], time: DateTime<Utc>) -> [f64; 3] {
    let jd_utc = julian_days_from_utc(time);
    let jd_tt = tai_to_tt(utc_to_tai(jd_utc));
    let gmst_rad = gmst_rad_from_julian_days_tt(jd_tt);
    ecef_to_lla(teme_to_ecef(teme, gmst_rad))
}

/// Returns the Julian days in UTC for the given UTC datetime.
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

/// Converts UTC time scale (in days) to TAI time scale (in days).
fn utc_to_tai(utc: f64) -> f64 {
    const TAI_MINUS_UTC: f64 = 37.0; // TAI-UTC in seconds (2025, https://datacenter.iers.org/data/html/bulletinc-069.html)
    utc + TAI_MINUS_UTC / SECONDS_PER_DAY
}

/// Converts TAI time scale (in days) to TT time scale (in days).
fn tai_to_tt(tai: f64) -> f64 {
    const TT_MINUS_TAI: f64 = 32.184; // TT-TAI in seconds
    tai + TT_MINUS_TAI / SECONDS_PER_DAY
}

/// Calculates the Greenwich Mean Sidereal Time (GMST) in radians.
///
/// # Arguments
/// * `julian_days` - The Julian days in TT time scale
///
/// # Returns
///
/// The GMST in radians, normalized to [0, 2π]
fn gmst_rad_from_julian_days_tt(julian_days: f64) -> f64 {
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
    gmst.rem_euclid(360.0).to_radians()
}

/// Converts a position vector from True Equator Mean Equinox (TEME) frame to Earth-Centered Earth-Fixed (ECEF) frame
///
/// # Arguments
/// * `position` - A 3D position vector [x, y, z] in the TEME frame (in km)
/// * `gmst` - Greenwich Mean Sidereal Time in radians
///
/// # Returns
/// A 3D position vector [x, y, z] in the ECEF frame (same units as input)
fn teme_to_ecef(teme: [f64; 3], gmst_rad: f64) -> [f64; 3] {
    let [teme_x, teme_y, teme_z] = teme;
    let (sin_theta, cos_theta) = gmst_rad.sin_cos();
    let x = cos_theta * teme_x + sin_theta * teme_y;
    let y = -sin_theta * teme_x + cos_theta * teme_y;
    [x, y, teme_z]
}

/// Converts a position vector from Earth-Centered Earth-Fixed (ECEF) frame to geodetic coordinates (LLA)
///
/// # Arguments
/// * `position` - A 3D position vector [x, y, z] in the ECEF frame (in km)
///
/// # Returns
/// * A array [latitude, longitude, altitude] where:
///   - latitude: Geodetic latitude in degrees (-90° to +90°)
///   - longitude: Geodetic longitude in degrees (-180° to +180°)
///   - altitude: Height above WGS84 ellipsoid in km
fn ecef_to_lla(position: [f64; 3]) -> [f64; 3] {
    const A: f64 = 6378.137; // WGS84 Earth semi-major axis (km)
    const F: f64 = 1.0 / 298.257223563; // Flattening
    const B: f64 = A * (1.0 - F); // Semi-minor axis (km)
    const E2: f64 = 1.0 - (B * B) / (A * A); // Square of first eccentricity

    let [x, y, z] = position;

    // Calculate longitude
    let longitude = y.atan2(x).to_degrees();

    // Calculate latitude
    let p = (x.powi(2) + y.powi(2)).sqrt();
    let theta = (z * A) / (p * B);
    let (sin_theta, cos_theta) = theta.sin_cos();
    let latitude = ((z + E2 * B * sin_theta.powi(3)) / (p - E2 * A * cos_theta.powi(3)))
        .atan()
        .to_degrees();

    // Calculate altitude
    let n = A / (1.0 - E2 * latitude.to_radians().sin().powi(2)).sqrt();
    let altitude = p / latitude.to_radians().cos() - n;

    [latitude, longitude, altitude]
}
