use ratatui::style::Color;
use serde::Deserialize;

use crate::coordinates::Lla;

/// Configuration for the application.
#[derive(Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub world_map: WorldMapConfig,
    pub satellite_groups: SatelliteGroupsConfig,
    pub sky: SkyConfig,
    pub timeline: TimelineConfig,
}

/// Configuration for the world map widget.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorldMapConfig {
    pub follow_object: bool,
    pub follow_smoothing: f64,
    pub show_terminator: bool,
    pub show_visibility_area: bool,
    pub lon_delta_deg: f64,
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
            lon_delta_deg: 10.0,
            map_color: Color::Gray,
            trajectory_color: Color::LightBlue,
            terminator_color: Color::DarkGray,
            visibility_area_color: Color::Yellow,
        }
    }
}

/// Configuration for satellite groups widget.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SatelliteGroupsConfig {
    pub cache_lifetime_mins: u64,
    pub groups: Vec<GroupConfig>,
}

#[derive(Clone, Deserialize)]
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
            cache_lifetime_mins: 2 * 60,
            groups: vec![
                // Specific objects of interest
                GroupConfig::with_id("ISS".into(), "1998-067A".into()),
                GroupConfig::with_id("CSS".into(), "2021-035A".into()),
                // Special-interest satellites
                GroupConfig::with_group("Last 30 Days".into(), "last-30-days".into()),
                GroupConfig::with_group("Space Stations".into(), "stations".into()),
                GroupConfig::with_group("100 Brightest".into(), "visual".into()),
                // Weather & Earth resources satellites
                GroupConfig::with_group("Weather".into(), "weather".into()),
                GroupConfig::with_group("Earth Resources".into(), "resource".into()),
                GroupConfig::with_group("SARSAT".into(), "sarsat".into()),
                GroupConfig::with_group("Disaster Monitoring".into(), "dmc".into()),
                GroupConfig::with_group("TDRSS".into(), "tdrss".into()),
                GroupConfig::with_group("ARGOS".into(), "argos".into()),
                GroupConfig::with_group("Planet".into(), "planet".into()),
                GroupConfig::with_group("Spire".into(), "spire".into()),
                // Communications satellites
                GroupConfig::with_group("GEO".into(), "geo".into()),
                GroupConfig::with_group("GPZ".into(), "gpz".into()),
                GroupConfig::with_group("GPZ+".into(), "gpz-plus".into()),
                GroupConfig::with_group("Intelsat".into(), "intelsat".into()),
                GroupConfig::with_group("SES".into(), "ses".into()),
                GroupConfig::with_group("Eutelsat".into(), "eutelsat".into()),
                GroupConfig::with_group("Telesat".into(), "telesat".into()),
                GroupConfig::with_group("Starlink".into(), "starlink".into()),
                GroupConfig::with_group("OneWeb".into(), "oneweb".into()),
                GroupConfig::with_group("Qianfan".into(), "qianfan".into()),
                GroupConfig::with_group("Hulianwang Digui".into(), "hulianwang".into()),
                GroupConfig::with_group("Kuiper".into(), "kuiper".into()),
                GroupConfig::with_group("Iridium NEXT".into(), "iridium-NEXT".into()),
                GroupConfig::with_group("Orbcomm".into(), "orbcomm".into()),
                GroupConfig::with_group("Globalstar".into(), "globalstar".into()),
                GroupConfig::with_group("Amateur Radio".into(), "amateur".into()),
                GroupConfig::with_group("SatNOGS".into(), "satnogs".into()),
                GroupConfig::with_group("Experimental Comm".into(), "x-comm".into()),
                GroupConfig::with_group("Other Comm".into(), "other-comm".into()),
                // Navigation satellites
                GroupConfig::with_group("GNSS".into(), "gnss".into()),
                GroupConfig::with_group("GPS Ops".into(), "gps-ops".into()),
                GroupConfig::with_group("GLONASS Ops".into(), "glo-ops".into()),
                GroupConfig::with_group("Galileo".into(), "galileo".into()),
                GroupConfig::with_group("Beidou".into(), "beidou".into()),
                GroupConfig::with_group("SBAS".into(), "sbas".into()),
                // Scientific satellites
                GroupConfig::with_group("Space & Earth Science".into(), "science".into()),
                GroupConfig::with_group("Geodetic".into(), "geodetic".into()),
                GroupConfig::with_group("Engineering".into(), "engineering".into()),
                GroupConfig::with_group("Education".into(), "education".into()),
                // Miscellaneous satellites
                GroupConfig::with_group("Military".into(), "military".into()),
                GroupConfig::with_group("Radar Calibration".into(), "radar".into()),
                GroupConfig::with_group("CubeSats".into(), "cubesat".into()),
                // Debris
                GroupConfig::with_group("Fengyun 1C Debris".into(), "fengyun-1c-debris".into()),
                GroupConfig::with_group("Iridium 33 Debris".into(), "iridium-33-debris".into()),
                GroupConfig::with_group("Cosmos 2251 Debris".into(), "cosmos-2251-debris".into()),
            ],
        }
    }
}

/// Configuration for the sky widget.
#[derive(Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SkyConfig {
    pub ground_station: Option<GroundStationConfig>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundStationConfig {
    pub name: Option<String>,
    pub position: Lla,
}

/// Configuration for the timeline widget.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TimelineConfig {
    pub time_delta_mins: i64,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self { time_delta_mins: 1 }
    }
}
