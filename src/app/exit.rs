use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    crossterm::event::Event,
    text::{Line, Text},
    widgets::{Block, Widget},
};

use crate::{
    app::{App, KEY_CODE_ACCEPT, KEY_CODE_DECLINE},
    state::AppState,
};

#[derive(Debug)]
pub(super) struct Exit;

impl Exit {
    pub fn handle_event(app: &mut App, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => Self::handle_key_event(app, key_event)?,
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(app: &mut App, key_event: KeyEvent) -> Result<()> {
        if key_event.code == KEY_CODE_ACCEPT {
            app.exit = true;
        } else if key_event.code == KEY_CODE_DECLINE {
            app.state = AppState::Home;
        };

        Ok(())
    }
}

impl Widget for Exit {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" VChat64 ");
        let text = Text::from("Quit?").centered();
        let block = Block::bordered().title(title);

        block.render(area, buf);
    }
}
