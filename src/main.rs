#![warn(clippy::all)]

use color_eyre::Result;

mod app;

use crate::app::App;

pub const CHILL_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(250);
pub const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(10);

fn main() -> Result<()> {
    // Add custom writer to limit log file size.
    let log_file = std::io::LineWriter::new(std::fs::File::create("./log.log").unwrap());
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(std::sync::Mutex::new(log_file))
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_level(true)
        .with_target(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_thread_ids(true)
        .pretty()
        .init();

    color_eyre::install()?;

    set_custom_crossterm_panic_hook();

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

/// Adds a terminal restore to the normal panic hook.
fn set_custom_crossterm_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        if let Err(e) = crossterm::terminal::disable_raw_mode() {
            eprintln!(
                "Failed to disable raw mode for terminal! A terminal restart is required: {}",
                e
            );
        }
        if let Err(e) =
            crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)
        {
            eprintln!("Failed to leave alternate screen: {}", e);
        }
        original_hook(panic_info);
    }));
}
