use crate::config;
use ratatui::style::Color;
use serde::Deserialize;
use std::collections::HashMap;

pub type WeatherReports<'a> = HashMap<&'a str, WeatherReport>;

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherDesc {
    pub value: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct CurrentCondition {
    pub temp_C: String,
    pub FeelsLikeC: String,
    pub windspeedKmph: String,
    pub winddir16Point: String,
    pub precipMM: String,
    pub weatherDesc: Vec<WeatherDesc>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherReport {
    pub current_condition: Vec<CurrentCondition>,
}

pub fn get_weather_data(
    client: &reqwest::blocking::Client,
    city: &str,
) -> Result<WeatherReport, reqwest::Error> {
    let url = format!("https://wttr.in/{}?format=j1", city);
    client.get(url).send()?.json::<WeatherReport>()
}

pub fn get_temp_color(temp: i32) -> Color {
    match temp {
        t if t < 10 => config::CEEFAX_GREEN,
        t if (10..15).contains(&t) => config::CEEFAX_CYAN,
        _ => config::CEEFAX_YELLOW,
    }
}

// --- Unit Tests ---
// This module will only be compiled when running `cargo test`.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_colors() {
        // Test below 10 degrees
        assert_eq!(get_temp_color(5), config::CEEFAX_GREEN);
        assert_eq!(get_temp_color(-2), config::CEEFAX_GREEN);

        // Test between 10 and 14 degrees
        assert_eq!(get_temp_color(10), config::CEEFAX_CYAN);
        assert_eq!(get_temp_color(14), config::CEEFAX_CYAN);

        // Test 15 degrees and above
        assert_eq!(get_temp_color(15), config::CEEFAX_YELLOW);
        assert_eq!(get_temp_color(25), config::CEEFAX_YELLOW);
    }
}

