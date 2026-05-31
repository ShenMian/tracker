mod ecef;
mod lla;
mod teme;

pub use ecef::Ecef;
pub use lla::Lla;
pub use teme::Teme;

/// WGS84 geodetic reference frame constants.
mod wgs84 {
    /// Earth semi-major axis in kilometers.
    pub const A: f64 = 6378.137;
    /// Flattening factor.
    pub const F: f64 = 1.0 / 298.257223563;
    /// Earth semi-minor axis in kilometers.
    pub const B: f64 = A * (1.0 - F);
    /// First eccentricity squared.
    pub const E2: f64 = 1.0 - (B * B) / (A * A);
    /// Second eccentricity squared.
    pub const EP2: f64 = E2 / (1.0 - E2);
}
