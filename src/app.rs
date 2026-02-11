use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode},
};

mod exit;
mod home;

use crate::{
    app::{exit::Exit, home::Home},
    state::AppState,
    vchat::VChat,
};

pub const KEY_CODE_ACCEPT: KeyCode = KeyCode::Enter;
pub const KEY_CODE_DECLINE: KeyCode = KeyCode::Esc;

#[derive(Debug)]
pub struct App {
    state: AppState,
    vchat: Option<VChat>,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            vchat: None,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        match self.state {
            AppState::Home => frame.render_widget(Home, area),
            AppState::Exit => frame.render_widget(Exit, area),
        }
    }

    fn handle_event(&mut self) -> Result<()> {
        let event = event::read()?;

        match &self.state {
            AppState::Home => Home::handle_event(self, event)?,
            AppState::Exit => Exit::handle_event(self, event)?,
        };

        Ok(())
    }
}
