use std::time::Duration;

use tokio::fs;

use crate::config::SatelliteGroupConfig;

/// The `SatelliteGroup` type.
///
/// Type [`SatelliteGroup`] represents a group of satellites.
#[derive(Clone, Eq, Debug)]
pub struct SatelliteGroup {
    label: String,
    celestrak_id: CelestrakId,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum CelestrakId {
    CosparId(String),
    GroupName(String),
}

impl SatelliteGroup {
    /// Creates a new `SatelliteGroup`.
    pub fn new(label: String, celestrak_id: CelestrakId) -> Self {
        Self {
            label,
            celestrak_id,
        }
    }

    /// Returns the label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns SGP4 elements.
    ///
    /// If cache is expired, fetches elements from <https://celestrak.org>.
    /// Otherwise, reads elements from cache.
    ///
    /// # Arguments
    ///
    /// * `cache_lifetime` - Duration for which the cache is considered valid.
    pub async fn get_elements(&self, cache_lifetime: Duration) -> Option<Vec<sgp4::Elements>> {
        let cache_path =
            std::env::temp_dir().join(format!("tracker/{}.json", self.label().to_lowercase()));
        fs::create_dir_all(cache_path.parent().unwrap())
            .await
            .unwrap();

        // Fetch elements if cache doesn't exist
        if !std::fs::exists(&cache_path).unwrap() {
            if let Some(elements) = self.fetch_elements().await {
                fs::write(&cache_path, serde_json::to_string(&elements).unwrap())
                    .await
                    .unwrap();
            } else {
                return None;
            }
        }

        let age = fs::metadata(&cache_path)
            .await
            .unwrap()
            .modified()
            .unwrap()
            .elapsed()
            .unwrap();
        let is_cache_expired = age > cache_lifetime;

        // Fetch elements if cache is expired
        if is_cache_expired && let Some(elements) = self.fetch_elements().await {
            fs::write(&cache_path, serde_json::to_string(&elements).unwrap())
                .await
                .unwrap();
        }

        let json = fs::read_to_string(&cache_path).await.unwrap();
        serde_json::from_str(&json).unwrap()
    }

    /// Fetches SGP4 elements from <https://celestrak.org>.
    async fn fetch_elements(&self) -> Option<Vec<sgp4::Elements>> {
        const URL: &str = "https://celestrak.com/NORAD/elements/gp.php";

        let mut request = reqwest::Client::new().get(URL).query(&[("FORMAT", "json")]);
        request = match &self.celestrak_id {
            CelestrakId::CosparId(id) => request.query(&[("INTDES", id)]),
            CelestrakId::GroupName(group) => request.query(&[("GROUP", group)]),
        };

        let response = request.send().await.ok()?;
        Some(
            response
                .json()
                .await
                .expect("failed to parse JSON from celestrak.org"),
        )
    }
}

impl PartialEq for SatelliteGroup {
    fn eq(&self, other: &Self) -> bool {
        self.celestrak_id == other.celestrak_id
    }
}

impl From<SatelliteGroupConfig> for SatelliteGroup {
    fn from(config: SatelliteGroupConfig) -> Self {
        match (config.id, config.group) {
            (Some(cospar_id), None) => Self::new(config.label, CelestrakId::CosparId(cospar_id)),
            (None, Some(group_name)) => Self::new(config.label, CelestrakId::GroupName(group_name)),
            _ => unreachable!(),
        }
    }
}
