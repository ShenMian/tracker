use serde::Deserialize;

use crate::satellite_group::SatelliteGroup;

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
    pub groups: Vec<SatelliteGroup>,
}

impl Default for SatelliteGroupsConfig {
    fn default() -> Self {
        Self {
            cache_lifetime_min: 2 * 60,
            groups: vec![
                SatelliteGroup {
                    label: "ISS".into(),
                    cospar_id: Some("1998-067A".into()),
                    group_name: None,
                },
                SatelliteGroup {
                    label: "CSS".into(),
                    cospar_id: Some("2021-035A".into()),
                    group_name: None,
                },
                SatelliteGroup {
                    label: "Weather".into(),
                    cospar_id: None,
                    group_name: Some("weather".into()),
                },
                SatelliteGroup {
                    label: "NOAA".into(),
                    cospar_id: None,
                    group_name: Some("noaa".into()),
                },
                SatelliteGroup {
                    label: "GOES".into(),
                    cospar_id: None,
                    group_name: Some("goes".into()),
                },
                SatelliteGroup {
                    label: "Earth resources".into(),
                    cospar_id: None,
                    group_name: Some("resource".into()),
                },
                SatelliteGroup {
                    label: "Search & rescue".into(),
                    cospar_id: None,
                    group_name: Some("sarsat".into()),
                },
                SatelliteGroup {
                    label: "Disaster monitoring".into(),
                    cospar_id: None,
                    group_name: Some("dmc".into()),
                },
                SatelliteGroup {
                    label: "GPS Operational".into(),
                    cospar_id: None,
                    group_name: Some("gps-ops".into()),
                },
                SatelliteGroup {
                    label: "GLONASS Operational".into(),
                    cospar_id: None,
                    group_name: Some("glo-ops".into()),
                },
                SatelliteGroup {
                    label: "Galileo".into(),
                    cospar_id: None,
                    group_name: Some("galileo".into()),
                },
                SatelliteGroup {
                    label: "Beidou".into(),
                    cospar_id: None,
                    group_name: Some("beidou".into()),
                },
                SatelliteGroup {
                    label: "Space & Earth Science".into(),
                    cospar_id: None,
                    group_name: Some("science".into()),
                },
                SatelliteGroup {
                    label: "Geodetic".into(),
                    cospar_id: None,
                    group_name: Some("geodetic".into()),
                },
                SatelliteGroup {
                    label: "Engineering".into(),
                    cospar_id: None,
                    group_name: Some("engineering".into()),
                },
                SatelliteGroup {
                    label: "Education".into(),
                    cospar_id: None,
                    group_name: Some("education".into()),
                },
                SatelliteGroup {
                    label: "Military".into(),
                    cospar_id: None,
                    group_name: Some("military".into()),
                },
                SatelliteGroup {
                    label: "Radar calibration".into(),
                    cospar_id: None,
                    group_name: Some("radar".into()),
                },
                SatelliteGroup {
                    label: "CubeSats".into(),
                    cospar_id: None,
                    group_name: Some("cubesat".into()),
                },
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
