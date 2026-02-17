#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum Event {
    Crossterm(ratatui::crossterm::event::Event),
    ReDraw,
}

impl From<ratatui::crossterm::event::Event> for Event {
    fn from(event: ratatui::crossterm::event::Event) -> Self {
        Self::Crossterm(event)
    }
}
