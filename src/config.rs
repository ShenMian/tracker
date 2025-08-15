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
                SatelliteGroup::with_cospar_id("ISS".into(), "1998-067A".into()),
                SatelliteGroup::with_cospar_id("CSS".into(), "2021-035A".into()),
                SatelliteGroup::with_group_name("Weather".into(), "weather".into()),
                SatelliteGroup::with_group_name("NOAA".into(), "noaa".into()),
                SatelliteGroup::with_group_name("GOES".into(), "goes".into()),
                SatelliteGroup::with_group_name("Earth resources".into(), "resource".into()),
                SatelliteGroup::with_group_name("Search & rescue".into(), "sarsat".into()),
                SatelliteGroup::with_group_name("Disaster monitoring".into(), "dmc".into()),
                SatelliteGroup::with_group_name("GPS Operational".into(), "gps-ops".into()),
                SatelliteGroup::with_group_name("GLONASS Operational".into(), "glo-ops".into()),
                SatelliteGroup::with_group_name("Galileo".into(), "galileo".into()),
                SatelliteGroup::with_group_name("Beidou".into(), "beidou".into()),
                SatelliteGroup::with_group_name("Space & Earth Science".into(), "science".into()),
                SatelliteGroup::with_group_name("Geodetic".into(), "geodetic".into()),
                SatelliteGroup::with_group_name("Engineering".into(), "engineering".into()),
                SatelliteGroup::with_group_name("Education".into(), "education".into()),
                SatelliteGroup::with_group_name("Military".into(), "military".into()),
                SatelliteGroup::with_group_name("Radar calibration".into(), "radar".into()),
                SatelliteGroup::with_group_name("CubeSats".into(), "cubesat".into()),
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
