use crate::app::state::State;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct CodeInput;

impl State for CodeInput {
    fn handle_event(
        &mut self,
        ctx: &mut crate::app::context::AppContext,
        event: crossterm::event::Event,
    ) -> color_eyre::Result<Option<super::AppState>> {
        todo!()
    }

    fn render(
        &self,
        ctx: &mut crate::app::context::AppContext,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        todo!()
    }
}
