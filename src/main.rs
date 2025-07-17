mod app;
mod config;
mod ui;
mod wttr;

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = config::Cli::parse();
    let country_config = match cli.country.to_lowercase().as_str() {
        "germany" | "de" => config::GERMANY,
        _ => config::UK,
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Delegate all logic to the app module
    app::run_app(&mut terminal, country_config)?;

    // Restore the terminal state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

