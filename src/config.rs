use ratatui::style::Color;
use serde::Deserialize;

use crate::utils::Lla;

/// Configuration for the application.
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub world_map: WorldMapConfig,
    pub satellite_groups: SatelliteGroupsConfig,
    pub sky: SkyConfig,
}

/// Configuration for the world map widget.
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorldMapConfig {
    pub follow_object: bool,
    pub follow_smoothing: f64,
    pub show_terminator: bool,
    pub show_visibility_area: bool,

    pub lon_delta_deg: f64,
    pub time_delta_min: i64,

    pub map_color: Color,
    pub trajectory_color: Color,
    pub terminator_color: Color,
    pub visibility_area_color: Color,
}

impl Default for WorldMapConfig {
    fn default() -> Self {
        Self {
            follow_object: true,
            follow_smoothing: 0.3,
            show_terminator: true,
            show_visibility_area: true,
            time_delta_min: 1,
            lon_delta_deg: 10.0,
            map_color: Color::Gray,
            trajectory_color: Color::LightBlue,
            terminator_color: Color::DarkGray,
            visibility_area_color: Color::Yellow,
        }
    }
}

/// Configuration for satellite groups widget.
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SatelliteGroupsConfig {
    pub cache_lifetime_min: u64,
    pub groups: Vec<GroupConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
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

/// Configuration for the sky widget.
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SkyConfig {
    pub ground_station: Option<StationConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StationConfig {
    pub name: Option<String>,
    pub position: Lla,
}
