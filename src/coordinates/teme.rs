use super::Ecef;

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
