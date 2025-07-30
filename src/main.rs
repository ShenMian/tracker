use anyhow::Result;

use crate::{app::App, config::Config};

mod app;
mod config;
mod event;
mod object;
mod satellite_group;
mod tui;
mod utils;
mod widgets;

#[tokio::main]
async fn main() -> Result<()> {
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
