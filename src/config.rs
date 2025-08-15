use serde::Deserialize;

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
    pub groups: Vec<SatelliteGroupConfig>,
}

#[derive(Deserialize)]
pub struct SatelliteGroupConfig {
    pub label: String,
    pub id: Option<String>,
    pub group: Option<String>,
}

impl SatelliteGroupConfig {
    fn with_cospar_id(label: String, cospar_id: String) -> Self {
        Self {
            label,
            id: Some(cospar_id),
            group: None,
        }
    }

    fn with_group_name(label: String, group_name: String) -> Self {
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
                SatelliteGroupConfig::with_cospar_id("ISS".into(), "1998-067A".into()),
                SatelliteGroupConfig::with_cospar_id("CSS".into(), "2021-035A".into()),
                SatelliteGroupConfig::with_group_name("Weather".into(), "weather".into()),
                SatelliteGroupConfig::with_group_name("NOAA".into(), "noaa".into()),
                SatelliteGroupConfig::with_group_name("GOES".into(), "goes".into()),
                SatelliteGroupConfig::with_group_name("Earth resources".into(), "resource".into()),
                SatelliteGroupConfig::with_group_name("Search & rescue".into(), "sarsat".into()),
                SatelliteGroupConfig::with_group_name("Disaster monitoring".into(), "dmc".into()),
                SatelliteGroupConfig::with_group_name("GPS Operational".into(), "gps-ops".into()),
                SatelliteGroupConfig::with_group_name(
                    "GLONASS Operational".into(),
                    "glo-ops".into(),
                ),
                SatelliteGroupConfig::with_group_name("Galileo".into(), "galileo".into()),
                SatelliteGroupConfig::with_group_name("Beidou".into(), "beidou".into()),
                SatelliteGroupConfig::with_group_name(
                    "Space & Earth Science".into(),
                    "science".into(),
                ),
                SatelliteGroupConfig::with_group_name("Geodetic".into(), "geodetic".into()),
                SatelliteGroupConfig::with_group_name("Engineering".into(), "engineering".into()),
                SatelliteGroupConfig::with_group_name("Education".into(), "education".into()),
                SatelliteGroupConfig::with_group_name("Military".into(), "military".into()),
                SatelliteGroupConfig::with_group_name("Radar calibration".into(), "radar".into()),
                SatelliteGroupConfig::with_group_name("CubeSats".into(), "cubesat".into()),
            ],
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub world_map: WorldMapConfig,
    pub satellite_groups: SatelliteGroupsConfig,
}
