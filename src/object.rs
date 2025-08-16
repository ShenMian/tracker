use chrono::{DateTime, Duration, Utc};

use crate::utils::*;

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
    pub fn predict(&self, time: &DateTime<Utc>) -> Result<State, sgp4::Error> {
        let minutes_since_epoch = (*time - self.epoch).as_seconds_f64() / 60.0;

        let prediction = self
            .constants
            .propagate(sgp4::MinutesSinceEpoch(minutes_since_epoch))?;

        let teme = Teme::from(prediction.position);

        let epoch = epoch_from_utc(time);
        let gmst = gmst_from_jd_tt(epoch.to_jde_tt_days());
        let cefe = teme.to_ecef(gmst);

        Ok(State {
            position: cefe.to_lla(),
            velocity: prediction.velocity,
        })
    }
}

#[derive(Clone, Debug)]
pub struct State {
    /// The position of the object in geodetic coordinates in km.
    pub position: Lla,
    /// The velocity of the object in km/s.
    pub velocity: [f64; 3],
}

impl State {
    pub fn latitude(&self) -> f64 {
        self.position.latitude
    }

    pub fn longitude(&self) -> f64 {
        self.position.longitude
    }

    pub fn altitude(&self) -> f64 {
        self.position.altitude
    }

    pub fn speed(&self) -> f64 {
        (self.velocity[0].powi(2) + self.velocity[1].powi(2) + self.velocity[2].powi(2)).sqrt()
    }
}
