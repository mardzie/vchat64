use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    crossterm::event::Event,
    layout::{Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, Widget},
};

use crate::{state::AppState, vchat::VChat};

pub const KEY_CODE_ACCEPT: KeyCode = KeyCode::Enter;
pub const KEY_CODE_DECLINE: KeyCode = KeyCode::Esc;

pub struct App {
    exit: bool,
    state: AppState,
    vchat: VChat,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: Default::default(),
            state: Default::default(),
            vchat: VChat::new("127.0.0.1:22000").unwrap(),
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }

        Ok(())
    }

    pub fn handle_event(&mut self) -> Result<()> {
        let event = event::read()?;

        match &self.state {
            AppState::App => self.handle_app_event(event)?,
            AppState::Exit => self.handle_exit_event(event)?,
        };

        Ok(())
    }

    fn handle_app_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => self.handle_key_event(key_event)?,
            _ => {}
        };

        Ok(())
    }

    fn handle_exit_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key_event) => {
                if key_event.code == KEY_CODE_ACCEPT {
                    self.exit = true;
                } else if key_event.code == KEY_CODE_DECLINE {
                    self.state = AppState::App;
                };
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.kind {
            KeyEventKind::Press => match key_event.code {
                KeyCode::Char('q') => {
                    self.state = crate::state::AppState::Exit;
                }
                _ => {}
            },
            _ => {}
        };

        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.area());
    }

    fn render_main_area(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = Line::from(" VChat64 ").left_aligned().bold().yellow();
        let instructions = Line::from(vec![" Quit".into(), " <Q> ".bold().yellow()]).left_aligned();
        let block = Block::bordered()
            .border_type(BorderType::Plain)
            .title(title)
            .title_bottom(instructions);
        let block_area = block.inner(area);
        block.render(area, buf);

        let layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(1),
        ]);
        let [my_friend_code_area, line_area, input_friend_code_area] = layout.areas(block_area);

        let text = Line::from("My friend code").centered().red();
        text.render(my_friend_code_area, buf);

        let line_block = Block::bordered()
            .border_type(BorderType::Plain)
            .borders(Borders::BOTTOM);
        line_block.render(line_area, buf);

        let text = Line::from("Input friend code").centered().red();
        text.render(input_friend_code_area, buf);
    }

    fn render_call_area(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = Line::from(" In Call ").bold().yellow();
        let block = Block::bordered()
            .border_type(BorderType::Plain)
            .title(title);
        let block_area = block.inner(area);
        block.render(area, buf);
    }

    fn render_exit_area(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let vertical_layout = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Fill(1),
        ]);
        let [_, vertical_exit_area, _] = vertical_layout.areas(area);

        let middle_layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(32),
            Constraint::Fill(1),
        ]);
        let [_, exit_area, _] = middle_layout.areas(vertical_exit_area);

        let title = Line::from(" Exit? ").left_aligned().bold().red();
        let instructions = Line::from(vec![
            " Back".into(),
            " <ESC> ".bold().yellow(),
            " Confirm".into(),
            " <ENTER> ".bold().yellow(),
        ]);
        let block = Block::bordered()
            .border_type(BorderType::Thick)
            .title(title)
            .title_bottom(instructions);
        let block_area = block.inner(exit_area);
        Clear::default().render(exit_area, buf); // Clear exit_area area so no chars shine throug.
        block.render(exit_area, buf);

        let text = Line::from("Do you want to exit?").centered().red();
        text.render(block_area, buf);
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let horizontal_layout =
            Layout::horizontal([Constraint::Percentage(60), Constraint::Fill(1)]);
        let [main_area, call_area] = horizontal_layout.areas(area);

        self.render_main_area(main_area, buf);
        self.render_call_area(call_area, buf);

        if self.state == AppState::Exit {
            self.render_exit_area(area, buf);
        };
    }
}
