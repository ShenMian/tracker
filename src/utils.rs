use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use hifitime::Epoch;
use reverse_geocoder::ReverseGeocoder;
use serde::Deserialize;

use std::{
    f64::consts::{PI, TAU},
    sync::LazyLock,
};

use crate::object::Object;

/// A position in TEME frame.
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
    pub fn to_ecef(&self, gmst: f64) -> Ecef {
        teme_to_ecef(self, gmst)
    }
}

impl From<[f64; 3]> for Teme {
    fn from([x, y, z]: [f64; 3]) -> Self {
        Self::new(x, y, z)
    }
}

/// A position in ECEF frame.
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
#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct Lla {
    /// Latitude in degrees.
    pub lat: f64,
    /// Longitude in degrees.
    pub lon: f64,
    /// Altitude in km.
    pub alt: f64,
}

/// Reverse geocoder instance used to convert coordinates to location names.
static GEOCODER: LazyLock<ReverseGeocoder> = LazyLock::new(ReverseGeocoder::new);

impl Lla {
    pub fn new(lat: f64, lon: f64, alt: f64) -> Self {
        debug_assert!((-90.0..=90.0).contains(&lat));
        debug_assert!((-180.0..=180.0).contains(&lon));
        debug_assert!(alt >= 0.0);
        Lla { lat, lon, alt }
    }

    /// Converts the position to a ECEF position.
    pub fn to_ecef(&self) -> Ecef {
        lla_to_ecef(self)
    }

    /// Computes the azimuth and elevation from the observer's position to this
    /// point.
    pub fn az_el(&self, observer: &Lla) -> (f64, f64) {
        // Observer and target in ECEF
        let obs_ecef = observer.to_ecef();
        let tgt_ecef = self.to_ecef();

        // Vector from observer to target in ECEF
        let dx = tgt_ecef.x - obs_ecef.x;
        let dy = tgt_ecef.y - obs_ecef.y;
        let dz = tgt_ecef.z - obs_ecef.z;

        // Convert delta vector to local ENU coordinates at observer
        let lat0 = observer.lat.to_radians();
        let lon0 = observer.lon.to_radians();
        let sin_lat0 = lat0.sin();
        let cos_lat0 = lat0.cos();
        let sin_lon0 = lon0.sin();
        let cos_lon0 = lon0.cos();

        let east = -sin_lon0 * dx + cos_lon0 * dy;
        let north = -sin_lat0 * cos_lon0 * dx - sin_lat0 * sin_lon0 * dy + cos_lat0 * dz;
        let up = cos_lat0 * cos_lon0 * dx + cos_lat0 * sin_lon0 * dy + sin_lat0 * dz;

        // Azimuth: angle from north towards east, range [0, 360)
        let az_rad = east.atan2(north);
        let az_deg = az_rad.to_degrees().rem_euclid(360.0);

        // Elevation: angle between local horizontal plane and line-of-sight
        let horizontal_dist = (east.powi(2) + north.powi(2)).sqrt();
        let el_rad = up.atan2(horizontal_dist);
        let el_deg = el_rad.to_degrees();

        // If target is exactly at observer position
        if horizontal_dist == 0.0 && up == 0.0 {
            return (f64::NAN, f64::NAN);
        }

        (az_deg, el_deg)
    }

    /// Returns the city and country name.
    pub fn country_city(&self) -> (String, String) {
        let record = GEOCODER.search((self.lat, self.lon)).record;
        let city = &record.name;
        let country = match isocountry::CountryCode::for_alpha2(&record.cc) {
            Ok(code) => code.name(),
            Err(_) => "Unknown",
        };
        (country.to_owned(), city.to_owned())
    }
}

/// Converts a position vector from TEME frame to ECEF frame.
///
/// # Arguments
///
/// * `teme` - A position in the TEME frame (in km)
/// * `gmst_rad` - Greenwich Mean Sidereal Time in radians
///
/// # Returns
///
/// A position in the ECEF frame (same units as input).
fn teme_to_ecef(teme: &Teme, gmst_rad: f64) -> Ecef {
    let (sin_theta, cos_theta) = gmst_rad.sin_cos();
    let x = cos_theta * teme.x + sin_theta * teme.y;
    let y = -sin_theta * teme.x + cos_theta * teme.y;
    Ecef::new(x, y, teme.z)
}

mod wgs84 {
    pub const A: f64 = 6378.137; // Earth semi-major axis (km)
    pub const F: f64 = 1.0 / 298.257223563; // Flattening
    pub const B: f64 = A * (1.0 - F); // Semi-minor axis (km)
    pub const E2: f64 = 1.0 - (B * B) / (A * A); // Square of first eccentricity
}

/// Converts a ECEF position to geodetic position.
fn ecef_to_lla(ecef: &Ecef) -> Lla {
    use wgs84::*;

    // Calculate longitude
    let longitude = ecef.y.atan2(ecef.x);

    // Calculate latitude
    let p = (ecef.x.powi(2) + ecef.y.powi(2)).sqrt();
    let theta = (ecef.z * A).atan2(p * B);
    let (sin_theta, cos_theta) = theta.sin_cos();
    let latitude =
        ((ecef.z + E2 * B * sin_theta.powi(3)) / (p - E2 * A * cos_theta.powi(3))).atan();

    // Calculate altitude
    let n = A / (1.0 - E2 * latitude.sin().powi(2)).sqrt();
    let altitude = p / latitude.cos() - n;

    Lla::new(latitude.to_degrees(), longitude.to_degrees(), altitude)
}

/// Converts a geodetic position to ECEF position.
fn lla_to_ecef(lla: &Lla) -> Ecef {
    use wgs84::*;

    let lat = lla.lat.to_radians();
    let lon = lla.lon.to_radians();
    let alt = lla.alt;

    let sin_lat = lat.sin();
    let cos_lat = lat.cos();
    let cos_lon = lon.cos();
    let sin_lon = lon.sin();

    let n = A / (1.0 - E2 * sin_lat.powi(2)).sqrt();
    let x = (n + alt) * cos_lat * cos_lon;
    let y = (n + alt) * cos_lat * sin_lon;
    let z = (n * (1.0 - E2) + alt) * sin_lat;

    Ecef::new(x, y, z)
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
/// The GMST in radians, normalized to [0, 2π].
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
/// A tuple `(longitude, latitude)` in radians.
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
        let lat = (-(lon - sub_lon).cos() / decl.tan()).atan();
        // Skip if latitude is infinite (can happen at equinoxes when decl == 0)
        if lat.is_infinite() {
            continue;
        }
        points.push((lon.to_degrees(), lat.to_degrees()));
    }
    points
}

/// Calculates ground track points of the object.
pub fn calculate_ground_track(object: &Object, time: &DateTime<Utc>) -> Vec<(f64, f64)> {
    let mut points = Vec::with_capacity(object.orbital_period().num_minutes() as usize);
    for duration in (1..object.orbital_period().num_minutes()).map(Duration::minutes) {
        let state = object.predict(&(*time + duration)).unwrap();
        points.push((state.longitude(), state.latitude()));
    }
    points
}

/// Calculates the visibility circle for a point on the Earth's surface.
///
/// See <https://en.wikipedia.org/wiki/Great-circle_distance>
pub fn calculate_visibility_area(position: &Lla) -> Vec<(f64, f64)> {
    const AZIMUTH_STEP: usize = 10;

    let lat0_rad = position.lat.to_radians();
    let lon0_rad = position.lon.to_radians();
    let earth_radius = 6371.0088_f64; // mean Earth radius in km
    let cos_c = earth_radius / (earth_radius + position.alt.max(0.1));
    let central_angle_rad = cos_c.acos();
    let mut points = Vec::with_capacity(360 / AZIMUTH_STEP);
    for az in (-180..=180)
        .step_by(AZIMUTH_STEP)
        .map(|az| (az as f64).to_radians())
    {
        let lat_rad = (lat0_rad.sin() * central_angle_rad.cos()
            + lat0_rad.cos() * central_angle_rad.sin() * az.cos())
        .asin();
        let y = az.sin() * central_angle_rad.sin() * lat0_rad.cos();
        let x = central_angle_rad.cos() - lat0_rad.sin() * lat_rad.sin();
        let lon_rad = lon0_rad + y.atan2(x);
        let lat_deg = lat_rad.to_degrees();
        let lon_deg = wrap_longitude_deg(lon_rad.to_degrees());
        points.push((lon_deg, lat_deg));
    }
    points
}

/// Calculates sky track points for the object as seen from a ground station.
pub fn calculate_sky_track(
    object: &Object,
    ground_station: &Lla,
    time: &DateTime<Utc>,
) -> Vec<(f64, f64)> {
    const WINDOW_MINUTES: i64 = 30;
    const STEP_MIN: usize = 1;

    let mut points = Vec::with_capacity(2 * WINDOW_MINUTES as usize / STEP_MIN);
    for duration in (-WINDOW_MINUTES..=WINDOW_MINUTES)
        .step_by(STEP_MIN)
        .map(Duration::minutes)
    {
        let state = object.predict(&(*time + duration)).unwrap();
        let (az, el) = state.position.az_el(ground_station);
        if el < 0.0 {
            continue;
        }
        points.push(az_el_to_canvas(az, el));
    }
    points
}

/// Calculates satellite pass time segments within a given time window.
pub fn calculate_pass_times(
    object: &Object,
    observer: &Lla,
    start_time: &DateTime<Utc>,
    end_time: &DateTime<Utc>,
) -> Vec<(DateTime<Utc>, DateTime<Utc>)> {
    debug_assert!(start_time <= end_time);

    const TIME_STEP: Duration = Duration::minutes(1);

    let mut pass_segments = Vec::new();
    let mut current_pass_start: Option<DateTime<Utc>> = None;

    let mut time = *start_time;
    while time <= *end_time {
        let state = object.predict(&time).unwrap();
        let (_, el) = state.position.az_el(observer);
        let is_visible = el >= 0.0;

        match (current_pass_start, is_visible) {
            (None, true) => {
                // Start of a new pass
                current_pass_start = Some(time);
            }
            (Some(start), false) => {
                // End of current pass
                pass_segments.push((start, time - TIME_STEP));
                current_pass_start = None;
            }
            _ => {}
        }

        time += TIME_STEP;
    }

    if let Some(start) = current_pass_start {
        pass_segments.push((start, *end_time));
    }

    pass_segments
}

/// Converts azimuth and elevation to canvas coordinates.
///
/// Canvas is a unit circle using a Cartesian coordinate system.
pub fn az_el_to_canvas(az: f64, el: f64) -> (f64, f64) {
    let r = 1.0 - (el / 90.0);
    debug_assert!((0.0..=1.0).contains(&r));
    let (x, y) = polar_to_cartesian(r, (-az + 90.0).to_radians());
    (x, y)
}

/// Converts canvas coordinates to azimuth and elevation.
pub fn canvas_to_az_el(x: f64, y: f64) -> (f64, f64) {
    let (r, theta) = cartesian_to_polar(x, y);
    let el = (1.0 - r) * 90.0;
    let az = (90.0 - theta.to_degrees()).rem_euclid(360.0);
    (az, el)
}

/// Converts polar coordinates to Cartesian coordinates.
fn polar_to_cartesian(r: f64, theta: f64) -> (f64, f64) {
    (r * theta.cos(), r * theta.sin())
}

/// Converts Cartesian coordinates to polar coordinates.
fn cartesian_to_polar(x: f64, y: f64) -> (f64, f64) {
    let r = (x.powi(2) + y.powi(2)).sqrt();
    let theta = y.atan2(x);
    (r, theta)
}

/// Wraps a value to the range [-180, 180].
pub fn wrap_longitude_deg(lon: f64) -> f64 {
    (lon + 180.0).rem_euclid(360.0) - 180.0
}

/// Wraps a value to the range [-π, π].
pub fn wrap_longitude_rad(lon: f64) -> f64 {
    (lon + PI).rem_euclid(TAU) - PI
}
