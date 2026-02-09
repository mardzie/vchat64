#[derive(Debug, Default, PartialEq, Eq)]
pub enum AppState {
    #[default]
    Main,
    InCall(CallMode),
    Exit,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum CallMode {
    #[default]
    Call,
    AddToCall,
}
