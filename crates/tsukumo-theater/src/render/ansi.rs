//! Deterministic ANSI serialization for visual evidence and remote replay.

use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier};
use unicode_width::UnicodeWidthStr;

/// Serializes a rendered buffer with explicit SGR colors and CJK cell widths.
pub fn buffer_to_ansi(buffer: &Buffer) -> String {
    let area = buffer.area();
    let mut output = String::new();
    let mut active = None;
    for y in 0..area.height {
        let mut x = 0;
        while x < area.width {
            let Some(cell) = buffer.cell((x, y)) else {
                break;
            };
            let style = (cell.fg, cell.bg, cell.modifier);
            if active != Some(style) {
                write_style(&mut output, style.0, style.1, style.2);
                active = Some(style);
            }
            let symbol = cell.symbol();
            output.push_str(symbol);
            let width = UnicodeWidthStr::width(symbol).max(1);
            let Ok(cell_width) = u16::try_from(width) else {
                break;
            };
            x = x.saturating_add(cell_width);
        }
        output.push_str("\u{1b}[0m");
        active = None;
        if y + 1 < area.height {
            output.push_str("\r\n");
        }
    }
    output
}

fn write_style(output: &mut String, foreground: Color, background: Color, modifier: Modifier) {
    output.push_str("\u{1b}[0m");
    if modifier.contains(Modifier::BOLD) {
        output.push_str("\u{1b}[1m");
    }
    write_color(output, foreground, true);
    write_color(output, background, false);
}

fn write_color(output: &mut String, color: Color, foreground: bool) {
    let base = if foreground { 30 } else { 40 };
    let bright = if foreground { 90 } else { 100 };
    match color {
        Color::Reset => output.push_str(if foreground {
            "\u{1b}[39m"
        } else {
            "\u{1b}[49m"
        }),
        Color::Black => write_standard(output, base),
        Color::Red => write_standard(output, base + 1),
        Color::Green => write_standard(output, base + 2),
        Color::Yellow => write_standard(output, base + 3),
        Color::Blue => write_standard(output, base + 4),
        Color::Magenta => write_standard(output, base + 5),
        Color::Cyan => write_standard(output, base + 6),
        Color::Gray => write_standard(output, base + 7),
        Color::DarkGray => write_standard(output, bright),
        Color::LightRed => write_standard(output, bright + 1),
        Color::LightGreen => write_standard(output, bright + 2),
        Color::LightYellow => write_standard(output, bright + 3),
        Color::LightBlue => write_standard(output, bright + 4),
        Color::LightMagenta => write_standard(output, bright + 5),
        Color::LightCyan => write_standard(output, bright + 6),
        Color::White => write_standard(output, bright + 7),
        Color::Indexed(index) => {
            let channel = if foreground { 38 } else { 48 };
            output.push_str(&format!("\u{1b}[{channel};5;{index}m"));
        }
        Color::Rgb(red, green, blue) => {
            let channel = if foreground { 38 } else { 48 };
            output.push_str(&format!("\u{1b}[{channel};2;{red};{green};{blue}m"));
        }
    }
}

fn write_standard(output: &mut String, code: u8) {
    output.push_str(&format!("\u{1b}[{code}m"));
}
