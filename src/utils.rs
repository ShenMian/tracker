use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use hifitime::Epoch;

use std::f64::consts::{PI, TAU};

use crate::{coordinates::Lla, object::Object};

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
