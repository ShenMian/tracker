use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use hifitime::Epoch;

use crate::object::Object;

/// A position in True Equator Mean Equinox (TEME) frame.
#[derive(Clone, PartialEq, Debug)]
pub struct Teme {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Teme {
    /// Creates a new `Teme`.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Teme { x, y, z }
    }

    /// Converts the position to a ECEF position.
    ///
    /// # Arguments
    ///
    /// * `gmst` - Greenwich Mean Sidereal Time in radians
    ///
    /// # Returns
    ///
    /// A position in the ECEF frame (same units as input)
    pub fn to_ecef(&self, gmst: f64) -> Ecef {
        teme_to_ecef(self, gmst)
    }
}

impl From<[f64; 3]> for Teme {
    fn from([x, y, z]: [f64; 3]) -> Self {
        Self::new(x, y, z)
    }
}

/// A position in Earth-Centered Earth-Fixed (ECEF) frame.
#[derive(Clone, PartialEq, Debug)]
pub struct Ecef {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Ecef {
    /// Creates a new `Ecef`.
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Ecef { x, y, z }
    }

    /// Converts the position to a geodetic position.
    pub fn to_lla(&self) -> Lla {
        ecef_to_lla(self)
    }
}

/// A position in geodetic coordinates.
#[derive(Clone, PartialEq, Debug)]
pub struct Lla {
    /// Latitude in degrees.
    pub lat: f64,
    /// Longitude in degrees.
    pub lon: f64,
    /// Altitude in km.
    pub alt: f64,
}

impl Lla {
    pub fn new(lat: f64, lon: f64, alt: f64) -> Self {
        debug_assert!((-90.0..=90.0).contains(&lat));
        debug_assert!((-180.0..=180.0).contains(&lon));
        debug_assert!(alt >= 0.0);
        Lla { lat, lon, alt }
    }
}

/// Converts a position vector from True Equator Mean Equinox (TEME) frame to
/// Earth-Centered Earth-Fixed (ECEF) frame
///
/// # Arguments
///
/// * `position` - A position in the TEME frame (in km)
/// * `gmst` - Greenwich Mean Sidereal Time in radians
///
/// # Returns
///
/// A position in the ECEF frame (same units as input)
fn teme_to_ecef(teme: &Teme, gmst_rad: f64) -> Ecef {
    let (sin_theta, cos_theta) = gmst_rad.sin_cos();
    let x = cos_theta * teme.x + sin_theta * teme.y;
    let y = -sin_theta * teme.x + cos_theta * teme.y;
    Ecef::new(x, y, teme.z)
}

/// Converts a position vector from Earth-Centered Earth-Fixed (ECEF) frame to
/// geodetic coordinates (LLA)
///
/// # Arguments
///
/// * `position` - A position in the ECEF frame (in km)
///
/// # Returns
///
/// A position in geodetic coordinates
fn ecef_to_lla(ecef: &Ecef) -> Lla {
    // Constants for WGS84 ellipsoid
    const A: f64 = 6378.137; // Earth semi-major axis (km)
    const F: f64 = 1.0 / 298.257223563; // Flattening
    const B: f64 = A * (1.0 - F); // Semi-minor axis (km)
    const E2: f64 = 1.0 - (B * B) / (A * A); // Square of first eccentricity

    // Calculate longitude
    let longitude = ecef.y.atan2(ecef.x);

    // Calculate latitude
    let p = (ecef.x.powi(2) + ecef.y.powi(2)).sqrt();
    let theta = (ecef.z * A) / (p * B);
    let (sin_theta, cos_theta) = theta.sin_cos();
    let latitude =
        ((ecef.z + E2 * B * sin_theta.powi(3)) / (p - E2 * A * cos_theta.powi(3))).atan();

    // Calculate altitude
    let n = A / (1.0 - E2 * latitude.sin().powi(2)).sqrt();
    let altitude = p / latitude.cos() - n;

    Lla::new(latitude.to_degrees(), longitude.to_degrees(), altitude)
}

/// Returns the Epoch for the given UTC timestamp.
pub fn epoch_from_utc(time: &DateTime<Utc>) -> Epoch {
    Epoch::from_gregorian_utc(
        time.year(),
        time.month() as u8,
        time.day() as u8,
        time.hour() as u8,
        time.minute() as u8,
        time.second() as u8,
        time.nanosecond(),
    )
}

/// Calculates the Greenwich Mean Sidereal Time (GMST) in radians.
///
/// # Arguments
///
/// * `jd` - The Julian days in TT time scale
///
/// # Returns
///
/// The GMST in radians, normalized to [0, 2π]
pub fn gmst_from_jd_tt(jd: f64) -> f64 {
    const J2000_EPOCH: f64 = 2451545.0; // Julian Date for J2000.0 epoch
    const JULIAN_CENTURY: f64 = 36525.0; // Days in a Julian century

    // GMST formula coefficients (in degrees)
    const GMST_MEAN: f64 = 280.46061837;
    const GMST_ADVANCE: f64 = 360.98564736629;
    const T2_COEFF: f64 = 0.000387933;
    const T3_COEFF: f64 = -1.0 / 38710000.0;

    // Calculate time in Julian centuries since J2000.0
    let t = (jd - J2000_EPOCH) / JULIAN_CENTURY;

    // Calculate GMST in degrees
    let gmst =
        GMST_MEAN + GMST_ADVANCE * (jd - J2000_EPOCH) + T2_COEFF * t.powi(2) + T3_COEFF * t.powi(3);

    // Convert to radians and normalize to [0, 2π]
    gmst.rem_euclid(360.0).to_radians()
}

/// Calculates the subsolar point at a given UTC timestamp.
///
/// # Arguments
///
/// * `time` - The UTC timestamp for which to compute the subsolar point.
///
/// # Returns
///
/// A tuple `(longitude, latitude)` in radians, where:
/// - `longitude`: Subsolar longitude in the range [-π, π) radians.
/// - `latitude`: Subsolar latitude in radians.
pub fn subsolar_point(time: &DateTime<Utc>) -> (f64, f64) {
    let epoch = epoch_from_utc(time);
    let jd = epoch.to_jde_tt_days();

    let n = jd - 2451545.0;
    let mean_long = (280.46 + 0.9856474 * n).rem_euclid(360.0).to_radians();
    let mean_anom = (357.528 + 0.9856003 * n).to_radians();
    let eclip_long = mean_long
        + 1.915_f64.to_radians() * mean_anom.sin()
        + 0.02_f64.to_radians() * (2.0 * mean_anom).sin();
    let obliq = 23.439_f64.to_radians();
    let decl = (obliq.sin() * eclip_long.sin()).asin();
    let gmst = gmst_from_jd_tt(jd);
    let lon = wrap_longitude_rad(mean_long - gmst);
    (lon, decl)
}

/// Calculates a set of points representing the day-night terminator.
///
/// # Arguments
///
/// * `time` - The UTC timestamp for which to compute the terminator.
///
/// # Returns
///
/// A vector of `(longitude, latitude)` pairs in degrees, representing the
/// terminator line.
pub fn calculate_terminator(time: &DateTime<Utc>) -> Vec<(f64, f64)> {
    const LON_STEP: usize = 5;

    let (sub_lon, decl) = subsolar_point(time);
    let mut points = Vec::with_capacity(361 / LON_STEP);
    for lon in (-180..=180)
        .step_by(LON_STEP)
        .map(|lon| (lon as f64).to_radians())
    {
        // lat = atan(-cos(lon - sub_lon) / tan(decl))
        let lat = (-(lon - sub_lon).cos() / decl.tan()).atan();
        // Skip if latitude is infinite (can happen at equinoxes when decl == 0)
        if lat.is_infinite() {
            continue;
        }
        points.push((lon.to_degrees(), lat.to_degrees()));
    }
    points
}

/// Calculates a set of points representing the trajectory of the object.
pub fn calculate_trajectory(object: &Object, time: &DateTime<Utc>) -> Vec<(f64, f64)> {
    // Calculate future positions along the trajectory
    let mut points = Vec::new();
    for minutes in 1..object.orbital_period().num_minutes() {
        let state = object
            .predict(&(*time + Duration::minutes(minutes)))
            .unwrap();
        points.push((state.longitude(), state.latitude()));
    }
    points
}

/// Calculates the visibility circle for a point on the Earth's surface.
///
/// See <https://en.wikipedia.org/wiki/Great-circle_distance>
pub fn calculate_visibility_area(position: &Lla, num_points: usize) -> Vec<(f64, f64)> {
    const AZIMUTH_STEP: usize = 10;

    let lat0_rad = position.lat.to_radians();
    let lon0_rad = position.lon.to_radians();
    let earth_radius = 6371.0088_f64; // mean Earth radius in km
    let cos_c = earth_radius / (earth_radius + position.alt.max(0.1));
    let central_angle_rad = cos_c.acos();
    let mut points = Vec::with_capacity(num_points + 1);
    for azimuth in (-180..=180)
        .step_by(AZIMUTH_STEP)
        .map(|azimuth| (azimuth as f64).to_radians())
    {
        let lat_rad = (lat0_rad.sin() * central_angle_rad.cos()
            + lat0_rad.cos() * central_angle_rad.sin() * azimuth.cos())
        .asin();
        let y = azimuth.sin() * central_angle_rad.sin() * lat0_rad.cos();
        let x = central_angle_rad.cos() - lat0_rad.sin() * lat_rad.sin();
        let lon_rad = lon0_rad + y.atan2(x);
        let lat_deg = lat_rad.to_degrees();
        let lon_deg = wrap_longitude_deg(lon_rad.to_degrees());
        points.push((lon_deg, lat_deg));
    }
    points
}

/// Wraps a longitude value in degrees to the range [-180, 180].
pub fn wrap_longitude_deg(lon: f64) -> f64 {
    (lon + 180.0).rem_euclid(360.0) - 180.0
}

/// Wraps a longitude value in radians to the range [-π, π].
pub fn wrap_longitude_rad(lon: f64) -> f64 {
    (lon + std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI
}
