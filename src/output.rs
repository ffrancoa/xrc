use anstyle::{AnsiColor, Color, Style};

pub const RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
pub const GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
pub const YELLOW: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
pub const CYAN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan)));
pub const BOLD_UNDERLINE: Style = Style::new().bold().underline();

pub fn paint(style: Style, text: &str) -> String {
    format!("{}{}{}", style.render(), text, style.render_reset())
}
