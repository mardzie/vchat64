use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event},
    style::Stylize,
    text::Line,
    widgets::{Block, Widget},
};

use crate::state::{AppState, CallMode};

#[derive(Debug)]
pub struct App {
    app_state: AppState,
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
            AppState::InCall(call_mode) => self.handle_event_in_call(event, call_mode.clone())?,
            AppState::Exit => self.exit()?,
        };

        Ok(())
    }

    fn handle_event_main(&mut self, event: Event) -> Result<()> {
        Ok(())
    }

    fn handle_event_in_call(&mut self, event: Event, call_mode: CallMode) -> Result<()> {
        Ok(())
    }

    fn exit(&mut self) -> Result<()> {
        self.exit = true;
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
            exit: false,
        }
    }
}
