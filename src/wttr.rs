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

/// The trait that defines our contract for any weather data provider.
pub trait WeatherClient: Send + Sync + 'static {
    fn fetch(&self, city: &str) -> Result<WeatherReport, String>;
}

/// The implementation that makes real network calls to wttr.in.
pub struct LiveWeatherClient {
    client: reqwest::blocking::Client,
}

impl LiveWeatherClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl WeatherClient for LiveWeatherClient {
    fn fetch(&self, city: &str) -> Result<WeatherReport, String> {
        let url = format!("https://wttr.in/{}?format=j1", city);
        let response = self
            .client
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
}

pub fn get_temp_color(temp: i32) -> Color {
    match temp {
        t if t < 10 => config::CEEFAX_GREEN,
        t if (10..15).contains(&t) => config::CEEFAX_CYAN,
        _ => config::CEEFAX_YELLOW,
    }
}

/// Maps a weather description string to a Unicode symbol string slice.
pub fn get_weather_icon(description: &str) -> &'static str {
    let desc_lower = description.to_lowercase();
    match desc_lower {
        s if s.contains("sunny") => "☀️",
        s if s.contains("clear") => "🌙",
        s if s.contains("partly cloudy") => "⛅",
        s if s.contains("cloudy") => "☁️",
        s if s.contains("overcast") => "🌥️",
        s if s.contains("mist") | s.contains("fog") => "🌫️",
        s if s.contains("drizzle") | s.contains("light rain") => "🌦️",
        s if s.contains("rain") | s.contains("shower") => "🌧️",
        s if s.contains("sleet") => "🌨️",
        s if s.contains("snow") => "❄️",
        s if s.contains("thunder") => "🌩️",
        _ => "?",
    }
}


// --- Unit and Integration Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_colors() {
        assert_eq!(get_temp_color(5), config::CEEFAX_GREEN);
        assert_eq!(get_temp_color(14), config::CEEFAX_CYAN);
        assert_eq!(get_temp_color(25), config::CEEFAX_YELLOW);
    }

    /// A mock client for testing without network access.
    struct MockWeatherClient {
        mock_data: String,
    }

    impl WeatherClient for MockWeatherClient {
        fn fetch(&self, _city: &str) -> Result<WeatherReport, String> {
            serde_json::from_str(&self.mock_data)
                .map_err(|e| format!("Mock data parsing failed: {}", e))
        }
    }

    /// An integration-style test for the data fetching logic.
    #[test]
    fn test_successful_data_fetch_with_mock() {
        let mock_json = r#"
        {
            "current_condition": [
                {
                    "temp_C": "15",
                    "FeelsLikeC": "14",
                    "windspeedKmph": "10",
                    "winddir16Point": "W",
                    "precipMM": "0.0",
                    "weatherDesc": [{"value": "Sunny"}]
                }
            ],
            "weather": [
                {
                    "hourly": [
                        {"time": "0", "tempC": "10", "weatherDesc": [{"value": "Clear"}]},
                        {"time": "300", "tempC": "12", "weatherDesc": [{"value": "Partly cloudy"}]}
                    ]
                }
            ]
        }
        "#;

        let mock_client = MockWeatherClient {
            mock_data: mock_json.to_string(),
        };

        let result = mock_client.fetch("test-city");
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.current_condition[0].temp_C, "15");
        assert_eq!(report.weather[0].hourly.len(), 2);
    }

    #[test]
    fn test_weather_icons() {
        assert_eq!(get_weather_icon("Sunny"), "☀️");
        assert_eq!(get_weather_icon("Light rain shower"), "🌦️");
        assert_eq!(get_weather_icon("Heavy snow"), "❄️");
        assert_eq!(get_weather_icon("Thundery outbreaks possible"), "🌩️");
        assert_eq!(get_weather_icon("Unknown description"), "?");
    }
}

