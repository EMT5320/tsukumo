//! Shared inspector chrome and typed state labels.

use super::super::text::truncate_width;
use super::super::theme::Theme;
use crate::app::StateStatus;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders};

pub(super) fn detail_line(label: &str, value: &str, width: u16, theme: Theme) -> Line<'static> {
    let maximum = usize::from(width).saturating_sub(6);
    Line::from(vec![
        Span::styled(format!("{label}  "), Style::default().fg(theme.muted)),
        Span::styled(
            truncate_width(value, maximum),
            Style::default().fg(theme.text),
        ),
    ])
}

pub(super) fn inspector_block(title: &str, theme: Theme) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ))
}

pub(super) const fn state_status_label(status: StateStatus) -> &'static str {
    match status {
        StateStatus::Active => "有效",
        StateStatus::Superseded => "已替代",
        StateStatus::Revoked => "已撤销",
        StateStatus::Expired => "已过期",
    }
}
