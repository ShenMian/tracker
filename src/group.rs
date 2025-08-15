use std::time::Duration;

use tokio::fs;

use crate::config::GroupConfig;

/// The `Group` type.
///
/// Type [`Group`] represents a group of satellites.
#[derive(Clone, Eq, Debug)]
pub struct Group {
    label: String,
    celestrak_id: CelestrakId,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum CelestrakId {
    /// COSPAR ID.
    Id(String),
    /// Group name.
    Group(String),
}

impl Group {
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
            CelestrakId::Id(id) => request.query(&[("INTDES", id)]),
            CelestrakId::Group(group) => request.query(&[("GROUP", group)]),
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

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.celestrak_id == other.celestrak_id
    }
}

impl From<GroupConfig> for Group {
    fn from(config: GroupConfig) -> Self {
        match (config.id, config.group) {
            (Some(id), None) => Self {
                label: config.label,
                celestrak_id: CelestrakId::Id(id),
            },
            (None, Some(group)) => Self {
                label: config.label,
                celestrak_id: CelestrakId::Group(group),
            },
            _ => unreachable!(),
        }
    }
}
