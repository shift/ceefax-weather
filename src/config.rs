use clap::Parser;
use ratatui::style::Color;
use serde::Deserialize;
use std::{env, fs, io, time::Duration};

// --- CEEFAX Color Palette ---
pub const CEEFAX_BLUE: Color = Color::Rgb(0, 0, 170);
pub const CEEFAX_GREEN: Color = Color::Rgb(0, 204, 0);
pub const CEEFAX_CYAN: Color = Color::Rgb(0, 204, 204);
pub const CEEFAX_YELLOW: Color = Color::Rgb(204, 204, 0);
pub const CEEFAX_WHITE: Color = Color::Rgb(255, 255, 255);
pub const CEEFAX_BLACK: Color = Color::Rgb(0, 0, 0);

// --- Unicode Teletext Mosaic Characters ---
pub const TELETEXT_CHARS: [char; 16] = [
    ' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█',
];

// --- Application Configuration ---
pub const REFRESH_INTERVAL: Duration = Duration::from_secs(15 * 60); // 15 minutes

// --- Command Line Argument Parsing ---
#[derive(Parser, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "COUNTRY", default_value = "uk")]
    pub country: String,
}

// --- Map Configuration Structures ---
#[derive(Clone, Deserialize)]
pub struct Region {
    pub name: String,
    pub city: String,
    pub char: char,
    pub temp_pos: [u16; 2],
}

#[derive(Clone, Deserialize)]
pub struct Country {
    pub map_template: Vec<String>,
    pub regions: Vec<Region>,
}

// --- ASCII Art ---
pub const WEATHER_TITLE: &str = "
██╗    ██╗███████╗ █████╗ ████████╗██╗  ██╗███████╗██████╗ 
██║    ██║██╔════╝██╔══██╗╚══██╔══╝██║  ██║██╔════╝██╔══██╗
██║ █╗ ██║█████╗  ███████║   ██║   ███████║█████╗  ██████╔╝
██║███╗██║██╔══╝  ██╔══██║   ██║   ██╔══██║██╔══╝  ██╔══██╗
╚███╔███╔╝███████╗██║  ██║   ██║   ██║  ██║███████╗██║  ██║
 ╚══╝╚══╝ ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
";

/// Loads a country configuration from a TOML file.
pub fn load_country_config(name: &str) -> Result<Country, Box<dyn std::error::Error>> {
    let mut exe_path = env::current_exe()?;
    exe_path.pop();
    let filename = exe_path.join("templates").join(format!("{}.toml", name));
    
    let config_str = fs::read_to_string(&filename)
        .map_err(|e| format!("Failed to read config file at {:?}: {}", filename, e))?;
    
    let country: Country = toml::from_str(&config_str)
        .map_err(|e| format!("Failed to parse TOML from {:?}: {}", filename, e))?;
        
    Ok(country)
}

/// Scans the templates directory and returns a list of available country names.
pub fn get_available_countries() -> io::Result<Vec<String>> {
    let mut exe_path = env::current_exe()?;
    exe_path.pop();
    let templates_path = exe_path.join("templates");
    
    let mut countries = Vec::new();
    for entry in fs::read_dir(templates_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                countries.push(stem.to_string());
            }
        }
    }
    Ok(countries)
}

