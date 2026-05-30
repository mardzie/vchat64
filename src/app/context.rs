use std::{
    net::SocketAddr,
    sync::{self, Arc, atomic::AtomicBool, mpsc::SyncSender},
};

use friend_code::FriendCode;
use tokio::runtime;
use vchat::VChat;

use crate::app::{
    app_config::AppConfig,
    app_events::Event,
    helpers::{load_atomic_bool, store_atomic_bool},
    state::AppState,
    widgets::line_text_area::LineTextArea,
};

#[derive(Debug)]
pub struct AppContext {
    pub exit: Arc<AtomicBool>,
    pub state: AppState,
    pub config: AppConfig,

    pub error_msg: Option<(String, chrono::DateTime<chrono::Utc>)>,
    pub event_tx: sync::mpsc::SyncSender<Event>,

    pub vchat: VChat,
    pub addr_input: LineTextArea,

    pub public_friend_code: FriendCode,
    pub local_friend_code: FriendCode,

    runtime: tokio::runtime::Runtime,
}

impl AppContext {
    pub fn new(state: AppState, config: AppConfig, event_tx: SyncSender<Event>) -> Self {
        let runtime = runtime::Builder::new_current_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to create a new tokio runtime!");

        let exit = Arc::new(AtomicBool::new(false));

        let vchat = VChat::new(SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            config.port(),
        ));

        let public_friend_code = FriendCode::new_public(&runtime, config.port())
            .expect("Failed to get public friend code");
        let local_friend_code =
            FriendCode::new_local(config.port()).expect("Failed to get local friend code");

        Self {
            exit,
            state,
            config,

            error_msg: None,
            event_tx,

            vchat,
            addr_input: LineTextArea::new(String::new(), 0),

            public_friend_code,
            local_friend_code,

            runtime,
        }
    }

    pub fn get_error(&self) -> &Option<(String, chrono::DateTime<chrono::Utc>)> {
        &self.error_msg
    }

    pub fn set_error(&mut self, s: Option<String>) {
        if let Some(s) = s {
            self.error_msg = Some((s, chrono::Utc::now()));
        } else {
            self.error_msg = None;
        }
    }

    pub fn get_state(&self) -> &AppState {
        &self.state
    }

    pub fn to_state(&mut self, new_state: AppState) {
        self.state = new_state;
        if self.event_tx.try_send(Event::ReDraw).is_err() {
            tracing::warn!(
                "Failed to issue redraw: Content may be outdated: Press any key to update."
            );
        };
    }

    pub fn runtime(&self) -> &runtime::Runtime {
        &self.runtime
    }

    pub fn get_exit(&self) -> bool {
        load_atomic_bool(&self.exit)
    }

    pub fn set_exit(&self, val: bool) {
        store_atomic_bool(&self.exit, val);
    }
}
