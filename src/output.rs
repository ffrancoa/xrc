use anstyle::{AnsiColor, Color, Style};

pub const RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
pub const GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
pub const YELLOW: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
pub const CYAN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));
pub const BOLD_UNDERLINE: Style = Style::new().bold().underline();
pub const BOLD_GREEN: Style = GREEN.bold();
pub const BOLD_RED: Style = RED.bold();
pub const BOLD_YELLOW: Style = YELLOW.bold();

pub fn paint(style: Style, text: &str) -> String {
    format!("{}{}{}", style.render(), text, style.render_reset())
}

pub fn info(message: &str) -> String {
    format!("{} {}", paint(BOLD_GREEN, "info:"), message)
}

pub fn warning(message: &str) -> String {
    format!("{} {}", paint(BOLD_YELLOW, "warning:"), message)
}

pub fn error(message: &str) -> String {
    let mut chars = message.chars();
    let lowercased = match chars.next() {
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    };
    format!("{} {}", paint(BOLD_RED, "error:"), lowercased)
}
