use std::{
    sync::{self, Arc, atomic::AtomicBool, mpsc::SyncSender},
    thread::{self, JoinHandle},
};

use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Widget,
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
        app_config::AppConfig,
        app_events::Event,
        context::AppContext,
        helpers::load_atomic_bool,
        state::{AppState, State},
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
        let ctx = AppContext::new(AppState::app(), config, event_tx.clone());

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

        match *self.ctx.get_state() {
            AppState::App(a) => a.handle_event(&mut self.ctx, &event)?,
            AppState::CodeInput(c) => c.handle_event(&mut self.ctx, &event)?,
            AppState::Exit(e) => e.handle_event(&mut self.ctx, &event)?,
        };

        self.clear_expired_error();

        Ok(())
    }

    fn clear_expired_error(&mut self) {
        if let Some((err, timestamp)) = &self.ctx.get_error()
            && *timestamp
                + chrono::Duration::seconds((2 * err.split_ascii_whitespace().count()) as i64)
                < chrono::Utc::now()
        {
            self.ctx.set_error(None);
        };
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        state::app::App.render(&self.ctx, area, buf);

        match self.ctx.get_state() {
            AppState::Exit(e) => e.render(&self.ctx, area, buf),
            _ => {}
        };
    }
}
