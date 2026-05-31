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
        use super::wgs84::*;

        // Calculate longitude
        let longitude = self.y.atan2(self.x);

        // Calculate latitude
        let p = (self.x.powi(2) + self.y.powi(2)).sqrt();
        let theta = (self.z * A).atan2(p * B);
        let (sin_theta, cos_theta) = theta.sin_cos();
        let latitude =
            ((self.z + E2 * B * sin_theta.powi(3)) / (p - E2 * A * cos_theta.powi(3))).atan();

        // Calculate altitude
        let n = A / (1.0 - E2 * latitude.sin().powi(2)).sqrt();
        let altitude = p / latitude.cos() - n;

        Lla::new(latitude.to_degrees(), longitude.to_degrees(), altitude)
    }
}
