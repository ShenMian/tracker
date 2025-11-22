use std::{fmt::Display, time::Duration};

use tokio::fs;

use crate::config::GroupConfig;

/// The `Group` type.
///
/// Type [`Group`] represents a group of satellites.
#[derive(Clone, Eq, Debug)]
pub struct Group {
    label: String,
    identifier: Identifier,
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
        let cache_path = std::env::temp_dir().join(format!(
            "tracker/{}.json",
            self.identifier.to_string().to_lowercase()
        ));
        fs::create_dir_all(cache_path.parent().unwrap())
            .await
            .unwrap();

        // Fetch elements if cache doesn't exist
        if !fs::try_exists(&cache_path).await.unwrap() {
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
        const TIMEOUT_SECS: u64 = 10;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .build()
            .ok()?;

        let mut request = client.get(URL).query(&[("FORMAT", "json")]);
        request = match &self.identifier {
            Identifier::Id(id) => request.query(&[("INTDES", id)]),
            Identifier::Group(group) => request.query(&[("GROUP", group)]),
        };

        let response = match request.send().await {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Failed to fetch from celestrak.org: {}", e);
                return None;
            }
        };

        match response.json().await {
            Ok(data) => Some(data),
            Err(e) => {
                eprintln!("Failed to parse JSON from celestrak.org: {}", e);
                None
            }
        }
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl From<GroupConfig> for Group {
    fn from(config: GroupConfig) -> Self {
        match (config.id, config.group) {
            (Some(id), None) => Self {
                label: config.label,
                identifier: Identifier::Id(id),
            },
            (None, Some(group)) => Self {
                label: config.label,
                identifier: Identifier::Group(group),
            },
            _ => panic!("invalid `satellite_groups.groups` configuration"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
enum Identifier {
    /// COSPAR ID.
    Id(String),
    /// Group name.
    Group(String),
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Identifier::Id(id) => write!(f, "{id}"),
            Identifier::Group(group) => write!(f, "{group}"),
        }
    }
}
