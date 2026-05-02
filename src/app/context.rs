use std::sync::{self, Arc, atomic::AtomicBool};

use friend_code::FriendCode;
use vchat::VChat;

use crate::app::{app_events::Event, config::Config, state::AppState};

#[derive(Debug)]
pub struct AppContext {
    exit: Arc<AtomicBool>,
    state: AppState,
    config: Config,
    error_msg: Option<(String, chrono::DateTime<chrono::Utc>)>,
    event_channel_tx: sync::mpsc::SyncSender<Event>,
    vchat: VChat,

    public_friend_code: FriendCode,
    local_friend_code: FriendCode,

    runtime: tokio::runtime::Runtime,
}
