use std::time::Duration;

use serde::Deserialize;
use tokio::fs;

/// The `SatelliteGroup` type.
///
/// Type [`SatelliteGroup`] represents a group of satellites.
#[derive(Clone, Debug, Deserialize)]
pub struct SatelliteGroup {
    label: String,
    cospar_id: Option<String>,
    group_name: Option<String>,
}

impl SatelliteGroup {
    /// Creates a new `SatelliteGroup` with the given COSPAR ID.
    pub fn with_cospar_id(label: String, cospar_id: String) -> Self {
        Self {
            label,
            cospar_id: Some(cospar_id),
            group_name: None,
        }
    }

    /// Creates a new `SatelliteGroup` with the given group name.
    pub fn with_group_name(label: String, group_name: String) -> Self {
        Self {
            label,
            cospar_id: None,
            group_name: Some(group_name),
        }
    }

    /// Returns the label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the international designator.
    fn cospar_id(&self) -> Option<&str> {
        self.cospar_id.as_deref()
    }

    /// Returns CelesTrak group name.
    fn group_name(&self) -> Option<&str> {
        self.group_name.as_deref()
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
            std::env::temp_dir().join(format!("tracker/{}.json", self.label.to_lowercase()));
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
        request = match (self.cospar_id(), self.group_name()) {
            (Some(id), None) => request.query(&[("INTDES", id)]),
            (None, Some(group)) => request.query(&[("GROUP", group)]),
            _ => unreachable!(),
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
