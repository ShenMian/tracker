use anyhow::{Context as _, Result, bail};
use clap::Parser;
use rust_i18n::i18n;

i18n!("locales", fallback = "en");

mod app;
mod config;
mod coordinates;
mod event;
mod group;
mod object;
mod shared_state;
mod tui;
mod utils;
mod widgets;

use app::App;
use config::{Config, SatelliteGroupsConfig};

#[derive(Parser)]
#[command(version)]
struct Args {
    /// Load this group at start, by its config label. Repeatable.
    #[arg(long = "group", value_name = "LABEL")]
    groups: Vec<String>,
    /// Select the first object of this group. Must be a --group label too.
    #[arg(long, value_name = "LABEL")]
    follow: Option<String>,
}

impl Args {
    /// Checks the labels against the configured groups.
    fn validate(&self, config: &SatelliteGroupsConfig) -> Result<()> {
        let labels: Vec<&str> = config.groups.iter().map(|g| g.label.as_str()).collect();
        for group in &self.groups {
            if !labels.contains(&group.as_str()) {
                bail!(
                    "unknown group \"{group}\", available: {}",
                    labels.join(", ")
                );
            }
        }
        if let Some(follow) = &self.follow
            && !self.groups.contains(follow)
        {
            bail!("--follow {follow} also needs --group {follow}");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Set the application's locale based on the system locale
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    rust_i18n::set_locale(&locale);

    let config = load_config().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {e}");
        eprintln!("Using default configuration.");
        Config::default()
    });
    args.validate(&config.satellite_groups)?;
    let mut app = App::with_config(config).context("failed to initialize application")?;
    let groups = &mut app.states.satellite_groups_state;
    groups.start_with(&args.groups, args.follow.as_deref());
    groups.reload_selected_entries();
    app.run().await
}

fn load_config() -> Result<Config> {
    let path = std::env::home_dir()
        .context("failed to get home directory")?
        .join(".config/tracker/config.toml");
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;
    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(flags: &[&str]) -> Args {
        Args::try_parse_from(std::iter::once(&"tracker").chain(flags)).unwrap()
    }

    fn default_groups() -> SatelliteGroupsConfig {
        Config::default().satellite_groups
    }

    /// A config whose only group is labelled `Hubble`.
    fn hubble_only() -> SatelliteGroupsConfig {
        let config: Config = toml::from_str(
            r#"[satellite_groups]
groups = [{ label = "Hubble", id = "1990-037B" }]"#,
        )
        .unwrap();
        config.satellite_groups
    }

    #[test]
    fn no_flags_are_valid() {
        assert!(args(&[]).validate(&default_groups()).is_ok());
    }

    #[test]
    fn configured_groups_and_a_followed_group_are_valid() {
        let args = args(&["--group", "ISS", "--group", "CSS", "--follow", "ISS"]);
        assert!(args.validate(&default_groups()).is_ok());
    }

    #[test]
    fn unknown_group_error_names_it_and_lists_the_labels() {
        let message = args(&["--group", "NOPE"])
            .validate(&default_groups())
            .unwrap_err()
            .to_string();
        assert!(message.contains("NOPE"), "{message}");
        assert!(message.contains("ISS"), "{message}");
        assert!(message.contains("Weather"), "{message}");
    }

    #[test]
    fn group_labels_must_match_exactly() {
        for label in ["iss", "IS", "ISSS"] {
            let result = args(&["--group", label]).validate(&default_groups());
            assert!(result.is_err(), "{label} was accepted");
        }
    }

    #[test]
    fn every_group_is_checked_not_only_the_first() {
        let args = args(&["--group", "ISS", "--group", "NOPE"]);
        assert!(args.validate(&default_groups()).is_err());
    }

    #[test]
    fn followed_group_must_be_one_of_the_groups() {
        let message = args(&["--group", "CSS", "--follow", "ISS"])
            .validate(&default_groups())
            .unwrap_err()
            .to_string();
        assert!(message.contains("ISS"), "{message}");
    }

    #[test]
    fn follow_alone_is_rejected() {
        let result = args(&["--follow", "ISS"]).validate(&default_groups());
        assert!(result.is_err());
    }

    #[test]
    fn labels_come_from_the_config_not_the_defaults() {
        assert!(
            args(&["--group", "Hubble"])
                .validate(&hubble_only())
                .is_ok()
        );
        assert!(args(&["--group", "ISS"]).validate(&hubble_only()).is_err());
    }
}
