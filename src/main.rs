use anyhow::Result;
use rust_i18n::i18n;

i18n!("locales", fallback = "en");

use crate::{app::App, config::Config};

mod app;
mod config;
mod event;
mod group;
mod object;
mod tui;
mod utils;
mod widgets;

#[tokio::main]
async fn main() -> Result<()> {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"));
    rust_i18n::set_locale(&locale);

    let path = std::env::home_dir()
        .unwrap()
        .join(".config/tracker/config.toml");

    let config = if path.exists() {
        let content = std::fs::read_to_string(&path).expect("Failed to read config file");
        let config: Config = toml::from_str(&content).expect("Failed to parse config file");
        config
    } else {
        Config::default()
    };

    App::with_config(config)?.run().await
}
