use anyhow::Result;
use ratatui::{layout::Rect, Frame};

use crate::app::App;

pub mod object_information;
pub mod satellites;
pub mod track_map;

// - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
// - https://github.com/ratatui/ratatui/tree/master/examples

pub trait Component {
    fn render(&self, app: &App, frame: &mut Frame, area: Rect) -> Result<()>;
}
