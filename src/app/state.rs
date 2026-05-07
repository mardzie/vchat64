use crate::app::state::{app::App, code_input::CodeInput, exit::Exit};

pub(super) mod app;
pub(super) mod code_input;
pub(super) mod exit;

pub(super) trait State {
    fn handle_event(
        &self,
        ctx: &mut crate::app::context::AppContext,
        event: &crossterm::event::Event,
    ) -> color_eyre::Result<()>;

    fn render(
        &self,
        ctx: &crate::app::context::AppContext,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    App(App),
    CodeInput(CodeInput),
    Exit(Exit),
}

impl AppState {
    pub fn app() -> Self {
        Self::App(App)
    }

    pub fn code_input() -> Self {
        Self::CodeInput(CodeInput)
    }

    pub fn exit() -> Self {
        Self::Exit(Exit)
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::app()
    }
}
