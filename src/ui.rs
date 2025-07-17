use crate::{app::AppData, config, wttr};
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Wrap},
    Frame,
};
use std::collections::HashMap;

pub fn loading_ui(f: &mut Frame, counter: u16) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(f.size());

    let title_style = Style::default().fg(config::CEEFAX_WHITE).bg(config::CEEFAX_BLACK);
    let time_style = Style::default().fg(config::CEEFAX_YELLOW).bg(config::CEEFAX_BLACK);
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
        .style(Style::default().fg(config::CEEFAX_WHITE).bg(config::CEEFAX_BLUE))
        .alignment(Alignment::Center);

    f.render_widget(Block::default().style(Style::default().bg(config::CEEFAX_BLUE)), f.size());
    f.render_widget(header_widget, chunks[0]);
    f.render_widget(loading_body, chunks[1]);
}

pub fn error_ui(f: &mut Frame, error: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(f.size());

    let title_style = Style::default().fg(config::CEEFAX_WHITE).bg(config::CEEFAX_BLACK);
    let header_text = "P404 ERROR";
    let header_widget = Paragraph::new(header_text).style(title_style.bold());

    let blue_bg_style = Style::default().fg(config::CEEFAX_WHITE).bg(config::CEEFAX_BLUE);
    let error_body = Paragraph::new(format!("\n\nAn error occurred:\n\n{}", error))
        .style(blue_bg_style)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    
    let footer_widget = Paragraph::new("[R]etry      [Q]uit").style(blue_bg_style);

    f.render_widget(Block::default().style(blue_bg_style), f.size());
    f.render_widget(header_widget, chunks[0]);
    f.render_widget(error_body, chunks[1]);
    f.render_widget(footer_widget, chunks[2]);
}

pub fn main_ui(f: &mut Frame, data: &AppData, updated_at: &DateTime<Local>) {
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

    let title_style = Style::default().fg(config::CEEFAX_WHITE).bg(config::CEEFAX_BLACK);
    let time_style = Style::default().fg(config::CEEFAX_YELLOW).bg(config::CEEFAX_BLACK);
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

    let blue_bg_style = Style::default().fg(config::CEEFAX_WHITE).bg(config::CEEFAX_BLUE);
    let title_widget = Paragraph::new(config::WEATHER_TITLE).style(blue_bg_style.bold());
    let left_text_widget = Paragraph::new(Text::from(data.country.left_text.join("\n"))).style(blue_bg_style);
    let right_text_widget = Paragraph::new(Text::from(data.summaries.join("\n"))).style(blue_bg_style);
    let map_widget = draw_map_widget(&data.country, &data.reports);
    let footer_text = format!(
        "[D]etails [R]efresh      Updated: {}      {}",
        updated_at.format("%H:%M:%S"),
        data.country.footer_text
    );
    let footer_widget = Paragraph::new(footer_text).style(blue_bg_style);

    f.render_widget(Block::default().style(blue_bg_style), f.size());
    f.render_widget(header_widget, main_chunks[0]);
    f.render_widget(title_widget, left_chunks[0]);
    f.render_widget(left_text_widget, left_chunks[1]);
    f.render_widget(right_text_widget, right_chunks[0]);
    f.render_widget(map_widget, right_chunks[1]);
    f.render_widget(footer_widget, main_chunks[2]);
}

pub fn details_ui(f: &mut Frame, data: &AppData) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)])
        .split(f.size());

    let title_style = Style::default().fg(config::CEEFAX_WHITE).bg(config::CEEFAX_BLACK);
    let header_text = "P182 Weather Details";
    let header_widget = Paragraph::new(header_text).style(title_style.bold());

    let mut details_text = Vec::new();
    for region in data.country.regions {
        if let Some(report) = data.reports.get(region.name) {
            let condition = &report.current_condition[0];
            details_text.push(Line::from(Span::styled(format!("-- {} --", region.name), Style::default().fg(config::CEEFAX_YELLOW).bold())));
            details_text.push(Line::from(format!("  Feels Like: {}°C", condition.FeelsLikeC)));
            details_text.push(Line::from(format!("  Wind: {} {} km/h", condition.winddir16Point, condition.windspeedKmph)));
            details_text.push(Line::from(format!("  Precip: {} mm", condition.precipMM)));
            details_text.push(Line::from(" "));
        }
    }
    
    let blue_bg_style = Style::default().fg(config::CEEFAX_WHITE).bg(config::CEEFAX_BLUE);
    let details_widget = Paragraph::new(details_text)
        .style(blue_bg_style) // Explicitly style the paragraph itself
        .block(Block::default().style(blue_bg_style))
        .wrap(Wrap { trim: true });

    let footer_widget = Paragraph::new("[M]ap View").style(blue_bg_style);

    f.render_widget(Block::default().style(blue_bg_style), f.size());
    f.render_widget(header_widget, main_chunks[0]);
    f.render_widget(details_widget, main_chunks[1]);
    f.render_widget(footer_widget, main_chunks[2]);
}

fn draw_map_widget<'a>(country: &config::Country, reports: &wttr::WeatherReports) -> Paragraph<'a> {
    let mut lines: Vec<Line> = Vec::new();
    let template = country.map_template;

    for y in (0..template.len()).step_by(2) {
        let mut spans: Vec<Span> = Vec::new();
        for x in (0..template[y].len()).step_by(2) {
            let tl = template[y].chars().nth(x).unwrap_or(' ');
            let tr = template[y].chars().nth(x + 1).unwrap_or(' ');
            let bl = if y + 1 < template.len() { template[y + 1].chars().nth(x).unwrap_or(' ') } else { ' ' };
            let br = if y + 1 < template.len() { template[y + 1].chars().nth(x + 1).unwrap_or(' ') } else { ' ' };

            let mut land_pixels = HashMap::new();
            let mut bitmask = 0;

            if tl != ' ' { bitmask |= 1; *land_pixels.entry(tl).or_insert(0) += 1; }
            if tr != ' ' { bitmask |= 2; *land_pixels.entry(tr).or_insert(0) += 1; }
            if bl != ' ' { bitmask |= 4; *land_pixels.entry(bl).or_insert(0) += 1; }
            if br != ' ' { bitmask |= 8; *land_pixels.entry(br).or_insert(0) += 1; }

            let dominant_char = land_pixels.into_iter().max_by_key(|&(_, count)| count).map(|(c, _)| c);
            let mut bg_color = config::CEEFAX_BLUE;
            if let Some(dc) = dominant_char {
                for region in country.regions {
                    if region.char == dc {
                        if let Some(report) = reports.get(region.name) {
                            let temp = report.current_condition[0].temp_C.parse::<i32>().unwrap_or(0);
                            bg_color = wttr::get_temp_color(temp);
                        }
                        break;
                    }
                }
            }
            
            let mosaic_char = config::TELETEXT_CHARS[bitmask];
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
                        let bg_color = original_span.style.bg.unwrap_or(config::CEEFAX_BLUE);
                        lines[temp_y as usize].spans[x_pos] = Span::styled(
                            temp_digit.to_string(),
                            Style::new().fg(config::CEEFAX_WHITE).bold().bg(bg_color),
                        );
                    }
                }
            }
        }
    }

    // Explicitly set the background for the whole paragraph block to fix rendering glitches
    Paragraph::new(Text::from(lines))
        .block(Block::default().style(Style::default().bg(config::CEEFAX_BLUE)))
}

