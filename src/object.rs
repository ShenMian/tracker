use chrono::{DateTime, Duration, Utc};
use nalgebra::{Point3, Vector3};

use crate::utils::teme_to_lla;

const SECONDS_PER_DAY: f64 = 24.0 * 60.0 * 60.0;

/// A satellite object with orbital elements.
#[derive(Clone, Debug)]
pub struct Object {
    epoch: DateTime<Utc>,
    orbital_period: Duration,
    elements: sgp4::Elements,
    constants: sgp4::Constants,
}

impl Object {
    /// Creates a new `Object` from SGP4 elements.
    pub fn from_elements(elements: sgp4::Elements) -> Self {
        let orbital_period = Duration::seconds((SECONDS_PER_DAY / elements.mean_motion) as i64);

        Self {
            epoch: DateTime::from_naive_utc_and_offset(elements.datetime, Utc),
            orbital_period,
            constants: sgp4::Constants::from_elements(&elements).unwrap(),
            elements,
        }
    }

    /// Returns the name of the object.
    pub fn name(&self) -> Option<&str> {
        self.elements.object_name.as_deref()
    }

    /// Returns the UTC timestamp of the elements.
    pub fn epoch(&self) -> DateTime<Utc> {
        self.epoch
    }

    /// Returns the orbital period of the object.
    pub fn orbital_period(&self) -> &Duration {
        &self.orbital_period
    }

    /// Returns the SGP4 elements of the object.
    pub fn elements(&self) -> &sgp4::Elements {
        &self.elements
    }

    /// Predicts the state of the object at the given time.
    pub fn predict(&self, time: DateTime<Utc>) -> Result<State, sgp4::Error> {
        let minutes_since_epoch = (time - self.epoch).as_seconds_f64() / 60.0;

        let prediction = self
            .constants
            .propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch))?;

        let [lat, lon, alt] = teme_to_lla(Point3::from(prediction.position), time);

        Ok(State {
            position: Point3::new(lon, lat, alt),
            velocity: Vector3::from(prediction.velocity),
        })
    }
}

#[derive(Clone, Debug)]
pub struct State {
    /// The position of the object in geodetic coordinates (longitude, latitude,
    /// altitude) in km.
    pub position: Point3<f64>,
    /// The velocity of the object in km/s.
    pub velocity: Vector3<f64>,
}

impl State {
    pub fn latitude(&self) -> f64 {
        self.position.y
    }

    pub fn longitude(&self) -> f64 {
        self.position.x
    }

    pub fn altitude(&self) -> f64 {
        self.position.z
    }

    pub fn speed(&self) -> f64 {
        (self.velocity.x.powi(2) + self.velocity.y.powi(2) + self.velocity.z.powi(2)).sqrt()
    }
}
