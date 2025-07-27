use std::time::Duration;

use strum::{Display, EnumIter};
use tokio::fs;

/// The `SatelliteGroup` type.
///
/// Type [`SatelliteGroup`] represents a group of satellites.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Display, EnumIter)]
pub enum SatelliteGroup {
    // Space stations
    #[strum(to_string = "CSS")]
    Css,
    #[strum(to_string = "ISS")]
    Iss,

    // Weather satellites
    Weather,
    #[strum(to_string = "NOAA")]
    Noaa,
    #[strum(to_string = "GOES")]
    Goes,

    // Earth resources satellites
    #[strum(to_string = "Earth resources")]
    EarthResources,
    #[strum(to_string = "Search & rescue")]
    SearchRescue,
    #[strum(to_string = "Disaster monitoring")]
    DisasterMonitoring,

    // Navigation satellites
    #[strum(to_string = "GPS Operational")]
    Gps,
    #[strum(to_string = "GLONASS Operational")]
    Glonass,
    Galileo,
    Beidou,

    // Scientific satellites
    #[strum(to_string = "Space & Earth Science")]
    SpaceEarthScience,
    Geodetic,
    Engineering,
    Education,

    // Miscellaneous satellites
    #[strum(to_string = "DFH-1")]
    Dfh1,
    Military,
    #[strum(to_string = "Radar calibration")]
    RadarCalibration,
    CubeSats,
}

impl SatelliteGroup {
    /// Returns SGP4 elements.
    ///
    /// If cache is expired, fetches elements from <https://celestrak.org>.
    /// Otherwise, reads elements from cache.
    pub async fn get_elements(&self) -> Option<Vec<sgp4::Elements>> {
        let cache_path =
            std::env::temp_dir().join(format!("tracker/{}.json", self.to_string().to_lowercase()));
        fs::create_dir_all(cache_path.parent().unwrap())
            .await
            .unwrap();

        // Fetch elements if cache doesn't exist
        if !std::fs::exists(&cache_path).unwrap() {
            if let Some(elements) = self.fetch_elements().await {
                fs::write(&cache_path, serde_json::to_string(&elements).unwrap())
                    .await
                    .unwrap();
            } else {
                return None;
            }
        }

        let age = fs::metadata(&cache_path)
            .await
            .unwrap()
            .modified()
            .unwrap()
            .elapsed()
            .unwrap();
        let is_cache_expired = age > Duration::from_secs(2 * 60 * 60);

        // Fetch elements if cache is expired
        if is_cache_expired {
            if let Some(elements) = self.fetch_elements().await {
                fs::write(&cache_path, serde_json::to_string(&elements).unwrap())
                    .await
                    .unwrap();
            }
        }

        let json = fs::read_to_string(&cache_path).await.unwrap();
        serde_json::from_str(&json).unwrap()
    }

    /// Fetches SGP4 elements from <https://celestrak.org>.
    async fn fetch_elements(&self) -> Option<Vec<sgp4::Elements>> {
        const URL: &str = "https://celestrak.com/NORAD/elements/gp.php";

        let mut request = reqwest::Client::new().get(URL).query(&[("FORMAT", "json")]);
        request = match (self.cospar_id(), self.group_name()) {
            (Some(id), None) => request.query(&[("INTDES", id)]),
            (None, Some(group)) => request.query(&[("GROUP", group)]),
            _ => unreachable!(),
        };

        let response = request.send().await.ok()?;
        Some(
            response
                .json()
                .await
                .expect("failed to parse JSON from celestrak.org"),
        )
    }

    /// Returns the international designator.
    fn cospar_id(&self) -> Option<&str> {
        match self {
            Self::Iss => Some("1998-067A"),
            Self::Css => Some("2021-035A"),
            Self::Dfh1 => Some("1970-034A"),
            _ => None,
        }
    }

    /// Returns CelesTrak group name.
    fn group_name(&self) -> Option<&str> {
        match self {
            Self::Weather => Some("weather"),
            Self::Noaa => Some("noaa"),
            Self::Goes => Some("goes"),
            Self::EarthResources => Some("resource"),
            Self::SearchRescue => Some("sarsat"),
            Self::DisasterMonitoring => Some("dmc"),
            Self::Gps => Some("gps-ops"),
            Self::Glonass => Some("glo-ops"),
            Self::Galileo => Some("galileo"),
            Self::Beidou => Some("beidou"),
            Self::SpaceEarthScience => Some("science"),
            Self::Geodetic => Some("geodetic"),
            Self::Engineering => Some("engineering"),
            Self::Education => Some("education"),
            Self::Military => Some("military"),
            Self::RadarCalibration => Some("radar"),
            Self::CubeSats => Some("cubesat"),
            _ => None,
        }
    }
}
