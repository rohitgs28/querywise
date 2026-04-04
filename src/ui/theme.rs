// src/ui/theme.rs
//
// Configurable color themes for QueryWise.
//
// Supports built-in themes (dark, light, solarized, monokai, dracula, nord)
// and custom themes via config.toml. This addresses GitHub issue #4.

use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;

/// A complete color theme for the TUI.
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,

    // Base colors
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub border_focused: Color,
    pub title: Color,

    // Panel colors
    pub panel_bg: Color,
    pub panel_header_bg: Color,
    pub panel_header_fg: Color,

    // SQL highlighting
    pub sql_keyword: Color,
    pub sql_function: Color,
    pub sql_string: Color,
    pub sql_number: Color,
    pub sql_operator: Color,
    pub sql_comment: Color,
    pub sql_type: Color,
    pub sql_table: Color,

    // Status and feedback
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,

    // Results table
    pub table_header_bg: Color,
    pub table_header_fg: Color,
    pub table_row_alt: Color,
    pub table_selected: Color,

    // AI chat
    pub chat_user: Color,
    pub chat_ai: Color,
    pub chat_system: Color,

    // Input
    pub input_bg: Color,
    pub input_fg: Color,
    pub input_cursor: Color,
    pub input_placeholder: Color,

    // Schema browser
    pub schema_table: Color,
    pub schema_column: Color,
    pub schema_type: Color,

    // Status bar
    pub statusbar_bg: Color,
    pub statusbar_fg: Color,
    pub statusbar_mode: Color,
}

/// Theme configuration from config.toml.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ThemeConfig {
    /// Name of built-in theme or "custom"
    pub name: Option<String>,
    /// Custom color overrides (hex strings like "#ff5555")
    pub colors: Option<CustomColors>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CustomColors {
    pub bg: Option<String>,
    pub fg: Option<String>,
    pub border: Option<String>,
    pub keyword: Option<String>,
    pub string: Option<String>,
    pub number: Option<String>,
    pub success: Option<String>,
    pub error: Option<String>,
    pub warning: Option<String>,
}

impl Theme {
    /// Load a theme by name.
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            "solarized" | "solarized-dark" => Self::solarized_dark(),
            "solarized-light" => Self::solarized_light(),
            "monokai" => Self::monokai(),
            "dracula" => Self::dracula(),
            "nord" => Self::nord(),
            _ => Self::dark(), // default
        }
    }

    /// Load from config, with fallback to dark theme.
    pub fn from_config(config: &ThemeConfig) -> Self {
        let mut theme = match config.name.as_deref() {
            Some(name) => Self::from_name(name),
            None => Self::dark(),
        };

        // Apply custom color overrides
        if let Some(ref colors) = config.colors {
            if let Some(ref c) = colors.bg { theme.bg = parse_hex(c); }
            if let Some(ref c) = colors.fg { theme.fg = parse_hex(c); }
            if let Some(ref c) = colors.border { theme.border = parse_hex(c); }
            if let Some(ref c) = colors.keyword { theme.sql_keyword = parse_hex(c); }
            if let Some(ref c) = colors.string { theme.sql_string = parse_hex(c); }
            if let Some(ref c) = colors.number { theme.sql_number = parse_hex(c); }
            if let Some(ref c) = colors.success { theme.success = parse_hex(c); }
            if let Some(ref c) = colors.error { theme.error = parse_hex(c); }
            if let Some(ref c) = colors.warning { theme.warning = parse_hex(c); }
        }

        theme
    }

    /// Default dark theme.
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            border: Color::Rgb(88, 91, 112),
            border_focused: Color::Rgb(137, 180, 250),
            title: Color::Rgb(137, 180, 250),
            panel_bg: Color::Rgb(30, 30, 46),
            panel_header_bg: Color::Rgb(49, 50, 68),
            panel_header_fg: Color::Rgb(205, 214, 244),
            sql_keyword: Color::Rgb(203, 166, 247),   // purple
            sql_function: Color::Rgb(137, 180, 250),   // blue
            sql_string: Color::Rgb(166, 227, 161),     // green
            sql_number: Color::Rgb(250, 179, 135),     // peach
            sql_operator: Color::Rgb(148, 226, 213),   // teal
            sql_comment: Color::Rgb(108, 112, 134),    // overlay0
            sql_type: Color::Rgb(249, 226, 175),       // yellow
            sql_table: Color::Rgb(245, 194, 231),      // pink
            success: Color::Rgb(166, 227, 161),
            error: Color::Rgb(243, 139, 168),
            warning: Color::Rgb(249, 226, 175),
            info: Color::Rgb(137, 180, 250),
            table_header_bg: Color::Rgb(49, 50, 68),
            table_header_fg: Color::Rgb(205, 214, 244),
            table_row_alt: Color::Rgb(36, 39, 58),
            table_selected: Color::Rgb(69, 71, 90),
            chat_user: Color::Rgb(137, 180, 250),
            chat_ai: Color::Rgb(166, 227, 161),
            chat_system: Color::Rgb(108, 112, 134),
            input_bg: Color::Rgb(36, 39, 58),
            input_fg: Color::Rgb(205, 214, 244),
            input_cursor: Color::Rgb(245, 194, 231),
            input_placeholder: Color::Rgb(108, 112, 134),
            schema_table: Color::Rgb(249, 226, 175),
            schema_column: Color::Rgb(205, 214, 244),
            schema_type: Color::Rgb(108, 112, 134),
            statusbar_bg: Color::Rgb(49, 50, 68),
            statusbar_fg: Color::Rgb(205, 214, 244),
            statusbar_mode: Color::Rgb(166, 227, 161),
        }
    }

    /// Light theme for daytime use.
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            bg: Color::Rgb(239, 241, 245),
            fg: Color::Rgb(76, 79, 105),
            border: Color::Rgb(172, 176, 190),
            border_focused: Color::Rgb(30, 102, 245),
            title: Color::Rgb(30, 102, 245),
            panel_bg: Color::Rgb(239, 241, 245),
            panel_header_bg: Color::Rgb(220, 224, 232),
            panel_header_fg: Color::Rgb(76, 79, 105),
            sql_keyword: Color::Rgb(136, 57, 239),
            sql_function: Color::Rgb(30, 102, 245),
            sql_string: Color::Rgb(64, 160, 43),
            sql_number: Color::Rgb(254, 100, 11),
            sql_operator: Color::Rgb(23, 146, 153),
            sql_comment: Color::Rgb(140, 143, 161),
            sql_type: Color::Rgb(223, 142, 29),
            sql_table: Color::Rgb(234, 118, 203),
            success: Color::Rgb(64, 160, 43),
            error: Color::Rgb(210, 15, 57),
            warning: Color::Rgb(223, 142, 29),
            info: Color::Rgb(30, 102, 245),
            table_header_bg: Color::Rgb(220, 224, 232),
            table_header_fg: Color::Rgb(76, 79, 105),
            table_row_alt: Color::Rgb(230, 233, 239),
            table_selected: Color::Rgb(188, 192, 204),
            chat_user: Color::Rgb(30, 102, 245),
            chat_ai: Color::Rgb(64, 160, 43),
            chat_system: Color::Rgb(140, 143, 161),
            input_bg: Color::Rgb(230, 233, 239),
            input_fg: Color::Rgb(76, 79, 105),
            input_cursor: Color::Rgb(234, 118, 203),
            input_placeholder: Color::Rgb(140, 143, 161),
            schema_table: Color::Rgb(223, 142, 29),
            schema_column: Color::Rgb(76, 79, 105),
            schema_type: Color::Rgb(140, 143, 161),
            statusbar_bg: Color::Rgb(220, 224, 232),
            statusbar_fg: Color::Rgb(76, 79, 105),
            statusbar_mode: Color::Rgb(64, 160, 43),
        }
    }

    /// Dracula theme — popular with developers.
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            border: Color::Rgb(68, 71, 90),
            border_focused: Color::Rgb(189, 147, 249),
            title: Color::Rgb(189, 147, 249),
            panel_bg: Color::Rgb(40, 42, 54),
            panel_header_bg: Color::Rgb(68, 71, 90),
            panel_header_fg: Color::Rgb(248, 248, 242),
            sql_keyword: Color::Rgb(255, 121, 198),    // pink
            sql_function: Color::Rgb(139, 233, 253),   // cyan
            sql_string: Color::Rgb(241, 250, 140),     // yellow
            sql_number: Color::Rgb(189, 147, 249),     // purple
            sql_operator: Color::Rgb(255, 184, 108),   // orange
            sql_comment: Color::Rgb(98, 114, 164),     // comment
            sql_type: Color::Rgb(139, 233, 253),       // cyan
            sql_table: Color::Rgb(80, 250, 123),       // green
            success: Color::Rgb(80, 250, 123),
            error: Color::Rgb(255, 85, 85),
            warning: Color::Rgb(255, 184, 108),
            info: Color::Rgb(139, 233, 253),
            table_header_bg: Color::Rgb(68, 71, 90),
            table_header_fg: Color::Rgb(248, 248, 242),
            table_row_alt: Color::Rgb(49, 51, 65),
            table_selected: Color::Rgb(68, 71, 90),
            chat_user: Color::Rgb(139, 233, 253),
            chat_ai: Color::Rgb(80, 250, 123),
            chat_system: Color::Rgb(98, 114, 164),
            input_bg: Color::Rgb(68, 71, 90),
            input_fg: Color::Rgb(248, 248, 242),
            input_cursor: Color::Rgb(248, 248, 242),
            input_placeholder: Color::Rgb(98, 114, 164),
            schema_table: Color::Rgb(241, 250, 140),
            schema_column: Color::Rgb(248, 248, 242),
            schema_type: Color::Rgb(98, 114, 164),
            statusbar_bg: Color::Rgb(68, 71, 90),
            statusbar_fg: Color::Rgb(248, 248, 242),
            statusbar_mode: Color::Rgb(80, 250, 123),
        }
    }

    /// Nord theme — calm, arctic-inspired.
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(216, 222, 233),
            border: Color::Rgb(76, 86, 106),
            border_focused: Color::Rgb(136, 192, 208),
            title: Color::Rgb(136, 192, 208),
            panel_bg: Color::Rgb(46, 52, 64),
            panel_header_bg: Color::Rgb(59, 66, 82),
            panel_header_fg: Color::Rgb(216, 222, 233),
            sql_keyword: Color::Rgb(180, 142, 173),
            sql_function: Color::Rgb(129, 161, 193),
            sql_string: Color::Rgb(163, 190, 140),
            sql_number: Color::Rgb(208, 135, 112),
            sql_operator: Color::Rgb(136, 192, 208),
            sql_comment: Color::Rgb(76, 86, 106),
            sql_type: Color::Rgb(235, 203, 139),
            sql_table: Color::Rgb(143, 188, 187),
            success: Color::Rgb(163, 190, 140),
            error: Color::Rgb(191, 97, 106),
            warning: Color::Rgb(235, 203, 139),
            info: Color::Rgb(136, 192, 208),
            table_header_bg: Color::Rgb(59, 66, 82),
            table_header_fg: Color::Rgb(216, 222, 233),
            table_row_alt: Color::Rgb(52, 60, 72),
            table_selected: Color::Rgb(67, 76, 94),
            chat_user: Color::Rgb(136, 192, 208),
            chat_ai: Color::Rgb(163, 190, 140),
            chat_system: Color::Rgb(76, 86, 106),
            input_bg: Color::Rgb(59, 66, 82),
            input_fg: Color::Rgb(216, 222, 233),
            input_cursor: Color::Rgb(236, 239, 244),
            input_placeholder: Color::Rgb(76, 86, 106),
            schema_table: Color::Rgb(235, 203, 139),
            schema_column: Color::Rgb(216, 222, 233),
            schema_type: Color::Rgb(76, 86, 106),
            statusbar_bg: Color::Rgb(59, 66, 82),
            statusbar_fg: Color::Rgb(216, 222, 233),
            statusbar_mode: Color::Rgb(163, 190, 140),
        }
    }

    /// Solarized Dark.
    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark".to_string(),
            bg: Color::Rgb(0, 43, 54),
            fg: Color::Rgb(131, 148, 150),
            border: Color::Rgb(88, 110, 117),
            border_focused: Color::Rgb(38, 139, 210),
            title: Color::Rgb(38, 139, 210),
            panel_bg: Color::Rgb(0, 43, 54),
            panel_header_bg: Color::Rgb(7, 54, 66),
            panel_header_fg: Color::Rgb(147, 161, 161),
            sql_keyword: Color::Rgb(133, 153, 0),
            sql_function: Color::Rgb(38, 139, 210),
            sql_string: Color::Rgb(42, 161, 152),
            sql_number: Color::Rgb(211, 54, 130),
            sql_operator: Color::Rgb(203, 75, 22),
            sql_comment: Color::Rgb(88, 110, 117),
            sql_type: Color::Rgb(181, 137, 0),
            sql_table: Color::Rgb(108, 113, 196),
            success: Color::Rgb(133, 153, 0),
            error: Color::Rgb(220, 50, 47),
            warning: Color::Rgb(181, 137, 0),
            info: Color::Rgb(38, 139, 210),
            table_header_bg: Color::Rgb(7, 54, 66),
            table_header_fg: Color::Rgb(147, 161, 161),
            table_row_alt: Color::Rgb(7, 54, 66),
            table_selected: Color::Rgb(88, 110, 117),
            chat_user: Color::Rgb(38, 139, 210),
            chat_ai: Color::Rgb(133, 153, 0),
            chat_system: Color::Rgb(88, 110, 117),
            input_bg: Color::Rgb(7, 54, 66),
            input_fg: Color::Rgb(147, 161, 161),
            input_cursor: Color::Rgb(238, 232, 213),
            input_placeholder: Color::Rgb(88, 110, 117),
            schema_table: Color::Rgb(181, 137, 0),
            schema_column: Color::Rgb(131, 148, 150),
            schema_type: Color::Rgb(88, 110, 117),
            statusbar_bg: Color::Rgb(7, 54, 66),
            statusbar_fg: Color::Rgb(147, 161, 161),
            statusbar_mode: Color::Rgb(133, 153, 0),
        }
    }

    /// Solarized Light.
    pub fn solarized_light() -> Self {
        let mut theme = Self::solarized_dark();
        theme.name = "solarized-light".to_string();
        theme.bg = Color::Rgb(253, 246, 227);
        theme.fg = Color::Rgb(101, 123, 131);
        theme.panel_bg = Color::Rgb(253, 246, 227);
        theme.panel_header_bg = Color::Rgb(238, 232, 213);
        theme.input_bg = Color::Rgb(238, 232, 213);
        theme.table_row_alt = Color::Rgb(238, 232, 213);
        theme
    }

    /// Monokai — classic editor theme.
    pub fn monokai() -> Self {
        Self {
            name: "monokai".to_string(),
            bg: Color::Rgb(39, 40, 34),
            fg: Color::Rgb(248, 248, 242),
            border: Color::Rgb(117, 113, 94),
            border_focused: Color::Rgb(166, 226, 46),
            title: Color::Rgb(166, 226, 46),
            panel_bg: Color::Rgb(39, 40, 34),
            panel_header_bg: Color::Rgb(52, 53, 47),
            panel_header_fg: Color::Rgb(248, 248, 242),
            sql_keyword: Color::Rgb(249, 38, 114),
            sql_function: Color::Rgb(102, 217, 239),
            sql_string: Color::Rgb(230, 219, 116),
            sql_number: Color::Rgb(174, 129, 255),
            sql_operator: Color::Rgb(249, 38, 114),
            sql_comment: Color::Rgb(117, 113, 94),
            sql_type: Color::Rgb(102, 217, 239),
            sql_table: Color::Rgb(166, 226, 46),
            success: Color::Rgb(166, 226, 46),
            error: Color::Rgb(249, 38, 114),
            warning: Color::Rgb(230, 219, 116),
            info: Color::Rgb(102, 217, 239),
            table_header_bg: Color::Rgb(52, 53, 47),
            table_header_fg: Color::Rgb(248, 248, 242),
            table_row_alt: Color::Rgb(45, 46, 40),
            table_selected: Color::Rgb(73, 72, 62),
            chat_user: Color::Rgb(102, 217, 239),
            chat_ai: Color::Rgb(166, 226, 46),
            chat_system: Color::Rgb(117, 113, 94),
            input_bg: Color::Rgb(52, 53, 47),
            input_fg: Color::Rgb(248, 248, 242),
            input_cursor: Color::Rgb(248, 248, 242),
            input_placeholder: Color::Rgb(117, 113, 94),
            schema_table: Color::Rgb(230, 219, 116),
            schema_column: Color::Rgb(248, 248, 242),
            schema_type: Color::Rgb(117, 113, 94),
            statusbar_bg: Color::Rgb(52, 53, 47),
            statusbar_fg: Color::Rgb(248, 248, 242),
            statusbar_mode: Color::Rgb(166, 226, 46),
        }
    }

    // --- Style helpers ---

    pub fn keyword_style(&self) -> Style {
        Style::default().fg(self.sql_keyword).add_modifier(Modifier::BOLD)
    }

    pub fn function_style(&self) -> Style {
        Style::default().fg(self.sql_function)
    }

    pub fn string_style(&self) -> Style {
        Style::default().fg(self.sql_string)
    }

    pub fn number_style(&self) -> Style {
        Style::default().fg(self.sql_number)
    }

    pub fn comment_style(&self) -> Style {
        Style::default().fg(self.sql_comment).add_modifier(Modifier::ITALIC)
    }

    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error).add_modifier(Modifier::BOLD)
    }

    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    pub fn border_style(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.border_focused)
        } else {
            Style::default().fg(self.border)
        }
    }

    /// Returns the list of available theme names.
    pub fn available_themes() -> Vec<&'static str> {
        vec!["dark", "light", "dracula", "nord", "solarized-dark", "solarized-light", "monokai"]
    }
}

/// Parse a hex color string like "#ff5555" or "ff5555" into a Color.
fn parse_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return Color::Reset;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    Color::Rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_with_hash() {
        assert_eq!(parse_hex("#ff5555"), Color::Rgb(255, 85, 85));
    }

    #[test]
    fn test_parse_hex_without_hash() {
        assert_eq!(parse_hex("50fa7b"), Color::Rgb(80, 250, 123));
    }

    #[test]
    fn test_parse_hex_invalid() {
        assert_eq!(parse_hex("invalid"), Color::Reset);
    }

    #[test]
    fn test_theme_from_name() {
        let theme = Theme::from_name("dracula");
        assert_eq!(theme.name, "dracula");
    }

    #[test]
    fn test_theme_from_name_default() {
        let theme = Theme::from_name("nonexistent");
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available_themes();
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"dracula"));
        assert!(themes.contains(&"nord"));
    }

    #[test]
    fn test_config_override() {
        let config = ThemeConfig {
            name: Some("dark".to_string()),
            colors: Some(CustomColors {
                success: Some("#00ff00".to_string()),
                ..Default::default()
            }),
        };
        let theme = Theme::from_config(&config);
        assert_eq!(theme.success, Color::Rgb(0, 255, 0));
        // Other colors should remain from the dark theme
        assert_eq!(theme.name, "dark");
    }
}
