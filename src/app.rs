use crate::{config, ui, wttr};
use chrono::{DateTime, Local};
use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::{
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

// --- Application State Management ---
pub struct AppData<'a> {
    pub country: config::Country<'a>,
    pub reports: wttr::WeatherReports<'a>,
    pub summaries: Vec<String>,
}

pub enum AppState<'a> {
    Loading,
    Loaded {
        data: AppData<'a>,
        updated_at: DateTime<Local>,
        last_fetch: Instant,
    },
    Error(String),
}

pub enum ViewState {
    Main,
    Details,
}

fn spawn_fetch_thread(
    tx: mpsc::Sender<Result<AppData<'static>, String>>,
    country: config::Country<'static>,
) {
    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let mut weather_reports = std::collections::HashMap::new();
        let mut summaries = Vec::new();
        for region in country.regions {
            match wttr::get_weather_data(&client, region.city) {
                Ok(report) => {
                    if let Some(condition) = report.current_condition.first() {
                        let desc = condition.weatherDesc.first().map_or("N/A", |d| &d.value);
                        summaries.push(format!("{}: {}", region.name, desc));
                        weather_reports.insert(region.name, report.clone());
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(format!(
                        "Failed to fetch weather for {}: {}",
                        region.city, e
                    )));
                    return;
                }
            }
        }
        let _ = tx.send(Ok(AppData {
            country,
            reports: weather_reports,
            summaries,
        }));
    });
}

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    country: config::Country<'static>,
) -> io::Result<()> {
    let (tx, rx) = mpsc::channel();
    spawn_fetch_thread(tx.clone(), country);

    let mut app_state = AppState::Loading;
    let mut view_state = ViewState::Main;
    let mut counter: u16 = 100;

    loop {
        terminal.draw(|f| match &app_state {
            AppState::Loading => ui::loading_ui(f, counter),
            AppState::Loaded {
                data, updated_at, ..
            } => match view_state {
                ViewState::Main => ui::main_ui(f, data, updated_at),
                ViewState::Details => ui::details_ui(f, data),
            },
            AppState::Error(e) => ui::error_ui(f, e),
        })?;

        // --- Event Handling ---
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match app_state {
                    AppState::Error(_) => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('r') => {
                            app_state = AppState::Loading;
                            spawn_fetch_thread(tx.clone(), country);
                        }
                        _ => {}
                    },
                    _ => match view_state {
                        ViewState::Main => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('d') => view_state = ViewState::Details,
                            KeyCode::Char('r') => {
                                app_state = AppState::Loading;
                                spawn_fetch_thread(tx.clone(), country);
                            }
                            _ => {}
                        },
                        ViewState::Details => match key.code {
                            KeyCode::Char('m') | KeyCode::Esc => view_state = ViewState::Main,
                            _ => {}
                        },
                    },
                }
            }
        }

        // --- State Update ---
        if let Ok(result) = rx.try_recv() {
            match result {
                Ok(data) => {
                    app_state = AppState::Loaded {
                        data,
                        updated_at: Local::now(),
                        last_fetch: Instant::now(),
                    }
                }
                Err(e) => app_state = AppState::Error(e),
            }
        }

        if let AppState::Loaded { last_fetch, .. } = app_state {
            if last_fetch.elapsed() > config::REFRESH_INTERVAL {
                app_state = AppState::Loading;
                spawn_fetch_thread(tx.clone(), country);
            }
        }

        if matches!(app_state, AppState::Loading) {
            counter = 100 + (counter + 1 - 100) % 800;
        }
    }
}

