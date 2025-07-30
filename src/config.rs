use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct WorldMapConfig {
    pub follow_selected_object: bool,
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
}

impl Default for SatelliteGroupsConfig {
    fn default() -> Self {
        Self {
            cache_lifetime_min: 2 * 60,
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub world_map: WorldMapConfig,
    pub satellite_groups: SatelliteGroupsConfig,
}
