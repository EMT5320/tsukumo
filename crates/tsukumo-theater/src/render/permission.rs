//! Blocking permission-contract overlay.

use super::text::{truncate_width, wrap_width};
use super::theme::Theme;
use crate::app::PermissionView;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget};

pub(super) fn render_permission(
    permission_title: &str,
    permission: &PermissionView,
    page_index: usize,
    area: Rect,
    theme: Theme,
    buffer: &mut Buffer,
) {
    // A full-screen mask prevents modal edges from bisecting an underlying CJK cell.
    Clear.render(area, buffer);
    buffer.set_style(area, Style::default().fg(theme.text).bg(theme.ink));

    let modal_width = area.width.saturating_sub(4).clamp(20, 72);
    let available_height = area.height.saturating_sub(4).max(8);
    let modal_height = 16.min(available_height).min(area.height);
    let modal = centered(
        area,
        modal_width.min(area.width),
        modal_height.min(area.height),
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.urgent))
        .title(Span::styled(
            format!(" {permission_title} · 需要明确裁定 "),
            Style::default()
                .fg(theme.urgent)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(modal);
    block.render(modal, buffer);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let choices_area = Rect::new(
        inner.x,
        inner.y + inner.height.saturating_sub(1),
        inner.width,
        1,
    );
    render_choices(choices_area, theme, buffer);

    let body = Rect::new(
        inner.x,
        inner.y,
        inner.width,
        inner.height.saturating_sub(1),
    );
    let mut lines = permission_lines(permission, page_index, usize::from(body.width), theme);
    let capacity = usize::from(body.height);
    if lines.len() > capacity && capacity > 0 {
        lines.truncate(capacity);
        lines[capacity - 1] = Line::from(Span::styled(
            "…本页内容已折叠，请扩大终端",
            Style::default().fg(theme.urgent),
        ));
    }
    Paragraph::new(lines)
        .style(Style::default().fg(theme.text).bg(theme.ink))
        .render(body, buffer);
}

fn render_choices(area: Rect, theme: Theme, buffer: &mut Buffer) {
    let choices = if area.width >= 34 {
        vec![
            ("[1]仅一次", theme.text),
            ("   ", theme.text),
            ("[2]本次会话", theme.text),
            ("   ", theme.text),
            ("[D]拒绝", theme.urgent),
        ]
    } else {
        vec![
            ("1一次", theme.text),
            (" ", theme.text),
            ("2会话", theme.text),
            (" ", theme.text),
            ("D拒绝", theme.urgent),
        ]
    };
    let spans = choices
        .into_iter()
        .map(|(text, color)| {
            Span::styled(
                text,
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )
        })
        .collect::<Vec<_>>();
    Paragraph::new(Line::from(spans)).render(area, buffer);
}

fn permission_lines(
    permission: &PermissionView,
    page_index: usize,
    width: usize,
    theme: Theme,
) -> Vec<Line<'static>> {
    let value_width = width.saturating_sub(8);
    let page_count = permission.evidence_page_count();
    let page_index = page_index.min(page_count.saturating_sub(1));
    let page = permission.evidence_page(page_index);
    let mut lines = vec![
        labeled_line("工具", permission.tool.as_str(), value_width, theme),
        labeled_line("运行时", permission.runtime.as_str(), value_width, theme),
    ];

    let item = if page.item_count > 1 {
        format!(" {}/{}", page.item_index + 1, page.item_count)
    } else {
        String::new()
    };
    lines.push(Line::from(Span::styled(
        truncate_width(
            &format!(
                "{}{item} · 分页 {}/{} · 总页 {}/{} [↑↓←→]",
                page.label,
                page.part_index + 1,
                page.part_count,
                page_index + 1,
                page_count,
            ),
            width,
        ),
        Style::default()
            .fg(theme.urgent)
            .add_modifier(Modifier::BOLD),
    )));
    let value = if page.text.is_empty() {
        "未提供"
    } else {
        page.text.as_str()
    };
    push_wrapped_limited(&mut lines, "·", value, width, 5, theme);
    lines
}

fn labeled_line(label: &str, value: &str, maximum: usize, theme: Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label}  "), Style::default().fg(theme.muted)),
        Span::styled(
            truncate_width(value, maximum),
            Style::default().fg(theme.text),
        ),
    ])
}

fn push_wrapped_limited(
    lines: &mut Vec<Line<'static>>,
    label: &str,
    value: &str,
    width: usize,
    maximum_lines: usize,
    theme: Theme,
) {
    if maximum_lines == 0 {
        return;
    }
    let label_text = format!("{label}  ");
    let label_width = super::text::display_width(&label_text);
    let value_width = width.saturating_sub(label_width).max(1);
    let parts = wrap_width(value, value_width);
    let truncated = parts.len() > maximum_lines;
    for (index, part) in parts.into_iter().take(maximum_lines).enumerate() {
        let prefix = if index == 0 {
            label_text.clone()
        } else {
            " ".repeat(label_width)
        };
        let content = if truncated && index + 1 == maximum_lines {
            truncate_width(&(part + "…"), value_width)
        } else {
            part
        };
        lines.push(Line::from(vec![
            Span::styled(prefix, Style::default().fg(theme.muted)),
            Span::styled(content, Style::default().fg(theme.text)),
        ]));
    }
}

const fn centered(area: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    )
}
