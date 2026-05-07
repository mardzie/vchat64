use crate::app::state::{App, State};

impl State for App {
    fn handle_event(
        &mut self,
        ctx: &mut AppContext,
        event: crossterm::event::Event,
    ) -> Result<Option<super::AppState>> {
        todo!()
    }

    fn render(
        &self,
        ctx: &mut AppContext,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        todo!()
    }
}
