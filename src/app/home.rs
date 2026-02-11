use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{crossterm::event::Event, style::Stylize, widgets::Widget};

use crate::app::App;

#[derive(Debug)]
pub(super) struct Home;

impl Home {
    pub fn handle_event(app: &mut App, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => Self::handle_key_event(app, key_event)?,
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Result<()> {
        if key_event.is_press() {
            match key_event.code {
                KeyCode::Char('q') => {
                    app.state = crate::state::AppState::Exit;
                }
                _ => {}
            };
        };

        Ok(())
    }
}

impl Widget for Home {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        use ratatui::text::Line;

        Line::from("VChat64").bold().yellow().render(area, buf);
    }
}
