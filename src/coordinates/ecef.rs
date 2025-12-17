use super::Lla;

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
