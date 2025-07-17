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
    pub summaries: Vec<(String, &'static str)>,
    pub footer_text: (String, &'static str),
    pub left_text: (String, &'static str),
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

// ViewState now includes scroll position for list-based views.
pub enum ViewState {
    Main,
    Details { scroll: u16 },
    Hourly { region_index: usize, scroll: u16 },
    SelectCountry { available: Vec<String>, scroll: u16 },
}

fn spawn_fetch_thread(
    tx: mpsc::Sender<Result<AppData, String>>,
    country: Arc<config::Country>,
    client: Arc<dyn wttr::WeatherClient>,
) {
    thread::spawn(move || {
        let mut weather_reports = std::collections::HashMap::new();
        let mut summaries = Vec::new();
        for region in country.regions.iter() {
            match client.fetch(&region.city) {
                Ok(report) => {
                    if let Some(condition) = report.current_condition.first() {
                        let desc = condition.weatherDesc.first().map_or("N/A", |d| &d.value);
                        let icon = wttr::get_weather_icon(desc);
                        summaries.push((format!("{}: {}", region.name, desc), icon));
                        weather_reports.insert(region.name.clone(), report.clone());
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            }
        }

        let footer_desc = country.regions.first()
            .and_then(|region| weather_reports.get(&region.name))
            .and_then(|report| report.current_condition.first())
            .and_then(|condition| condition.weatherDesc.first())
            .map_or_else(|| "Weather summary unavailable.".to_string(), |desc| desc.value.clone());
        let footer_icon = wttr::get_weather_icon(&footer_desc);
        let footer_text = (footer_desc, footer_icon);

        let left_desc = country.regions.get(1)
            .or_else(|| country.regions.first())
            .and_then(|region| weather_reports.get(&region.name))
            .and_then(|report| report.current_condition.first())
            .and_then(|condition| condition.weatherDesc.first())
            .map_or_else(|| "No specific forecast.".to_string(), |desc| desc.value.clone());
        let left_icon = wttr::get_weather_icon(&left_desc);
        let left_text = (left_desc, left_icon);

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
    client: Arc<dyn wttr::WeatherClient>,
) -> io::Result<Option<String>> {
    let country_arc = Arc::new(country);
    let (tx, rx) = mpsc::channel();
    spawn_fetch_thread(tx.clone(), country_arc.clone(), client.clone());

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
                ViewState::Details { scroll } => ui::details_ui(f, data, *scroll),
                ViewState::Hourly { region_index, scroll } => ui::hourly_ui(f, data, *region_index, *scroll),
                ViewState::SelectCountry { available, scroll } => ui::select_country_ui(f, available, *scroll),
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
                            spawn_fetch_thread(tx.clone(), country_arc.clone(), client.clone());
                        }
                        _ => {}
                    },
                    AppState::Loaded { data, .. } => match &mut view_state {
                        ViewState::Main => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                            KeyCode::Char('d') => view_state = ViewState::Details { scroll: 0 },
                            KeyCode::Char('c') => {
                                if let Ok(available) = config::get_available_countries() {
                                    view_state = ViewState::SelectCountry { available, scroll: 0 };
                                }
                            }
                            KeyCode::Char('r') => {
                                app_state = AppState::Loading;
                                spawn_fetch_thread(tx.clone(), country_arc.clone(), client.clone());
                            }
                            _ => {}
                        },
                        ViewState::Details { scroll } => match key.code {
                            KeyCode::Char('m') | KeyCode::Esc => view_state = ViewState::Main,
                            KeyCode::Up => *scroll = scroll.saturating_sub(1),
                            KeyCode::Down => *scroll = scroll.saturating_add(1),
                            KeyCode::Char(c) => {
                                if let Some(digit) = c.to_digit(10) {
                                    let index = digit as usize;
                                    if index > 0 && index <= data.country.regions.len() {
                                        view_state = ViewState::Hourly { region_index: index - 1, scroll: 0 };
                                    }
                                }
                            }
                            _ => {}
                        },
                        ViewState::Hourly { scroll, .. } => match key.code {
                            KeyCode::Char('d') | KeyCode::Esc => view_state = ViewState::Details { scroll: 0 },
                            KeyCode::Up => *scroll = scroll.saturating_sub(1),
                            KeyCode::Down => *scroll = scroll.saturating_add(1),
                            _ => {}
                        },
                        ViewState::SelectCountry { available, scroll } => match key.code {
                            KeyCode::Char('m') | KeyCode::Esc => view_state = ViewState::Main,
                            KeyCode::Up => *scroll = scroll.saturating_sub(1),
                            KeyCode::Down => *scroll = scroll.saturating_add(1),
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
                spawn_fetch_thread(tx.clone(), country_arc.clone(), client.clone());
            }
        }

        if matches!(app_state, AppState::Loading) {
            counter = 100 + (counter + 1 - 100) % 800;
        }
    }
}

