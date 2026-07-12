//! CJK-safe terminal text helpers.

use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthChar;

pub(super) fn display_width(input: &str) -> usize {
    input
        .chars()
        .map(|character| UnicodeWidthChar::width(character).unwrap_or(0))
        .sum()
}

pub(super) fn truncate_width(input: &str, maximum: usize) -> String {
    if maximum == 0 {
        return String::new();
    }
    if display_width(input) <= maximum {
        return input.to_owned();
    }

    let ellipsis = "…";
    let ellipsis_width = 1;
    if maximum <= ellipsis_width {
        return ellipsis.to_owned();
    }
    let budget = maximum - ellipsis_width;
    let mut used = 0;
    let mut output = String::new();
    for character in input.chars() {
        let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
        if used + character_width > budget {
            break;
        }
        output.push(character);
        used += character_width;
    }
    output.push_str(ellipsis);
    output
}

/// Keeps both ends of a durable identifier so common prefixes remain distinguishable.
pub(super) fn compact_identifier(input: &str, maximum: usize) -> String {
    if display_width(input) <= maximum {
        return input.to_owned();
    }
    if maximum < 8 {
        return truncate_width(input, maximum);
    }
    // A twelve-cell tail keeps common human-readable ID suffixes distinguishable.
    let suffix_budget = 12.min((maximum - 1) / 2);
    let prefix_budget = maximum.saturating_sub(suffix_budget + 1);
    let prefix = take_prefix_width(input, prefix_budget);
    let suffix = take_suffix_width(input, suffix_budget);
    format!("{prefix}…{suffix}")
}

fn take_prefix_width(input: &str, maximum: usize) -> String {
    let mut used = 0;
    input
        .chars()
        .take_while(|character| {
            let width = UnicodeWidthChar::width(*character).unwrap_or(0);
            let keep = used + width <= maximum;
            if keep {
                used += width;
            }
            keep
        })
        .collect()
}

fn take_suffix_width(input: &str, maximum: usize) -> String {
    let mut used = 0;
    let mut reversed = input
        .chars()
        .rev()
        .take_while(|character| {
            let width = UnicodeWidthChar::width(*character).unwrap_or(0);
            let keep = used + width <= maximum;
            if keep {
                used += width;
            }
            keep
        })
        .collect::<Vec<_>>();
    reversed.reverse();
    reversed.into_iter().collect()
}
/// Truncates a styled line without flattening its semantic color spans.
pub(super) fn truncate_spans(spans: Vec<Span<'static>>, maximum: usize) -> Line<'static> {
    let mut remaining = maximum;
    let mut output = Vec::new();
    for span in spans {
        if remaining == 0 {
            break;
        }
        let content = truncate_width(span.content.as_ref(), remaining);
        let width = display_width(&content);
        output.push(Span::styled(content, span.style));
        remaining = remaining.saturating_sub(width);
        if width == 0 {
            break;
        }
    }
    Line::from(output)
}

/// Wraps untrusted display text on terminal cell boundaries.
pub(super) fn wrap_width(input: &str, maximum: usize) -> Vec<String> {
    if maximum == 0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut used = 0;
    for character in input.chars() {
        if character == '\n' {
            lines.push(std::mem::take(&mut line));
            used = 0;
            continue;
        }
        let width = UnicodeWidthChar::width(character).unwrap_or(0);
        if used > 0 && used + width > maximum {
            lines.push(std::mem::take(&mut line));
            used = 0;
        }
        if width <= maximum {
            line.push(character);
            used += width;
        }
    }
    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}
