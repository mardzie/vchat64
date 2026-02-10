use color_eyre::Result;

mod app;
mod state;

use app::App;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode};

fn main() -> Result<()> {
    env_logger::init();
    color_eyre::install()?;

    enable_raw_mode()?;

    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);

    disable_raw_mode()?;
    if let Err(e) = ratatui::try_restore() {
        eprintln!(
            "failed to restore terminal. Run `reset` or restart your terminal to recover: {}",
            e
        );
    };

    app_result
}
