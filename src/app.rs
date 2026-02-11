use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    style::Stylize,
    text::Line,
    widgets::{Block, Widget},
};

use crate::{state::AppState, vchat::VChat};

pub const KEY_CODE_ACCEPT: KeyCode = KeyCode::Enter;

#[derive(Debug)]
pub struct App {
    app_state: AppState,
    vchat: Option<VChat>,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_event(&mut self) -> Result<()> {
        let event = event::read()?;
        match &self.app_state {
            AppState::Main => self.handle_event_main(event)?,
            AppState::Exit(_) => self.exit(event)?,
        };

        Ok(())
    }

    fn handle_event_main(&mut self, event: Event) -> Result<()> {
        Ok(())
    }

    fn exit(&mut self, event: Event) -> Result<()> {
        match &mut self.app_state {
            AppState::Exit(confirmed) => {
                if *confirmed {
                    self.exit = true;
                } else {
                    if let Event::Key(key) = event {
                        if !key.is_press() {
                            return Ok(());
                        };

                        if key.code == KEY_CODE_ACCEPT {
                            *confirmed = true;
                        };
                    }
                }
            }
            _ => panic!("Exit should have not been called, only if the `AppState` is `Exit`"),
        };

        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from(" VChat64 ").bold();
        let block = Block::bordered().title(title);
        
        

        block.render(area, buf);
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            app_state: Default::default(),
            vchat: None,
            exit: false,
        }
    }
}
