use std::{fmt, fs, path::PathBuf, time::Duration};

use ureq::serde_json;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Satellite {
    // Space stations
    Css,
    Iss,

    // Weather & earth resources satellites
    Weather,
    NOAA,
    GOES,
    EarthResources,
    SearchRescue,
    DisasterMonitoring,

    // Navigation satellites
    Gps,
    Glonass,
    Galileo,
    Beidou,

    // Scientific satellites
    SpaceEarthScience,
    Geodetic,
    Engineering,
    Education,

    // Miscellaneous satellites
    Dfh1,
    Military,
    RadarCalibration,
    CubeSats,
}

impl Satellite {
    pub fn get_elements(&self) -> Vec<sgp4::Elements> {
        let cache_path = PathBuf::from(format!("cache/{}.json", self.to_string().to_lowercase()));
        fs::create_dir_all(cache_path.parent().unwrap()).unwrap();

        let should_update = match fs::metadata(&cache_path) {
            Ok(metadata) => {
                metadata.modified().unwrap().elapsed().unwrap() > Duration::from_secs(2 * 60 * 60)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            _ => panic!(),
        };

        if should_update {
            let elements = self.fetch_elements();
            fs::write(&cache_path, serde_json::to_string(&elements).unwrap()).unwrap();
            elements
        } else {
            let json = fs::read_to_string(&cache_path).unwrap();
            serde_json::from_str(&json).unwrap()
        }
    }

    /// Returns the international designator
    fn cospar_id(&self) -> Option<&str> {
        match self {
            Self::Iss => Some("1998-067A"),
            Self::Css => Some("2021-035A"),
            Self::Dfh1 => Some("1970-034A"),
            _ => None,
        }
    }

    /// Returns CelesTrak group name
    fn group(&self) -> Option<&str> {
        match self {
            Self::Weather => Some("weather"),
            Self::NOAA => Some("noaa"),
            Self::GOES => Some("goes"),
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

    fn fetch_elements(&self) -> Vec<sgp4::Elements> {
        let request = ureq::get("https://celestrak.org/NORAD/elements/gp.php");
        if let Some(id) = self.cospar_id() {
            let response = request
                .query("INTDES", id)
                .query("FORMAT", "json")
                .call()
                .unwrap();
            return response.into_json().unwrap();
        }
        if let Some(group) = self.group() {
            let response = request
                .query("GROUP", group)
                .query("FORMAT", "json")
                .call()
                .unwrap();
            return response.into_json().unwrap();
        }
        unreachable!();
    }
}

impl fmt::Display for Satellite {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Gps => write!(f, "GPS Operational"),
            Self::Glonass => write!(f, "GLONASS Operational"),
            Self::Css => write!(f, "CSS"),
            Self::Iss => write!(f, "ISS"),
            Self::Dfh1 => write!(f, "DFH-1"),
            Self::EarthResources => write!(f, "Earth resources"),
            Self::SearchRescue => write!(f, "Search & rescue"),
            Self::DisasterMonitoring => write!(f, "Disaster monitoring"),
            Self::SpaceEarthScience => write!(f, "Space & Earth Science"),
            Self::RadarCalibration => write!(f, "Radar calibration"),
            _ => write!(f, "{:?}", self),
        }
    }
}
