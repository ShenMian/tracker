use std::fmt::Display;

use ratatui::style::Color;
use serde::Deserialize;
use thiserror::Error;

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

impl Config {
    pub fn validate(&self) -> Result<(), ConfigError> {
        self.world_map.validate()?;
        self.sky.validate()?;
        self.timeline.validate()?;
        Ok(())
    }
}

/// Configuration for the world map widget.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct WorldMapConfig {
    pub follow_object: bool,
    pub follow_smoothing: f64,
    pub show_terminator: bool,
    pub show_subsolar_point: bool,
    pub show_night_shading: bool,
    pub show_visibility_area: bool,
    pub lon_delta_deg: f64,
    #[serde(alias = "map_color")]
    pub coast_color: Color,
    /// Background colour of the lit (day) side of the Earth.
    pub day_color: Color,
    pub trajectory_color: Color,
    pub terminator_color: Color,
    pub visibility_area_color: Color,
}

impl WorldMapConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(0.0..=1.0).contains(&self.follow_smoothing) {
            return Err(ConfigError::InvalidFollowSmoothing(self.follow_smoothing));
        }
        if self.lon_delta_deg <= 0.0 {
            return Err(ConfigError::InvalidLongitudeDelta(self.lon_delta_deg));
        }
        Ok(())
    }
}

impl Default for WorldMapConfig {
    fn default() -> Self {
        Self {
            follow_object: true,
            follow_smoothing: 0.3,
            show_subsolar_point: true,
            show_night_shading: true,
            show_visibility_area: true,
            show_terminator: false,
            lon_delta_deg: 10.0,
            coast_color: Color::Gray,
            day_color: Color::Rgb(0x14, 0x1d, 0x33),
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
    #[serde(flatten)]
    pub identifier: GroupIdentifier,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deserialize)]
pub enum GroupIdentifier {
    /// COSPAR ID.
    #[serde(rename = "id")]
    CosparId(String),
    /// Group name.
    #[serde(rename = "group")]
    Group(String),
}

impl Display for GroupIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CosparId(id) => write!(f, "{id}"),
            Self::Group(group) => write!(f, "{group}"),
        }
    }
}

impl GroupConfig {
    fn with_id(label: String, cospar_id: String) -> Self {
        Self {
            label,
            identifier: GroupIdentifier::CosparId(cospar_id),
        }
    }

    fn with_group(label: String, group_name: String) -> Self {
        Self {
            label,
            identifier: GroupIdentifier::Group(group_name),
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
                GroupConfig::with_group("Active".into(), "active".into()),
                GroupConfig::with_group("Analyst".into(), "analyst".into()),
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

impl SkyConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if let Some(ground_station) = &self.ground_station {
            ground_station.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroundStationConfig {
    pub name: Option<String>,
    pub position: Lla,
}

impl GroundStationConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(-90.0..=90.0).contains(&self.position.lat) {
            return Err(ConfigError::InvalidLatitude(self.position.lat));
        }
        if !(-180.0..=180.0).contains(&self.position.lon) {
            return Err(ConfigError::InvalidLongitude(self.position.lon));
        }
        if self.position.alt < 0.0 {
            return Err(ConfigError::InvalidAltitude(self.position.alt));
        }

        Ok(())
    }
}

/// Configuration for the timeline widget.
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TimelineConfig {
    pub time_delta_mins: i64,
}

impl TimelineConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.time_delta_mins <= 0 {
            return Err(ConfigError::InvalidTimeDelta(self.time_delta_mins));
        }
        Ok(())
    }
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self { time_delta_mins: 1 }
    }
}

/// An error encountered while validating the application configuration.
#[derive(Clone, Debug, Error, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum ConfigError {
    #[error("follow_smoothing must be between 0 and 1, got {0}")]
    InvalidFollowSmoothing(f64),
    #[error("lon_delta_deg must be a finite number greater than 0, got {0}")]
    InvalidLongitudeDelta(f64),
    #[error("latitude must be between -90 and 90, got {0}")]
    InvalidLatitude(f64),
    #[error("longitude must be between -180 and 180, got {0}")]
    InvalidLongitude(f64),
    #[error("altitude must be a finite non-negative number, got {0}")]
    InvalidAltitude(f64),
    #[error("time_delta_mins must be greater than 0, got {0}")]
    InvalidTimeDelta(i64),
}

#[test]
fn default_config_is_valid() {
    assert_eq!(Config::default().validate(), Ok(()));
}
