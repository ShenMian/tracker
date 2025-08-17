use anyhow::Result;
use rust_i18n::i18n;

i18n!("locales", fallback = "en");

mod app;
mod config;
mod event;
mod group;
mod object;
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

    let config = load_config();
    App::with_config(config)?.run().await
}

fn load_config() -> Config {
    let path = std::env::home_dir()
        .unwrap()
        .join(".config/tracker/config.toml");
    if !path.exists() {
        return Config::default();
    }

    let content = std::fs::read_to_string(&path).expect("failed to read config file");
    let config: Config = toml::from_str(&content).expect("failed to parse config file");
    config
}
