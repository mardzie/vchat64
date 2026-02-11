#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppState {
    #[default]
    Main,
    Exit(bool),
}
