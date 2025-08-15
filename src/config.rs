use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub world_map: WorldMapConfig,
    pub satellite_groups: SatelliteGroupsConfig,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct WorldMapConfig {
    pub follow_selected_object: bool,
    pub show_terminator: bool,
    pub show_cursor_position: bool,

    pub lon_delta_deg: f64,
    pub time_delta_min: i64,

    pub map_color: String,
    pub trajectory_color: String,
    pub terminator_color: String,
}

impl Default for WorldMapConfig {
    fn default() -> Self {
        Self {
            follow_selected_object: true,
            show_terminator: true,
            show_cursor_position: false,
            lon_delta_deg: 10.0,
            time_delta_min: 1,
            map_color: "gray".into(),
            trajectory_color: "light_blue".into(),
            terminator_color: "dark_gray".into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct SatelliteGroupsConfig {
    pub cache_lifetime_min: u64,
    pub groups: Vec<GroupConfig>,
}

#[derive(Deserialize)]
pub struct GroupConfig {
    pub label: String,
    pub id: Option<String>,
    pub group: Option<String>,
}

impl GroupConfig {
    fn with_id(label: String, cospar_id: String) -> Self {
        Self {
            label,
            id: Some(cospar_id),
            group: None,
        }
    }

    fn with_group(label: String, group_name: String) -> Self {
        Self {
            label,
            id: None,
            group: Some(group_name),
        }
    }
}

impl Default for SatelliteGroupsConfig {
    fn default() -> Self {
        Self {
            cache_lifetime_min: 2 * 60,
            groups: vec![
                GroupConfig::with_id("ISS".into(), "1998-067A".into()),
                GroupConfig::with_id("CSS".into(), "2021-035A".into()),
                GroupConfig::with_group("Weather".into(), "weather".into()),
                GroupConfig::with_group("NOAA".into(), "noaa".into()),
                GroupConfig::with_group("GOES".into(), "goes".into()),
                GroupConfig::with_group("Earth resources".into(), "resource".into()),
                GroupConfig::with_group("Search & rescue".into(), "sarsat".into()),
                GroupConfig::with_group("Disaster monitoring".into(), "dmc".into()),
                GroupConfig::with_group("GPS".into(), "gps-ops".into()),
                GroupConfig::with_group("GLONASS".into(), "glo-ops".into()),
                GroupConfig::with_group("Galileo".into(), "galileo".into()),
                GroupConfig::with_group("Beidou".into(), "beidou".into()),
                GroupConfig::with_group("Space & Earth Science".into(), "science".into()),
                GroupConfig::with_group("Geodetic".into(), "geodetic".into()),
                GroupConfig::with_group("Engineering".into(), "engineering".into()),
                GroupConfig::with_group("Education".into(), "education".into()),
                GroupConfig::with_group("Military".into(), "military".into()),
                GroupConfig::with_group("Radar calibration".into(), "radar".into()),
                GroupConfig::with_group("CubeSats".into(), "cubesat".into()),
            ],
        }
    }
}
