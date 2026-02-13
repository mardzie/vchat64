use color_eyre::Result;

mod app;
mod audio;
mod hash;
mod helpers;
mod state;
mod traits;
mod udp_net;
mod vchat;

use crate::app::App;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode};

pub const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(250);

fn main() -> Result<()> {
    let file = std::fs::File::create("./log.log").unwrap();
    env_logger::builder()
        .default_format()
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();

    color_eyre::install()?;

    enable_raw_mode()?;

    let mut terminal = ratatui::init();
    let mut app = App::new();
    let app_result = app.run(&mut terminal);
    app.stop();

    disable_raw_mode()?;
    if let Err(e) = ratatui::try_restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            e
        );
    };

    app_result
}
