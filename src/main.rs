use anyhow::{Context as _, Result};
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
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Set the application's locale based on the system locale
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    rust_i18n::set_locale(&locale);

    let config = load_config().unwrap_or_else(|e| {
        eprintln!("Failed to load configuration: {e}");
        eprintln!("Using default configuration.");
        Config::default()
    });
    let mut app = App::with_config(config).context("failed to initialize application")?;
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
    Ok(config)
}
