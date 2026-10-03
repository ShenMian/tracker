use std::{sync::LazyLock, time::Duration};

use tokio::fs;

use crate::config::{GroupConfig, GroupIdentifier};

/// The timeout duration for HTTP requests.
const HTTP_TIMEOUT_SECS: u64 = 10;

/// The shared HTTP client used for making requests.
static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(HTTP_TIMEOUT_SECS))
        .build()
        .expect("failed to create HTTP client")
});

/// The `Group` type.
///
/// Type [`Group`] represents a group of satellites.
#[derive(Clone, Eq, Debug)]
pub struct Group {
    label: String,
    identifier: GroupIdentifier,
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

        // Check if cache needs refresh (doesn't exist or expired)
        let needs_refresh = !fs::try_exists(&cache_path).await.unwrap()
            || fs::metadata(&cache_path)
                .await
                .unwrap()
                .modified()
                .unwrap()
                .elapsed()
                .unwrap()
                > cache_lifetime;

        if needs_refresh {
            let elements = self.fetch_elements().await?;
            let json = serde_json::to_string(&elements).unwrap();
            fs::write(&cache_path, json).await.unwrap();
        }

        let json = fs::read_to_string(&cache_path).await.unwrap();
        serde_json::from_str(&json).expect("failed to parse cache")
    }

    /// Fetches SGP4 elements from <https://celestrak.org>.
    async fn fetch_elements(&self) -> Option<Vec<sgp4::Elements>> {
        const URL: &str = "https://celestrak.org/NORAD/elements/gp.php";

        let mut request = HTTP_CLIENT.get(URL).query(&[("FORMAT", "json")]);
        request = match &self.identifier {
            GroupIdentifier::CosparId(id) => request.query(&[("INTDES", id)]),
            GroupIdentifier::Group(group) => request.query(&[("GROUP", group)]),
        };

        let response = request
            .send()
            .await
            .inspect_err(|e| eprintln!("Failed to fetch from celestrak.org: {}", e))
            .ok()?;

        response
            .json()
            .await
            .inspect_err(|e| eprintln!("Failed to parse JSON from celestrak.org: {}", e))
            .ok()
    }
}

impl PartialEq for Group {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl From<GroupConfig> for Group {
    fn from(config: GroupConfig) -> Self {
        Self {
            label: config.label,
            identifier: config.identifier,
        }
    }
}
