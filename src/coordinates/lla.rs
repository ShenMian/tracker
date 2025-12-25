use std::sync::LazyLock;

use reverse_geocoder::ReverseGeocoder;
use serde::Deserialize;

use super::Ecef;

/// Reverse geocoder instance used to convert coordinates to location names.
static GEOCODER: LazyLock<ReverseGeocoder> = LazyLock::new(ReverseGeocoder::new);

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

mod wgs84 {
    pub const A: f64 = 6378.137; // Earth semi-major axis (km)
    pub const F: f64 = 1.0 / 298.257223563; // Flattening
    pub const B: f64 = A * (1.0 - F); // Semi-minor axis (km)
    pub const E2: f64 = 1.0 - (B * B) / (A * A); // Square of first eccentricity
}

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
