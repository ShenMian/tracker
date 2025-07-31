use chrono::{DateTime, Datelike, Duration, Timelike, Utc};
use hifitime::Epoch;
use nalgebra::Point3;

use crate::object::Object;

/// Converts a position from TEME frame to LLA.
///
/// # Returns
///
/// An array `[latitude, longitude, altitude]` where:
///   - latitude: Geodetic latitude in degrees (-90° to +90°)
///   - longitude: Geodetic longitude in degrees (-180° to +180°)
///   - altitude: Height above WGS84 ellipsoid in km
pub fn teme_to_lla(teme: Point3<f64>, time: DateTime<Utc>) -> [f64; 3] {
    let epoch = epoch_from_utc(time);
    let gmst = gmst_from_jde_tt(epoch.to_jde_tt_days());
    ecef_to_lla(teme_to_ecef(teme, gmst))
}

/// Calculates the Greenwich Mean Sidereal Time (GMST) in radians.
///
/// # Arguments
///
/// * `jde` - The Julian days in TT time scale
///
/// # Returns
///
/// The GMST in radians, normalized to [0, 2π]
fn gmst_from_jde_tt(jde: f64) -> f64 {
    // Constants
    const J2000_EPOCH: f64 = 2451545.0; // Julian Date for J2000.0 epoch
    const JULIAN_CENTURY: f64 = 36525.0; // Days in a Julian century

    // GMST formula coefficients (in degrees)
    const GMST_MEAN: f64 = 280.46061837;
    const GMST_ADVANCE: f64 = 360.98564736629;
    const T2_COEFF: f64 = 0.000387933;
    const T3_COEFF: f64 = -1.0 / 38710000.0;

    // Calculate time in Julian centuries since J2000.0
    let t = (jde - J2000_EPOCH) / JULIAN_CENTURY;

    // Calculate GMST in degrees
    let gmst = GMST_MEAN
        + GMST_ADVANCE * (jde - J2000_EPOCH)
        + T2_COEFF * t.powi(2)
        + T3_COEFF * t.powi(3);

    // Convert to radians and normalize to [0, 2π]
    gmst.rem_euclid(360.0).to_radians()
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
fn teme_to_ecef(teme: Point3<f64>, gmst_rad: f64) -> Point3<f64> {
    let (sin_theta, cos_theta) = gmst_rad.sin_cos();
    let x = cos_theta * teme.x + sin_theta * teme.y;
    let y = -sin_theta * teme.x + cos_theta * teme.y;
    Point3::new(x, y, teme.z)
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
/// An array `[latitude, longitude, altitude]` where:
///   - latitude: Geodetic latitude in degrees (-90° to +90°)
///   - longitude: Geodetic longitude in degrees (-180° to +180°)
///   - altitude: Height above WGS84 ellipsoid in km
fn ecef_to_lla(ecef: Point3<f64>) -> [f64; 3] {
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

    [latitude.to_degrees(), longitude.to_degrees(), altitude]
}

/// Returns the Epoch for the given UTC timestamp.
fn epoch_from_utc(time: DateTime<Utc>) -> Epoch {
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
pub fn subsolar_point(time: DateTime<Utc>) -> (f64, f64) {
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
    let gmst = gmst_from_jde_tt(jd);
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
pub fn calculate_terminator(time: DateTime<Utc>) -> Vec<(f64, f64)> {
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
pub fn calculate_trajectory(object: &Object, time: DateTime<Utc>) -> Vec<(f64, f64)> {
    // Calculate future positions along the trajectory
    let mut points = Vec::new();
    for minutes in 1..object.orbital_period().num_minutes() {
        let time = time + Duration::minutes(minutes);
        let object_state = object.predict(time).unwrap();
        points.push((object_state.position.x, object_state.position.y));
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
