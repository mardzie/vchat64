mod app;
mod code_input;
mod exit;

#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppState {
    #[default]
    App(App),
    CodeInput(CodeInput),
    Exit(Exit),
}

#[derive(Debug, PartialEq, Eq)]
pub struct App;

#[derive(Debug, PartialEq, Eq)]
pub struct CodeInput;

#[derive(Debug, PartialEq, Eq)]
pub struct Exit;

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
