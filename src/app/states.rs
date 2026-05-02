use crate::app::{context::AppContext, state::AppState};
use color_eyre::Result;

pub trait State {
    fn handle_event(
        &mut self,
        ctx: &mut AppContext,
        event: crossterm::event::Event,
    ) -> Result<Option<AppState>>;

    fn render(
        &self,
        ctx: &mut AppContext,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    );
}
