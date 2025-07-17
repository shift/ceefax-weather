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
    widgets::{Block, Paragraph},
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

// --- Command Line Argument Parsing ---
#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
struct Cli {
    /// The country to display the weather map for (uk or germany)
    #[arg(short, long, value_name = "COUNTRY", default_value = "uk")]
    country: String,
}

// --- Data Structures for wttr.in JSON Response ---
#[derive(Deserialize, Debug, Clone)]
struct WeatherDesc {
    value: String,
}

#[derive(Deserialize, Debug, Clone)]
struct CurrentCondition {
    #[serde(rename = "temp_C")]
    temp_c: String,
    #[serde(rename = "weatherDesc")]
    weather_desc: Vec<WeatherDesc>,
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
        "                      SSSSS       ",
        "                     SSSSSSS      ",
        "   IIIII            SSSSSSSS      ",
        "   IIIII             SSSSSS       ",
        "    III             NNNNNNNN      ",
        "                   NNNNNNNNN      ",
        "                  NNNNNNNNNN      ",
        "      WWWWWW      NNNNNNNNN       ",
        "     WWWWWWWW      NNNNNN         ",
        "    WWWWWWWWWW      EEEEEE        ",
        "     WWWWWWWW      EEEEEEEE       ",
        "      WWWWW       EEEEEEEEEE      ",
        "                 EEEEEEEEEEEE     ",
        "                EEEEEEEEEEEE      ",
        "                 EEEEEEEE         ",
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
        "           NNNNNNNNNNN            ",
        "          NNNNNNNNNNNNN           ",
        "         NNNNNNNNNNNNNNN          ",
        "   WWWWWWWWNNNNNNNNNEEEEEEE       ",
        "  WWWWWWWWWWWWNNNNNEEEEEEEEE      ",
        " WWWWWWWWWWWWWWWWNEEEEEEEEEEE     ",
        "WWWWWWWWWWWWWWWWWNEEEEEEEEEEE     ",
        "WWWWWWWWWWWWWWWWEEEEEEEEEEEE      ",
        " WWWWWWWWWWWWWWEEEEEEEEEEEEE      ",
        "  WWWWWWWWWWWWSSSSSEEEEEEEEE      ",
        "   WWWWWWWWWSSSSSSSSSEEEEE        ",
        "     WWWWWSSSSSSSSSSSSSS          ",
        "      WSSSSSSSSSSSSSSSS           ",
        "       SSSSSSSSSSSSSSS            ",
        "        SSSSSSSSSSSS              ",
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

struct AppData<'a> {
    country: Country<'a>,
    reports: HashMap<&'a str, WeatherReport>,
    summaries: Vec<String>,
}

/// Fetches weather data for a given city from wttr.in.
fn get_weather_data(client: &reqwest::blocking::Client, city: &str) -> Result<WeatherReport, reqwest::Error> {
    let url = format!("https://wttr.in/{}?format=j1", city);
    client.get(url).send()?.json::<WeatherReport>()
}

/// Returns a ratatui color based on the temperature in Celsius.
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
                    let desc = condition.weather_desc.first().map_or("N/A", |d| &d.value);
                    summaries.push(format!("{}: {}", region.name, desc));
                    weather_reports.insert(region.name, report.clone());
                }
            }
        }
        let _ = tx.send(AppData { country, reports: weather_reports, summaries });
    });

    let mut app_data: Option<AppData> = None;
    let mut counter: u16 = 100;

    loop {
        terminal.draw(|f| {
            if let Some(data) = &app_data {
                ui(f, data);
            } else {
                loading_ui(f, counter);
            }
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char(_) | KeyCode::Esc = key.code {
                    return Ok(());
                }
            }
        }

        if let Ok(data) = rx.try_recv() {
            app_data = Some(data);
        }

        if app_data.is_none() {
            counter = 100 + (counter + 1 - 100) % 800;
        }
    }
}

/// Renders the loading screen UI.
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
    } else {
        0
    };
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

/// The main UI drawing function.
fn ui(f: &mut Frame, data: &AppData) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(f.size());

    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(55),
        ])
        .split(main_chunks[1]);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(10),
        ])
        .split(content_chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(10),
        ])
        .split(content_chunks[1]);

    let title_style = Style::default().fg(CEEFAX_WHITE).bg(CEEFAX_BLACK);
    let time_style = Style::default().fg(CEEFAX_YELLOW).bg(CEEFAX_BLACK);
    let left_text = "P181 CEEFAX 181";
    let date_text = Local::now().format("%a %d %b").to_string().to_uppercase();
    let time_text = Local::now().format("%H:%M/%S").to_string();
    
    let full_right_text_len = date_text.len() + time_text.len() + 3;
    let padding_len = if f.size().width as usize > left_text.len() + full_right_text_len {
        f.size().width as usize - left_text.len() - full_right_text_len
    } else {
        0
    };
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
    // Corrected: Pass a reference to data.country
    let map_widget = draw_map_widget(&data.country, &data.reports);
    let footer_widget = Paragraph::new(data.country.footer_text).style(blue_bg_style);

    f.render_widget(Block::default().style(blue_bg_style), f.size());
    f.render_widget(header_widget, main_chunks[0]);
    f.render_widget(title_widget, left_chunks[0]);
    f.render_widget(left_text_widget, left_chunks[1]);
    f.render_widget(right_text_widget, right_chunks[0]);
    f.render_widget(map_widget, right_chunks[1]);
    f.render_widget(footer_widget, main_chunks[2]);
}

/// Creates the map widget by drawing the colored regions and overlaying temperatures.
fn draw_map_widget<'a>(country: &Country, reports: &HashMap<&str, WeatherReport>) -> Paragraph<'a> {
    let mut lines: Vec<Line> = Vec::new();

    for (y, line_str) in country.map_template.iter().enumerate() {
        let mut spans: Vec<Span> = Vec::new();
        for (x, template_char) in line_str.chars().enumerate() {
            let mut bg_color = CEEFAX_BLUE;
            let mut is_land = false;

            if template_char != ' ' {
                for region in country.regions {
                    if region.char == template_char {
                        if let Some(report) = reports.get(region.name) {
                            let temp = report.current_condition[0].temp_c.parse::<i32>().unwrap_or(0);
                            bg_color = get_temp_color(temp);
                        }
                        is_land = true;
                        break;
                    }
                }
            }

            let mut temp_char = None;
            for region in country.regions {
                if let Some(report) = reports.get(region.name) {
                    let temp_str = &report.current_condition[0].temp_c;
                    let (temp_x, temp_y) = region.temp_pos;
                    if y as u16 == temp_y && (x as u16 >= temp_x) && (x < (temp_x + temp_str.len() as u16) as usize) {
                        temp_char = temp_str.chars().nth((x as u16 - temp_x) as usize);
                        break;
                    }
                }
            }

            if let Some(tc) = temp_char {
                spans.push(Span::styled(tc.to_string(), Style::new().fg(CEEFAX_WHITE).bold().bg(bg_color)));
            } else {
                let char_to_draw = if is_land { "█" } else { " " };
                spans.push(Span::styled(char_to_draw, Style::new().bg(bg_color)));
            }
        }
        lines.push(Line::from(spans));
    }

    Paragraph::new(Text::from(lines))
}

