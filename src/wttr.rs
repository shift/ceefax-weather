use crate::config;
use ratatui::style::Color;
use serde::Deserialize;
use std::collections::HashMap;

pub type WeatherReports = HashMap<String, WeatherReport>;

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherDesc {
    pub value: String,
}

#[derive(Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct Hourly {
    pub time: String,
    pub tempC: String,
    pub weatherDesc: Vec<WeatherDesc>,
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
pub struct WeatherDay {
    pub hourly: Vec<Hourly>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WeatherReport {
    pub current_condition: Vec<CurrentCondition>,
    pub weather: Vec<WeatherDay>,
}

pub fn get_weather_data(
    client: &reqwest::blocking::Client,
    city: &str,
) -> Result<WeatherReport, String> {
    let url = format!("https://wttr.in/{}?format=j1", city);
    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Network request failed: {}", e))?;

    let text = response
        .text()
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    match serde_json::from_str::<WeatherReport>(&text) {
        Ok(report) => Ok(report),
        Err(e) => {
            let pretty_payload = match serde_json::from_str::<serde_json::Value>(&text) {
                Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| text.clone()),
                Err(_) => text,
            };
            Err(format!(
                "Failed to decode API response: {}\n\n-- API Payload --\n{}",
                e, pretty_payload
            ))
        }
    }
}

pub fn get_temp_color(temp: i32) -> Color {
    match temp {
        t if t < 10 => config::CEEFAX_GREEN,
        t if (10..15).contains(&t) => config::CEEFAX_CYAN,
        _ => config::CEEFAX_YELLOW,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_colors() {
        assert_eq!(get_temp_color(5), config::CEEFAX_GREEN);
        assert_eq!(get_temp_color(14), config::CEEFAX_CYAN);
        assert_eq!(get_temp_color(25), config::CEEFAX_YELLOW);
    }
}

