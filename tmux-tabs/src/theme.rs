//! Reads the live tmux colour theme so the sidebar matches the user's scheme.

use ratatui::style::Color;
use std::collections::HashMap;

use crate::tmux::Tmux;

#[derive(Debug, Clone, Default)]
pub struct Style {
    pub bg: Option<String>,
    pub fg: Option<String>,
    pub bold: bool,
    pub reverse: bool,
}

fn parse_style(raw: &str) -> Style {
    let mut s = Style::default();
    for part in raw.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (key, _, val) = {
            match part.find('=') {
                Some(i) => (&part[..i], '=', &part[i + 1..]),
                None => (part, ' ', ""),
            }
        };
        match key {
            "bg" => s.bg = Some(val.to_string()),
            "fg" => s.fg = Some(val.to_string()),
            "bold" => s.bold = true,
            "reverse" => s.reverse = true,
            _ => {}
        }
    }
    s
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub active_bg: Color,
    pub active_fg: Color,
    pub active_bold: bool,
    pub bell_bg: Color,
    pub bell_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            bg: parse_color("#94cdba"),
            fg: Color::Black,
            active_bg: parse_color("#292e42"),
            active_fg: parse_color("#e0af68"),
            active_bold: true,
            bell_bg: Color::Red,
            bell_fg: Color::White,
        }
    }
}

/// Map a tmux colour token to a crossterm `Color`.
pub fn parse_color(token: &str) -> Color {
    let t = token.trim();
    if let Some(hex) = t.strip_prefix('#') {
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
    }
    if let Some(n) = t.strip_prefix("colour").or_else(|| t.strip_prefix("color")) {
        if let Ok(v) = n.parse::<u8>() {
            return Color::Indexed(v);
        }
    }
    match t {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "brightblack" => Color::DarkGray,
        "brightred" => Color::LightRed,
        "brightgreen" => Color::LightGreen,
        "brightyellow" => Color::LightYellow,
        "brightblue" => Color::LightBlue,
        "brightmagenta" => Color::LightMagenta,
        "brightcyan" => Color::LightCyan,
        "brightwhite" => Color::Gray,
        _ => Color::Reset,
    }
}

fn resolve(style: &Style, base: &Style, def_bg: Color, def_fg: Color) -> (Color, Color) {
    let mut bg = style
        .bg
        .clone()
        .or_else(|| base.bg.clone())
        .map(|s| parse_color(&s))
        .unwrap_or(def_bg);
    let mut fg = style
        .fg
        .clone()
        .or_else(|| base.fg.clone())
        .map(|s| parse_color(&s))
        .unwrap_or(def_fg);
    if style.reverse {
        std::mem::swap(&mut bg, &mut fg);
    }
    (bg, fg)
}

/// Read styles from tmux; falls back to defaults on any failure.
pub fn load_theme<T: Tmux>(t: &T) -> Theme {
    let mut styles: HashMap<&str, Style> = HashMap::new();
    for (name, key) in [
        ("normal", "window-status-style"),
        ("current", "window-status-current-style"),
        ("bell", "window-status-bell-style"),
        ("status", "status-style"),
    ] {
        let raw = t.run(&["show-options", "-gv", key]).unwrap_or_default();
        styles.insert(name, parse_style(&raw));
    }
    let def = Theme::default();
    let base = styles.get("status").cloned().unwrap_or_default();
    let normal = styles.get("normal").cloned().unwrap_or_default();

    let (bg, fg) = resolve(&normal, &base, def.bg, def.fg);

    let mut current = styles.get("current").cloned().unwrap_or_default();
    let (active_bg, active_fg) = if current.bg.is_none() && current.fg.is_none() {
        current.bold = true;
        (def.active_bg, def.active_fg)
    } else {
        resolve(&current, &base, def.active_bg, def.active_fg)
    };

    let bell = styles.get("bell").cloned().unwrap_or_default();
    let (bell_bg, bell_fg) = resolve(&bell, &base, def.bell_bg, def.bell_fg);

    Theme {
        bg,
        fg,
        active_bg,
        active_fg,
        active_bold: current.bold,
        bell_bg,
        bell_fg,
    }
}
