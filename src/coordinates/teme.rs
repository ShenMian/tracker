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
        let (sin_theta, cos_theta) = gmst.sin_cos();
        let x = cos_theta * self.x + sin_theta * self.y;
        let y = -sin_theta * self.x + cos_theta * self.y;
        Ecef::new(x, y, self.z)
    }
}

impl From<[f64; 3]> for Teme {
    fn from([x, y, z]: [f64; 3]) -> Self {
        Self::new(x, y, z)
    }
}
