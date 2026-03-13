#![warn(clippy::all)]

use color_eyre::Result;

mod app;
mod audio;
mod hash;
mod helpers;
mod state;
mod traits;
mod types;
mod udp_packet_net;
mod vchat;
mod voice_net;

use crate::app::App;

pub const CHILL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(250);
pub const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(10);

fn main() -> Result<()> {
    let file = std::fs::File::create("./log.log").unwrap();
    env_logger::builder()
        .default_format()
        .filter_level(log::LevelFilter::Trace)
        .target(env_logger::Target::Pipe(Box::new(file)))
        .init();

    color_eyre::install()?;

    let mut terminal = ratatui::init();
    let mut app = App::new();
    let app_result = app.run(&mut terminal);
    app.stop();

    if let Err(e) = ratatui::try_restore() {
        eprintln!(
            "Failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            e
        );
    };

    app_result
}
