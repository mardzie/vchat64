#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppState {
    #[default]
    App,
    CodeInput,
    Exit,
}
