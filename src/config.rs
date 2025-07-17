use clap::Parser;
use ratatui::style::Color;
use std::time::Duration;

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
#[derive(Clone, Copy)]
pub struct Region<'a> {
    pub name: &'a str,
    pub city: &'a str,
    pub char: char,
    pub temp_pos: (u16, u16),
}

#[derive(Clone, Copy)]
pub struct Country<'a> {
    pub map_template: &'a [&'a str],
    pub regions: &'a [Region<'a>],
    pub left_text: &'a [&'a str],
    pub footer_text: &'a str,
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

// --- Static Map and Region Definitions ---
pub const UK: Country = Country {
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

pub const GERMANY: Country = Country {
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

