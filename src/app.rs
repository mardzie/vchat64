use std::{
    net::{SocketAddr, ToSocketAddrs},
    str::FromStr,
    sync::{
        self, Arc,
        atomic::{self, AtomicBool, Ordering},
        mpsc::SyncSender,
    },
    thread::{self, JoinHandle},
};

use color_eyre::eyre::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, BorderType, Borders, Clear, List, Widget},
};

pub mod app_events;
pub mod config;
pub mod widgets;

use crate::{
    TIMEOUT,
    app::{app_events::Event, config::Config, widgets::line_text_area::LineTextArea},
    state::AppState,
    vchat::VChat,
};

pub const KEY_CODE_ACCEPT: KeyCode = KeyCode::Enter;
pub const KEY_CODE_DECLINE: KeyCode = KeyCode::Esc;

pub struct App {
    exit: Arc<AtomicBool>,
    error_msg: (String, chrono::DateTime<chrono::Utc>),
    config: Config,
    event_channel_tx: sync::mpsc::SyncSender<Event>,
    event_channel_rx: sync::mpsc::Receiver<Event>,
    state: AppState,
    addr_input: LineTextArea,
    vchat: VChat,

    event_handle: JoinHandle<()>,

    runtime: tokio::runtime::Runtime,
}

impl App {
    pub fn new() -> Self {
        let exit = Arc::new(atomic::AtomicBool::new(false));
        let (tx, rx) = std::sync::mpsc::sync_channel(2);

        let tx_c = tx.clone();
        let exit_c = exit.clone();
        let handle = thread::spawn(move || Self::crossterm_event_reader(tx_c, exit_c));

        let args: Vec<String> = std::env::args().collect();
        let config = Config::new(
            args.get(1)
                .map(|x| {
                    x.parse::<u16>()
                        .expect("Failed to convert first arg into port number!")
                })
                .unwrap_or(22000),
        );

        let runtime = tokio::runtime::Builder::new_current_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime!");

        Self {
            exit: exit.clone(),
            error_msg: (String::new(), chrono::DateTime::<chrono::Utc>::MAX_UTC),
            config,
            event_channel_tx: tx,
            event_channel_rx: rx,
            state: Default::default(),
            addr_input: LineTextArea::new("".to_string(), 0),
            vchat: VChat::new("0.0.0.0:22000", exit).unwrap(),

            event_handle: handle,

            runtime,
        }
    }

    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) -> Result<()> {
        log::info!("VChat64 running...");

        self.vchat.audio().play();

        while !self.exit.load(Ordering::Acquire) {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }

        self.vchat.audio().pause();

        Ok(())
    }

    fn crossterm_event_reader(event_reader: SyncSender<Event>, exit: Arc<AtomicBool>) {
        loop {
            if exit.load(Ordering::Acquire) {
                break;
            };

            match event::poll(TIMEOUT) {
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

            if let Err(_) = event_reader.send(event) {
                log::error!("Crossterm Event Reader: Reading channel closed.");
                break;
            };
        }

        log::debug!("Crossterm Event Reader: Shutting down...");
    }

    pub fn stop(self) {
        self.vchat.stop();
        let _ = self.event_handle.join();
    }

    fn set_error(&mut self, s: String) {
        self.error_msg = (s, chrono::Utc::now());
    }

    fn to_app_state(&mut self) {
        self.state = AppState::App;
        if let Err(_) = self.event_channel_tx.try_send(Event::ReDraw) {
            log::warn!("Failed to issue redraw: Content may be outdated: Press any key to update.");
        };
    }

    fn get_local_friend_code(&self) -> Result<String, String> {
        let ip = match local_ip_address::local_ip() {
            Ok(ip) => ip,
            Err(e) => {
                log::warn!("Failed to get local ip: {}", e);
                return Err("Failed to fetch local friend code!".to_string());
            }
        };

        Ok(self.get_friend_code_from_ip(ip))
    }

    fn get_public_friend_code(&self) -> Result<String, String> {
        let ip = match self
            .runtime
            .block_on(public_ip_address::perform_lookup(None))
        {
            Ok(lookup) => lookup.ip,
            Err(_) => {
                log::warn!("Failed to perform public ip lookup.");
                return Err("Failed to fetch public friend code!".to_string());
            }
        };

        Ok(self.get_friend_code_from_ip(ip))
    }

    fn get_friend_code_from_ip(&self, ip: std::net::IpAddr) -> String {
        let addr = SocketAddr::new(ip, self.config.port());
        Self::ip_to_friend_code(addr)
    }

    fn ip_to_friend_code(addr: SocketAddr) -> String {
        let mut addr_bytes: Vec<u8> = Vec::with_capacity(18); // 18 bytes to make space for ipv6 + port or ipv4 + port.
        match addr.ip() {
            std::net::IpAddr::V4(ipv4) => {
                addr_bytes.extend_from_slice(&ipv4.octets());
            }
            std::net::IpAddr::V6(ipv6) => {
                addr_bytes.extend_from_slice(&ipv6.octets());
            }
        };

        addr_bytes.extend_from_slice(&addr.port().to_be_bytes());

        let hex = hex::encode(addr_bytes);
        hex.chars()
            .collect::<Vec<char>>()
            .chunks(2)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ")
    }

    fn friend_code_to_ip(&mut self, mut friend_code: String) -> Result<SocketAddr, ()> {
        friend_code = friend_code.trim().to_string();
        friend_code = friend_code.replace(" ", "");

        let bytes = match hex::decode(friend_code) {
            Ok(bytes) => bytes,
            Err(e) => {
                log::warn!("Failed to decode friend code: {}", e);
                self.set_error(format!("Failed to decode friend code: {}!", e));
                return Err(());
            }
        };

        let mut port_bytes = [0u8; 2];
        let ip = match bytes.len() {
            6 => {
                let mut ip_bytes = [0u8; 4];
                ip_bytes.copy_from_slice(&bytes[..4]);
                port_bytes.copy_from_slice(&bytes[4..6]);

                std::net::IpAddr::from(std::net::Ipv4Addr::from_octets(ip_bytes))
            }
            18 => {
                let mut ip_bytes = [0u8; 16];
                ip_bytes.copy_from_slice(&bytes[..16]);
                port_bytes.copy_from_slice(&bytes[16..18]);

                std::net::IpAddr::V6(std::net::Ipv6Addr::from_octets(ip_bytes))
            }
            bytes => {
                log::warn!("Failed to decode friend code: Found {} bytes", bytes);
                self.set_error(format!(
                    "Failed to decode friend code: Invalid lenght {} bytes!",
                    bytes
                ));
                return Err(());
            }
        };

        Ok(std::net::SocketAddr::new(
            ip,
            u16::from_be_bytes(port_bytes),
        ))
    }

    pub fn handle_event(&mut self) -> Result<()> {
        let event = self.event_channel_rx.recv()?;

        let event = match event {
            Event::Crossterm(event) => event,
            Event::ReDraw => {
                return Ok(());
            }
        };

        match &self.state {
            AppState::App => self.handle_app_event(&event)?,
            AppState::CodeInput => {
                if self.addr_input.selected() {
                    if let event::Event::Key(key_event) = &event
                        && key_event.code == KeyCode::Enter
                    {
                        let buf = self.addr_input.get_buf().to_string();
                        if let Ok(addr) = self.friend_code_to_ip(buf) {
                            self.vchat.add_address(addr);
                        };

                        self.to_app_state();
                    } else {
                        self.addr_input.handle_event(&event)?;
                    };
                } else {
                    self.to_app_state();
                };
            }
            AppState::Exit => self.handle_exit_event(&event)?,
        };

        let max_timestamp = chrono::DateTime::<chrono::Utc>::MAX_UTC;
        let (err, timestamp) = &self.error_msg;
        if timestamp != &max_timestamp
            && *timestamp
                + chrono::Duration::seconds((2 * err.split_ascii_whitespace().count()) as i64)
                < chrono::Utc::now()
        {
            self.error_msg = (String::new(), max_timestamp);
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
                    self.state = AppState::Exit;
                    log::debug!("Into `Exit` state.");
                }
                KeyCode::Char('i') => {
                    self.state = AppState::CodeInput;
                    self.addr_input.select();
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
                    self.exit.store(true, Ordering::Release);
                    log::info!("Exiting...");
                } else if key_event.code == KEY_CODE_DECLINE {
                    self.state = AppState::App;
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

        let public_friend_code =
            Line::from(self.get_public_friend_code().map_or_else(|x| x, |y| y))
                .bold()
                .red()
                .centered();
        public_friend_code.render(public_code_area, buf);

        let title = Line::from(" Local Friend Code ").bold().yellow();
        let block = Block::new().borders(Borders::TOP).title(title);
        block.render(local_header_area, buf);

        let local_friend_code = Line::from(self.get_local_friend_code().map_or_else(|x| x, |y| y))
            .bold()
            .red()
            .centered();
        local_friend_code.render(local_code_area, buf);
    }

    fn render_text_area(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let title = Line::from(" Input Friend Code ").bold().red();
        let mut text_area_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(title)
            .title_alignment(Alignment::Left);

        if let AppState::CodeInput = self.state {
            let instructions = Line::from(vec![" Exit Input".into(), " <ESC> ".bold().yellow()]);
            text_area_block = text_area_block.title_bottom(instructions);
        };

        let text_area_block_area = text_area_block.inner(area);
        text_area_block.render(area, buf);

        self.addr_input.render(text_area_block_area, buf);
    }

    fn render_error_area(&self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let layout = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [block_area, error_area] = layout.areas(area);

        let block = Block::new()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Plain);
        block.render(block_area, buf);

        let line = Line::from(self.error_msg.0.clone()).on_red().white();
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

        match self.state {
            AppState::Exit => self.render_exit_area(area, buf),
            _ => {}
        };
    }
}
