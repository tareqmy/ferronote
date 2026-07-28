use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    pub border_active: Color,
    pub border_inactive: Color,
    pub title: Color,
    pub search_fg: Color,
    pub search_match: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub accent: Color,
}

impl ThemePalette {
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "gruvbox" => Self {
                border_active: Color::Rgb(250, 189, 47), // Gruvbox Yellow #FABD2F
                border_inactive: Color::Rgb(146, 131, 116), // Gruvbox Gray #928374
                title: Color::Rgb(251, 241, 199),        // Gruvbox Light #FBF1C7
                search_fg: Color::Rgb(250, 189, 47),
                search_match: Color::Rgb(184, 187, 38), // Gruvbox Green #B8BB26
                selection_bg: Color::Rgb(69, 133, 136), // Gruvbox Teal #458588
                selection_fg: Color::Rgb(251, 241, 199),
                accent: Color::Rgb(211, 134, 155), // Gruvbox Purple #D3869B
            },
            "nord" => Self {
                border_active: Color::Rgb(136, 192, 208), // Nord Cyan #88C0D0
                border_inactive: Color::Rgb(76, 86, 106), // Nord Dark Gray #4C566A
                title: Color::Rgb(236, 239, 244),         // Nord Snow #ECEFF4
                search_fg: Color::Rgb(129, 161, 193),     // Nord Blue #81A1C1
                search_match: Color::Rgb(163, 190, 140),  // Nord Green #A3BE8C
                selection_bg: Color::Rgb(94, 129, 172),   // Nord Deep Blue #5E81AC
                selection_fg: Color::Rgb(236, 239, 244),
                accent: Color::Rgb(180, 142, 173), // Nord Magenta #B48EAD
            },
            "dracula" => Self {
                border_active: Color::Rgb(189, 147, 249), // Dracula Purple #BD93F9
                border_inactive: Color::Rgb(98, 114, 164), // Dracula Comment #6272A4
                title: Color::Rgb(248, 248, 242),         // Dracula White #F8F8F2
                search_fg: Color::Rgb(255, 121, 198),     // Dracula Pink #FF79C6
                search_match: Color::Rgb(80, 250, 123),   // Dracula Green #50FA7B
                selection_bg: Color::Rgb(189, 147, 249),  // Dracula Purple #BD93F9
                selection_fg: Color::Rgb(40, 42, 54),     // Dracula Dark #282A36
                accent: Color::Rgb(255, 184, 108),        // Dracula Orange #FFB86C
            },
            _ => Self {
                border_active: Color::Blue,
                border_inactive: Color::DarkGray,
                title: Color::White,
                search_fg: Color::Yellow,
                search_match: Color::Green,
                selection_bg: Color::Blue,
                selection_fg: Color::White,
                accent: Color::Yellow,
            },
        }
    }
}
