use chrono::Local;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::Deserialize;
use std::{collections::HashMap, io, sync::mpsc, thread, time::Duration};

// --- CEEFAX Color Palette ---
const CEEFAX_BLUE: Color = Color::Rgb(0, 0, 170);
const CEEFAX_GREEN: Color = Color::Rgb(0, 204, 0);
const CEEFAX_CYAN: Color = Color::Rgb(0, 204, 204);
const CEEFAX_YELLOW: Color = Color::Rgb(204, 204, 0);
const CEEFAX_WHITE: Color = Color::Rgb(255, 255, 255);
const CEEFAX_BLACK: Color = Color::Rgb(0, 0, 0);

// --- Unicode Teletext Mosaic Characters ---
const TELETEXT_CHARS: [char; 16] = [
    ' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█',
];

// --- Command Line Argument Parsing ---
#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "COUNTRY", default_value = "uk")]
    country: String,
}

// --- Data Structures for wttr.in JSON Response ---
#[derive(Deserialize, Debug, Clone)]
struct WeatherDesc { value: String }

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)] // To match the API's naming convention
struct CurrentCondition {
    temp_C: String,
    FeelsLikeC: String,
    windspeedKmph: String,
    winddir16Point: String,
    precipMM: String,
    weatherDesc: Vec<WeatherDesc>,
}

#[derive(Deserialize, Debug, Clone)]
struct WeatherReport {
    current_condition: Vec<CurrentCondition>,
}

// --- Map Configuration Structures ---
#[derive(Clone, Copy)]
struct Region<'a> {
    name: &'a str,
    city: &'a str,
    char: char,
    temp_pos: (u16, u16),
}

#[derive(Clone, Copy)]
struct Country<'a> {
    map_template: &'a [&'a str],
    regions: &'a [Region<'a>],
    left_text: &'a [&'a str],
    footer_text: &'a str,
}

// --- ASCII Art ---
const WEATHER_TITLE: &str = "
██╗    ██╗███████╗ █████╗ ████████╗██╗  ██╗███████╗██████╗ 
██║    ██║██╔════╝██╔══██╗╚══██╔══╝██║  ██║██╔════╝██╔══██╗
██║ █╗ ██║█████╗  ███████║   ██║   ███████║█████╗  ██████╔╝
██║███╗██║██╔══╝  ██╔══██║   ██║   ██╔══██║██╔══╝  ██╔══██╗
╚███╔███╔╝███████╗██║  ██║   ██║   ██║  ██║███████╗██║  ██║
 ╚══╝╚══╝ ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
";

// --- Static Map and Region Definitions ---
const UK: Country = Country {
    map_template: &[
        "                                SSSSSSSSSSSSSSS                         ",
        "                              SSSSSSSSSSSSSSSSSSS                       ",
        "                            SSSSSSSSSSSSSSSSSSSSSSS                     ",
        "                          SSSSSSSSSSSSSSSSSSSSSSSSSS                    ",
        "                        SSSSSSSSSSSSSSSSSSSSSSSSSSSSSS                  ",
        "      IIIIIIIIII      SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS                  ",
        "    IIIIIIIIIIIIII    SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS                  ",
        "  IIIIIIIIIIIIIIIIII SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS                    ",
        "  IIIIIIIIIIIIIIIIII SSSSSSSSSSSSSSSSSSSSSSSSSSS                        ",
        "  IIIIIIIIIIIIIIII    NNNNNNNNNNNNNNNNSSSSSSSS                          ",
        "    IIIIIIIIIIII      NNNNNNNNNNNNNNNNNNNNNN                            ",
        "      IIIIII          NNNNNNNNNNNNNNNNNNNNNNNNNN                        ",
        "                      NNNNNNNNNNNNNNNNNNNNNNNNNNNN                      ",
        "                      NNNNNNNNNNNNNNNNNNNNNNNNNNNNNN                    ",
        "                      NNNNNNNNNNNNNNNNNNNNNNNNNNNNNN                    ",
        "        WWWWWWWW      NNNNNNNNNNNNNNNNNNNNNNNNNNNNNN                    ",
        "      WWWWWWWWWWWW    NNNNNNNNNNNNNNNNNNNNNNNNNN                        ",
        "    WWWWWWWWWWWWWWWW  NNNNNNNNNNNNNNNNNNNNNNNNNN                        ",
        "    WWWWWWWWWWWWWWWWWW  NNNNNNNNNNNNNNNNNNNN                            ",
        "    WWWWWWWWWWWWWWWWWWWW EEEEEENNNNNNNNNNNN                             ",
        "    WWWWWWWWWWWWWWWWWWWW EEEEEEEEEEEEE                                  ",
        "      WWWWWWWWWWWWWWWWWW EEEEEEEEEEEEEEE                                ",
        "        WWWWWWWWWWWWWW   EEEEEEEEEEEEEEEEEE                             ",
        "          WWWWWWWWWW     EEEEEEEEEEEEEEEEEEEEEE                         ",
        "                       EEEEEEEEEEEEEEEEEEEEEEEEEE                       ",
        "                     EEEEEEEEEEEEEEEEEEEEEEEEEEEE                       ",
        "                     EEEEEEEEEEEEEEEEEEEEEEEEEE                         ",
        "                       EEEEEEEEEEEEEEEEEEEEEE                           ",
        "                         EEEEEEEEEEEEEEEE                               ",
        "                           EEEEEEEEEE                                   ",
    ],
    regions: &[
        Region { name: "S. England", city: "London", char: 'E', temp_pos: (29, 12) },
        Region { name: "Wales", city: "Cardiff", char: 'W', temp_pos: (8, 9) },
        Region { name: "N. England", city: "Manchester", char: 'N', temp_pos: (24, 6) },
        Region { name: "Scotland", city: "Edinburgh", char: 'S', temp_pos: (24, 2) },
        Region { name: "N. Ireland", city: "Belfast", char: 'I', temp_pos: (4, 3) },
    ],
    left_text: &["TONIGHT:", "", "CLOUDY with", "patches of", "hill FOG", "", "RAIN", "moving in", "from the", "East"],
    footer_text: "Mainly DRY but a little RAIN in places later",
};

const GERMANY: Country = Country {
    map_template: &[
        "                      NNNNNNNNNNNNNNNNNNNNNN                          ",
        "                    NNNNNNNNNNNNNNNNNNNNNNNNNN                        ",
        "                  NNNNNNNNNNNNNNNNNNNNNNNNNNNNNN                      ",
        "  WWWWWW        NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN                      ",
        "WWWWWWWWWW    NNNNNNNNNNNNNNNNNNNNNNNNNEEEEEEEEE                      ",
        "WWWWWWWWWWWWWWNNNNNNNNNNNNNNNNNNNNNNNNEEEEEEEEEEEE                    ",
        "WWWWWWWWWWWWWWWWNNNNNNNNNNNNNNNNNNNNEEEEEEEEEEEEEEE                   ",
        "WWWWWWWWWWWWWWWWWWNNNNNNNNNNNNNNNNEEEEEEEEEEEEEEEEEE                  ",
        "WWWWWWWWWWWWWWWWWWWWNNNNNNNNNNNEEEEEEEEEEEEEEEEEEEEE                  ",
        "WWWWWWWWWWWWWWWWWWWWWNNNNNNNEEEEEEEEEEEEEEEEEEEEEEEE                  ",
        "WWWWWWWWWWWWWWWWWWWWWWWWNEEEEEEEEEEEEEEEEEEEEEEEEEEE                  ",
        "  WWWWWWWWWWWWWWWWWWWWWWEEEEEEEEEEEEEEEEEEEEEEEEEEEE                  ",
        "    WWWWWWWWWWWWWWWWWWSSSSSSSEEEEEEEEEEEEEEEEEEEEEEE                  ",
        "      WWWWWWWWWWWWWSSSSSSSSSSSSSEEEEEEEEEEEEEEEEEEEE                  ",
        "        WWWWWWWWSSSSSSSSSSSSSSSSSEEEEEEEEEEEEEEEEE                    ",
        "          WWWWSSSSSSSSSSSSSSSSSSSSSEEEEEEEEEEEEE                      ",
        "           WSSSSSSSSSSSSSSSSSSSSSSSSSEEEEEEEEE                        ",
        "          SSSSSSSSSSSSSSSSSSSSSSSSSSSSSEEEEE                          ",
        "         SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSEE                            ",
        "        SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS                              ",
        "       SSSSSSSSSSSSSSSSSSSSSSSSSSSSSSSS                               ",
        "      SSSSSSSSSSSSSSSSSSSSSSSSSSSSSS                                  ",
        "      SSSSSSSSSSSSSSSSSSSSSSSSSSSS                                    ",
        "       SSSSSSSSSSSSSSSSSSSSSSSS                                       ",
        "         SSSSSSSSSSSSSSSSSSSS                                         ",
        "           SSSSSSSSSSSSSSSS                                           ",
        "             SSSSSSSSSSSS                                             ",
        "               SSSSSSSS                                               ",
    ],
    regions: &[
        Region { name: "Nord", city: "Hamburg", char: 'N', temp_pos: (18, 2) },
        Region { name: "West", city: "Cologne", char: 'W', temp_pos: (6, 7) },
        Region { name: "Ost", city: "Berlin", char: 'E', temp_pos: (28, 7) },
        Region { name: "Süd", city: "Munich", char: 'S', temp_pos: (18, 12) },
    ],
    left_text: &["WETTER:", "", "Heute Nacht", "und Morgen:", "", "Meist", "trocken mit", "einigen", "Wolkenfeldern."],
    footer_text: "Meist trocken, aber später örtlich leichter Regen möglich",
};

// --- Application State Management ---
enum AppState<'a> {
    Loading,
    Loaded(AppData<'a>),
}

enum ViewState {
    Main,
    Details,
}

struct AppData<'a> {
    country: Country<'a>,
    reports: HashMap<&'a str, WeatherReport>,
    summaries: Vec<String>,
}

fn get_weather_data(client: &reqwest::blocking::Client, city: &str) -> Result<WeatherReport, reqwest::Error> {
    let url = format!("https://wttr.in/{}?format=j1", city);
    client.get(url).send()?.json::<WeatherReport>()
}

fn get_temp_color(temp: i32) -> Color {
    match temp {
        t if t < 10 => CEEFAX_GREEN,
        t if (10..15).contains(&t) => CEEFAX_CYAN,
        _ => CEEFAX_YELLOW,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let country_config: Country<'static> = match cli.country.to_lowercase().as_str() {
        "germany" | "de" => GERMANY,
        _ => UK,
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal, country_config)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, country: Country<'static>) -> io::Result<()> {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let client = reqwest::blocking::Client::new();
        let mut weather_reports = HashMap::new();
        let mut summaries = Vec::new();
        for region in country.regions {
            if let Ok(report) = get_weather_data(&client, region.city) {
                if let Some(condition) = report.current_condition.first() {
                    // Corrected: Use weatherDesc to match the struct field name
                    let desc = condition.weatherDesc.first().map_or("N/A", |d| &d.value);
                    summaries.push(format!("{}: {}", region.name, desc));
                    weather_reports.insert(region.name, report.clone());
                }
            }
        }
        let _ = tx.send(AppData { country, reports: weather_reports, summaries });
    });

    let mut app_state = AppState::Loading;
    let mut view_state = ViewState::Main;
    let mut counter: u16 = 100;

    loop {
        terminal.draw(|f| match &app_state {
            AppState::Loading => loading_ui(f, counter),
            AppState::Loaded(data) => match view_state {
                ViewState::Main => ui(f, data),
                ViewState::Details => details_ui(f, data),
            },
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match view_state {
                    ViewState::Main => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('d') => view_state = ViewState::Details,
                        _ => {}
                    },
                    ViewState::Details => match key.code {
                        KeyCode::Char('m') | KeyCode::Esc => view_state = ViewState::Main,
                        _ => {}
                    },
                }
            }
        }

        if let Ok(data) = rx.try_recv() {
            app_state = AppState::Loaded(data);
        }

        if matches!(app_state, AppState::Loading) {
            counter = 100 + (counter + 1 - 100) % 800;
        }
    }
}

fn loading_ui(f: &mut Frame, counter: u16) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(f.size());

    let title_style = Style::default().fg(CEEFAX_WHITE).bg(CEEFAX_BLACK);
    let time_style = Style::default().fg(CEEFAX_YELLOW).bg(CEEFAX_BLACK);
    let left_text = format!("P{} SEARCHING...", counter);
    let date_text = Local::now().format("%a %d %b").to_string().to_uppercase();
    let time_text = Local::now().format("%H:%M/%S").to_string();
    
    let full_right_text_len = date_text.len() + time_text.len() + 3;
    let padding_len = if f.size().width as usize > left_text.len() + full_right_text_len {
        f.size().width as usize - left_text.len() - full_right_text_len
    } else { 0 };
    let padding = " ".repeat(padding_len);

    let header_line = Line::from(vec![
        Span::styled(left_text, title_style.bold()),
        Span::styled(padding, title_style),
        Span::styled(date_text, title_style),
        Span::styled("   ", title_style),
        Span::styled(time_text, time_style),
    ]);
    let header_widget = Paragraph::new(header_line);

    let loading_body = Paragraph::new("\n\n\nSearching...")
        .style(Style::default().fg(CEEFAX_WHITE).bg(CEEFAX_BLUE))
        .alignment(Alignment::Center);

    f.render_widget(Block::default().style(Style::default().bg(CEEFAX_BLUE)), f.size());
    f.render_widget(header_widget, chunks[0]);
    f.render_widget(loading_body, chunks[1]);
}

fn ui(f: &mut Frame, data: &AppData) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(2)])
        .split(f.size());

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(main_chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(10)])
        .split(content_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(10)])
        .split(content_chunks[1]);

    let title_style = Style::default().fg(CEEFAX_WHITE).bg(CEEFAX_BLACK);
    let time_style = Style::default().fg(CEEFAX_YELLOW).bg(CEEFAX_BLACK);
    let left_text = "P181 CEEFAX 181";
    let date_text = Local::now().format("%a %d %b").to_string().to_uppercase();
    let time_text = Local::now().format("%H:%M/%S").to_string();
    
    let full_right_text_len = date_text.len() + time_text.len() + 3;
    let padding_len = if f.size().width as usize > left_text.len() + full_right_text_len {
        f.size().width as usize - left_text.len() - full_right_text_len
    } else { 0 };
    let padding = " ".repeat(padding_len);

    let header_line = Line::from(vec![
        Span::styled(left_text, title_style),
        Span::styled(padding, title_style),
        Span::styled(date_text, title_style),
        Span::styled("   ", title_style),
        Span::styled(time_text, time_style),
    ]);
    let header_widget = Paragraph::new(header_line);

    let blue_bg_style = Style::default().fg(CEEFAX_WHITE).bg(CEEFAX_BLUE);
    let title_widget = Paragraph::new(WEATHER_TITLE).style(blue_bg_style.bold());
    let left_text_widget = Paragraph::new(Text::from(data.country.left_text.join("\n"))).style(blue_bg_style);
    let right_text_widget = Paragraph::new(Text::from(data.summaries.join("\n"))).style(blue_bg_style);
    let map_widget = draw_map_widget(&data.country, &data.reports);
    let footer_widget = Paragraph::new(format!("[D]etails      {}", data.country.footer_text)).style(blue_bg_style);

    f.render_widget(Block::default().style(blue_bg_style), f.size());
    f.render_widget(header_widget, main_chunks[0]);
    f.render_widget(title_widget, left_chunks[0]);
    f.render_widget(left_text_widget, left_chunks[1]);
    f.render_widget(right_text_widget, right_chunks[0]);
    f.render_widget(map_widget, right_chunks[1]);
    f.render_widget(footer_widget, main_chunks[2]);
}

fn details_ui(f: &mut Frame, data: &AppData) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(f.size());

    let title_style = Style::default().fg(CEEFAX_WHITE).bg(CEEFAX_BLACK);
    let header_text = "P182 Weather Details";
    let header_widget = Paragraph::new(header_text).style(title_style.bold());

    let mut details_text = Vec::new();
    for region in data.country.regions {
        if let Some(report) = data.reports.get(region.name) {
            let condition = &report.current_condition[0];
            details_text.push(Line::from(Span::styled(format!("-- {} --", region.name), Style::default().fg(CEEFAX_YELLOW).bold())));
            details_text.push(Line::from(format!("  Feels Like: {}°C", condition.FeelsLikeC)));
            details_text.push(Line::from(format!("  Wind: {} {} km/h", condition.winddir16Point, condition.windspeedKmph)));
            details_text.push(Line::from(format!("  Precip: {} mm", condition.precipMM)));
            details_text.push(Line::from(" ")); // Spacer
        }
    }
    
    let blue_bg_style = Style::default().fg(CEEFAX_WHITE).bg(CEEFAX_BLUE);
    let details_widget = Paragraph::new(details_text)
        .block(Block::default().style(blue_bg_style))
        .wrap(Wrap { trim: true });

    let footer_widget = Paragraph::new("[M]ap View").style(blue_bg_style);

    f.render_widget(Block::default().style(blue_bg_style), f.size());
    f.render_widget(header_widget, main_chunks[0]);
    f.render_widget(details_widget, main_chunks[1]);
    f.render_widget(footer_widget, main_chunks[2]);
}

fn draw_map_widget<'a>(country: &Country, reports: &HashMap<&str, WeatherReport>) -> Paragraph<'a> {
    let mut lines: Vec<Line> = Vec::new();
    let template = country.map_template;

    for y in (0..template.len()).step_by(2) {
        let mut spans: Vec<Span> = Vec::new();
        for x in (0..template[y].len()).step_by(2) {
            let tl = template[y].chars().nth(x).unwrap_or(' ');
            let tr = template[y].chars().nth(x + 1).unwrap_or(' ');
            let bl = if y + 1 < template.len() { template[y + 1].chars().nth(x).unwrap_or(' ') } else { ' ' };
            let br = if y + 1 < template.len() { template[y + 1].chars().nth(x + 1).unwrap_or(' ') } else { ' ' };

            // Corrected: Removed unused 'pixels' variable
            let mut land_pixels = HashMap::new();
            let mut bitmask = 0;

            if tl != ' ' { bitmask |= 1; *land_pixels.entry(tl).or_insert(0) += 1; }
            if tr != ' ' { bitmask |= 2; *land_pixels.entry(tr).or_insert(0) += 1; }
            if bl != ' ' { bitmask |= 4; *land_pixels.entry(bl).or_insert(0) += 1; }
            if br != ' ' { bitmask |= 8; *land_pixels.entry(br).or_insert(0) += 1; }

            let dominant_char = land_pixels.into_iter().max_by_key(|&(_, count)| count).map(|(c, _)| c);
            let mut bg_color = CEEFAX_BLUE;
            if let Some(dc) = dominant_char {
                for region in country.regions {
                    if region.char == dc {
                        if let Some(report) = reports.get(region.name) {
                            let temp = report.current_condition[0].temp_C.parse::<i32>().unwrap_or(0);
                            bg_color = get_temp_color(temp);
                        }
                        break;
                    }
                }
            }
            
            let mosaic_char = TELETEXT_CHARS[bitmask];
            spans.push(Span::styled(mosaic_char.to_string(), Style::new().bg(bg_color)));
        }
        lines.push(Line::from(spans));
    }
    
    for region in country.regions {
        if let Some(report) = reports.get(region.name) {
            let temp_str = &report.current_condition[0].temp_C;
            let (temp_x, temp_y) = (region.temp_pos.0 / 2, region.temp_pos.1 / 2);

            if (temp_y as usize) < lines.len() {
                for (i, temp_digit) in temp_str.chars().enumerate() {
                    let x_pos = (temp_x as usize) + i;
                    if x_pos < lines[temp_y as usize].spans.len() {
                        let original_span = &lines[temp_y as usize].spans[x_pos];
                        let bg_color = original_span.style.bg.unwrap_or(CEEFAX_BLUE);
                        lines[temp_y as usize].spans[x_pos] = Span::styled(
                            temp_digit.to_string(),
                            Style::new().fg(CEEFAX_WHITE).bold().bg(bg_color),
                        );
                    }
                }
            }
        }
    }

    Paragraph::new(Text::from(lines))
}

