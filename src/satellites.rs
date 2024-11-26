use std::{fmt, fs, path::PathBuf, time::Duration};

use ureq::serde_json;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Satellites {
    // Navigation satellites
    Beidou,
    Galileo,
    Glonass,
    Gps,

    // Space stations
    Css,
    Iss,
}

impl Satellites {
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
    fn id(&self) -> Option<&str> {
        match self {
            Self::Iss => Some("1998-067A"),
            Self::Css => Some("2021-035A"),
            _ => None,
        }
    }

    /// Returns CelesTrak group name
    fn group(&self) -> Option<&str> {
        match self {
            Self::Glonass => Some("glo-ops"),
            Self::Gps => Some("gps-ops"),
            Self::Beidou => Some("beidou"),
            Self::Galileo => Some("galileo"),
            _ => None,
        }
    }

    fn fetch_elements(&self) -> Vec<sgp4::Elements> {
        let request = ureq::get("https://celestrak.org/NORAD/elements/gp.php");
        if let Some(id) = self.id() {
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

impl fmt::Display for Satellites {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
