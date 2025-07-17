use crate::{config, ui, wttr};
use chrono::{DateTime, Local};
use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::{
    io,
    sync::{mpsc, Arc},
    thread,
    time::{Duration, Instant},
};

// --- Application State Management ---
pub struct AppData {
    pub country: Arc<config::Country>,
    pub reports: wttr::WeatherReports,
    pub summaries: Vec<String>,
    pub footer_text: String,
    pub left_text: String,
}

pub enum AppState {
    Loading,
    Loaded {
        data: AppData,
        updated_at: DateTime<Local>,
        last_fetch: Instant,
    },
    Error(String),
}

pub enum ViewState {
    Main,
    Details,
    Hourly { region_index: usize },
    SelectCountry { available: Vec<String> },
}

fn spawn_fetch_thread(
    tx: mpsc::Sender<Result<AppData, String>>,
    country: Arc<config::Country>,
) {
    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let mut weather_reports = std::collections::HashMap::new();
        let mut summaries = Vec::new();
        for region in country.regions.iter() {
            match wttr::get_weather_data(&client, &region.city) {
                Ok(report) => {
                    if let Some(condition) = report.current_condition.first() {
                        let desc = condition.weatherDesc.first().map_or("N/A", |d| &d.value);
                        summaries.push(format!("{}: {}", region.name, desc));
                        weather_reports.insert(region.name.clone(), report.clone());
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            }
        }

        let footer_text = country.regions.first()
            .and_then(|region| weather_reports.get(&region.name))
            .and_then(|report| report.current_condition.first())
            .and_then(|condition| condition.weatherDesc.first())
            .map_or_else(|| "Weather summary unavailable.".to_string(), |desc| desc.value.clone());

        let left_text = country.regions.get(1)
            .or_else(|| country.regions.first())
            .and_then(|region| weather_reports.get(&region.name))
            .and_then(|report| report.current_condition.first())
            .and_then(|condition| condition.weatherDesc.first())
            .map_or_else(|| "No specific forecast.".to_string(), |desc| desc.value.clone());

        let _ = tx.send(Ok(AppData {
            country,
            reports: weather_reports,
            summaries,
            footer_text,
            left_text,
        }));
    });
}

pub fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    country: config::Country,
) -> io::Result<Option<String>> {
    let country_arc = Arc::new(country);
    let (tx, rx) = mpsc::channel();
    spawn_fetch_thread(tx.clone(), country_arc.clone());

    let mut app_state = AppState::Loading;
    let mut view_state = ViewState::Main;
    let mut counter: u16 = 100;

    loop {
        terminal.draw(|f| match &app_state {
            AppState::Loading => ui::loading_ui(f, counter),
            AppState::Loaded {
                data, updated_at, ..
            } => match &view_state {
                ViewState::Main => ui::main_ui(f, data, updated_at),
                ViewState::Details => ui::details_ui(f, data),
                ViewState::Hourly { region_index } => ui::hourly_ui(f, data, *region_index),
                ViewState::SelectCountry { available } => ui::select_country_ui(f, available),
            },
            AppState::Error(e) => ui::error_ui(f, e),
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match &mut app_state {
                    AppState::Error(_) => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                        KeyCode::Char('r') => {
                            app_state = AppState::Loading;
                            spawn_fetch_thread(tx.clone(), country_arc.clone());
                        }
                        _ => {}
                    },
                    AppState::Loaded { data, .. } => match &mut view_state {
                        ViewState::Main => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                            KeyCode::Char('d') => view_state = ViewState::Details,
                            KeyCode::Char('c') => {
                                if let Ok(available) = config::get_available_countries() {
                                    view_state = ViewState::SelectCountry { available };
                                }
                            }
                            KeyCode::Char('r') => {
                                app_state = AppState::Loading;
                                spawn_fetch_thread(tx.clone(), country_arc.clone());
                            }
                            _ => {}
                        },
                        ViewState::Details => match key.code {
                            KeyCode::Char('m') | KeyCode::Esc => view_state = ViewState::Main,
                            KeyCode::Char(c) => {
                                if let Some(digit) = c.to_digit(10) {
                                    let index = digit as usize;
                                    if index > 0 && index <= data.country.regions.len() {
                                        view_state = ViewState::Hourly { region_index: index - 1 };
                                    }
                                }
                            }
                            _ => {}
                        },
                        ViewState::Hourly { .. } => match key.code {
                            KeyCode::Char('d') | KeyCode::Esc => view_state = ViewState::Details,
                            _ => {}
                        },
                        ViewState::SelectCountry { available } => match key.code {
                            KeyCode::Char('m') | KeyCode::Esc => view_state = ViewState::Main,
                            KeyCode::Char(c) => {
                                if let Some(digit) = c.to_digit(10) {
                                    let index = digit as usize;
                                    if index > 0 && index <= available.len() {
                                        return Ok(Some(available[index - 1].clone()));
                                    }
                                }
                            }
                            _ => {}
                        },
                    },
                    AppState::Loading => {
                        if let KeyCode::Char('q') | KeyCode::Esc = key.code {
                            return Ok(None);
                        }
                    }
                }
            }
        }

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

        if let AppState::Loaded { ref mut last_fetch, .. } = app_state {
            if last_fetch.elapsed() > config::REFRESH_INTERVAL {
                app_state = AppState::Loading;
                spawn_fetch_thread(tx.clone(), country_arc.clone());
            }
        }

        if matches!(app_state, AppState::Loading) {
            counter = 100 + (counter + 1 - 100) % 800;
        }
    }
}

