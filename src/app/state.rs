use crate::app::state::{app::App, code_input::CodeInput, exit::Exit};

mod app;
mod code_input;
mod exit;

pub trait State {
    fn handle_event(
        &mut self,
        ctx: &mut crate::app::context::AppContext,
        event: crossterm::event::Event,
    ) -> color_eyre::Result<Option<AppState>>;

    fn render(
        &self,
        ctx: &mut crate::app::context::AppContext,
        area: ratatui::layout::Rect,
        buf: &mut ratatui::buffer::Buffer,
    );
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppState {
    App(App),
    CodeInput(CodeInput),
    Exit(Exit),
}

impl AppState {
    pub fn new_app() -> Self {
        Self::App(App::default())
    }

    pub fn new_code_input() -> Self {
        Self::CodeInput(CodeInput::default())
    }

    pub fn new_exit() -> Self {
        Self::Exit(Exit::default())
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::App(App::default())
    }
}
