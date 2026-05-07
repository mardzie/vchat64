use std::{
    sync::{self, Arc, atomic::AtomicBool, mpsc::SyncSender},
    thread::{self, JoinHandle},
};

use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use friend_code::FriendCode;
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, List, Widget},
};

pub mod app_config;

mod app_events;
mod context;
mod helpers;
mod state;
mod widgets;

use crate::{
    CHILL_TIMEOUT,
    app::{
        app_config::AppConfig, app_events::Event, context::AppContext, helpers::load_atomic_bool,
        state::AppState,
    },
};

pub const KEY_CODE_ACCEPT: KeyCode = KeyCode::Enter;
pub const KEY_CODE_DECLINE: KeyCode = KeyCode::Esc;

pub const EVENT_QUEUE_SIZE: usize = 8;

#[derive(Debug)]
pub struct App {
    ctx: AppContext,

    event_channel_rx: sync::mpsc::Receiver<Event>,
    event_handle: JoinHandle<()>,
}

impl App {
    pub fn new() -> Self {
        let args: Vec<String> = std::env::args().collect();
        let config = AppConfig::new(
            args.get(1)
                .map(|x| {
                    x.parse::<u16>()
                        .expect("Failed to convert first arg into port number!")
                })
                .unwrap_or(22000),
        );

        let (event_tx, event_rx) = std::sync::mpsc::sync_channel(EVENT_QUEUE_SIZE);
        let ctx = AppContext::new(AppState::new_app(), config, event_tx.clone());

        let exit_c = ctx.exit.clone();
        let handle = thread::spawn(move || Self::crossterm_event_reader(event_tx, exit_c));

        Self {
            ctx,

            event_channel_rx: event_rx,
            event_handle: handle,
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        log::info!("VChat64 running...");

        self.ctx.vchat.audio().play();

        while !self.ctx.get_exit() {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }

        self.ctx.vchat.audio().pause();

        Ok(())
    }

    /// Handles all incoming crossterm events and forwards them to the apps event handler.
    fn crossterm_event_reader(event_tx: SyncSender<Event>, exit: Arc<AtomicBool>) {
        loop {
            if load_atomic_bool(&exit) {
                break;
            };

            match event::poll(CHILL_TIMEOUT) {
                Ok(x) if x => {}
                Ok(_) => continue,
                Err(e) => {
                    log::error!(
                        "Crossterm Event Reader: Caught Error while polling for crossterm event: {}",
                        e
                    );
                    break;
                }
            }

            let event = match event::read() {
                Ok(event) => event.into(),
                Err(e) => {
                    log::error!("Failed to read terminal event: {}", e);
                    continue;
                }
            };

            if event_tx.send(event).is_err() {
                log::error!("Crossterm Event Reader: Reading channel closed.");
                break;
            };
        }

        log::debug!("Crossterm Event Reader: Shutting down...");
    }

    pub fn stop(self) {
        self.ctx.vchat.stop();
        let _ = self.event_handle.join();
    }

    pub fn handle_event(&mut self) -> Result<()> {
        let event = self.event_channel_rx.recv()?;

        let event = match event {
            Event::Crossterm(event) => event,
            Event::ReDraw => {
                return Ok(());
            }
        };

        match &self.ctx.get_state() {
            AppState::App(_) => self.handle_app_event(&event)?,
            AppState::CodeInput(_) => {
                if self.ctx.addr_input.selected() {
                    if let event::Event::Key(key_event) = &event
                        && key_event.code == KeyCode::Enter
                    {
                        let buf = self.ctx.addr_input.get_buf().to_string();
                        if let Ok(fc) = FriendCode::from_string_friend_code(&buf) {
                            self.ctx.vchat.add_address(fc.into_socket_addr());
                            self.ctx.addr_input.clear();
                        }

                        self.ctx.to_state(AppState::new_app());
                    } else {
                        self.ctx.addr_input.handle_event(&event)?;
                    };
                } else {
                    self.ctx.to_state(AppState::new_app());
                };
            }
            AppState::Exit(_) => self.handle_exit_event(&event)?,
        };

        if let Some((err, timestamp)) = &self.ctx.get_error()
            && *timestamp
                + chrono::Duration::seconds((2 * err.split_ascii_whitespace().count()) as i64)
                < chrono::Utc::now()
        {
            self.ctx.set_error(None);
        };

        Ok(())
    }

    fn handle_app_event(&mut self, event: &event::Event) -> Result<()> {
        match event {
            event::Event::Key(key_event) => self.handle_key_event(key_event)?,
            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: &KeyEvent) -> Result<()> {
        match key_event.kind {
            KeyEventKind::Press => match key_event.code {
                KeyCode::Char('q') => {
                    self.ctx.to_state(AppState::new_exit());
                    log::debug!("Into `Exit` state.");
                }
                KeyCode::Char('i') => {
                    self.ctx.to_state(AppState::new_code_input());
                    self.ctx.addr_input.select();
                    log::debug!("Into `CodeInput` state.");
                }
                _ => {}
            },
            _ => {}
        };

        Ok(())
    }

    fn handle_exit_event(&mut self, event: &event::Event) -> Result<()> {
        match event {
            event::Event::Key(key_event) => {
                if key_event.code == KEY_CODE_ACCEPT {
                    self.ctx.set_exit(true);
                    log::info!("Exiting...");
                } else if key_event.code == KEY_CODE_DECLINE {
                    self.ctx.to_state(AppState::new_app());
                    log::info!("Canceled exiting.");
                };
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.area());
    }

    fn render_main_area(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = Line::from(" VChat64 ").left_aligned().bold().yellow();
        let instructions = Line::from(vec![
            " Quit".into(),
            " <Q> ".bold().yellow(),
            " Input FC".into(),
            " <I> ".bold().yellow(),
        ])
        .left_aligned();
        let block = Block::bordered()
            .border_type(BorderType::Plain)
            .title(title)
            .title_bottom(instructions);
        let block_area = block.inner(area);
        block.render(area, buf);

        let layout = Layout::vertical([
            Constraint::Length(5),
            Constraint::Length(1),
            Constraint::Min(1),
        ]);
        let [my_friend_code_area, line_area, actions_area] = layout.areas(block_area);

        self.render_friend_codes(my_friend_code_area, buf);

        let line_block = Block::bordered()
            .border_type(BorderType::Plain)
            .borders(Borders::BOTTOM);
        line_block.render(line_area, buf);

        let action_area_layout = Layout::vertical([
            Constraint::Length(3),
            Constraint::Fill(1),
            Constraint::Length(2),
        ]);
        let [input_friend_code_area, _, error_area] = action_area_layout.areas(actions_area);

        self.render_text_area(input_friend_code_area, buf);

        self.render_error_area(error_area, buf);
    }

    fn render_friend_codes(
        &self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]);
        let [
            public_header_area,
            public_code_area,
            _,
            local_header_area,
            local_code_area,
        ] = layout.areas(area);

        let title = Line::from(" Public Friend Code ").bold().yellow();
        let block = Block::new().borders(Borders::TOP).title(title);
        block.render(public_header_area, buf);

        let public_friend_code_line = Line::from(self.ctx.public_friend_code.to_pretty_string())
            .bold()
            .red()
            .centered();
        public_friend_code_line.render(public_code_area, buf);

        let title = Line::from(" Local Friend Code ").bold().yellow();
        let block = Block::new().borders(Borders::TOP).title(title);
        block.render(local_header_area, buf);
        let local_friend_code_line = Line::from(self.ctx.local_friend_code.to_pretty_string())
            .bold()
            .red()
            .centered();
        local_friend_code_line.render(local_code_area, buf);
    }

    fn render_text_area(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = Line::from(" Input Friend Code ").bold().red();
        let mut text_area_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title)
            .title_alignment(Alignment::Left);

        if matches!(self.ctx.get_state(), AppState::CodeInput(_)) {
            let instructions = Line::from(vec![" Exit Input".into(), " <ESC> ".bold().yellow()]);
            text_area_block = text_area_block.title_bottom(instructions);
        };

        let text_area_block_area = text_area_block.inner(area);
        text_area_block.render(area, buf);

        self.ctx.addr_input.render(text_area_block_area, buf);
    }

    fn render_error_area(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [block_area, error_area] = layout.areas(area);

        let block = Block::new()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Plain);
        block.render(block_area, buf);

        let line = if let Some((e, _)) = &self.ctx.get_error() {
            Line::from(e.as_ref()).on_red().white()
        } else {
            Line::from("")
        };
        line.render(error_area, buf);
    }

    fn render_call_area(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = Line::from(" In Call ").bold().yellow();
        let block = Block::bordered()
            .border_type(BorderType::Plain)
            .title(title);
        let block_area = block.inner(area);
        block.render(area, buf);

        let addresses: Vec<String> = self
            .ctx
            .vchat
            .get_addresses()
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .to_vec()
            .into_iter()
            .map(|addr| addr.to_string())
            .collect();

        let list = List::new(addresses);
        list.render(block_area, buf);
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
        Clear.render(exit_area, buf); // Clear exit_area area so no chars shine throug.
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

        match self.ctx.get_state() {
            AppState::Exit(_) => self.render_exit_area(area, buf),
            _ => {}
        };
    }
}
