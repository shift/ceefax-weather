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
use std::{io, sync::Arc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = config::Cli::parse();
    let mut current_country_name = cli.country;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Create the single, shareable client for the application's lifetime.
    let client = Arc::new(wttr::LiveWeatherClient::new());

    loop {
        let country_config = config::load_country_config(&current_country_name).unwrap_or_else(|e| {
            eprintln!(
                "Error loading configuration for '{}': {}",
                current_country_name, e
            );
            std::process::exit(1);
        });

        // Inject the client into the application loop.
        match app::run_app(&mut terminal, country_config, client.clone())? {
            Some(new_country) => {
                current_country_name = new_country;
            }
            None => break,
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

