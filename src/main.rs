use anyhow::Result;

use crate::app::App;

mod app;
mod event;
mod object;
mod satellite_group;
mod tui;
mod utils;
mod widgets;

#[tokio::main]
async fn main() -> Result<()> {
    App::new()?.run().await
}
