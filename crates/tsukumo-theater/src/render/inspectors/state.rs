//! Durable state list, details, and source-event pagination.

use super::super::text::{compact_identifier, display_width, truncate_width};
use super::super::theme::Theme;
use super::shared::{detail_line, inspector_block, state_status_label};
use crate::app::{ProductView, StateView};
use crate::pack::ValidatedPresentationPack;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) fn render_state_inspector(
    pack: &ValidatedPresentationPack,
    view: &ProductView,
    selected: usize,
    evidence_page: usize,
    area: Rect,
    theme: Theme,
    buffer: &mut Buffer,
) {
    let block = inspector_block(&pack.manifest().terminology.memory_cabinet, theme);
    let inner = block.inner(area);
    block.render(area, buffer);
    if view.states.is_empty() {
        Paragraph::new("暂无可检查的持久状态。")
            .style(Style::default().fg(theme.muted))
            .render(inner, buffer);
        return;
    }

    let list_cap = usize::from(inner.height).saturating_sub(7).max(1);
    let selected = selected.min(view.states.len().saturating_sub(1));
    let window_start = selected.saturating_add(1).saturating_sub(list_cap);
    let mut lines = view
        .states
        .iter()
        .enumerate()
        .skip(window_start)
        .take(list_cap)
        .map(|(index, state)| state_line(index, selected, state, inner.width, theme))
        .collect::<Vec<_>>();

    if let Some(state) = view.states.get(selected) {
        lines.push(Line::from(""));
        lines.push(detail_line(
            "范围",
            state.scope.as_str(),
            inner.width,
            theme,
        ));
        lines.push(detail_line(
            "强度",
            &format!(
                "{} · {}",
                state.strength.as_str(),
                state_status_label(state.status)
            ),
            inner.width,
            theme,
        ));
        push_evidence_page(&mut lines, state, evidence_page, inner.width, theme);
    }
    lines.truncate(usize::from(inner.height));
    Paragraph::new(lines).render(inner, buffer);
}

fn state_line(
    index: usize,
    selected: usize,
    state: &StateView,
    width: u16,
    theme: Theme,
) -> Line<'static> {
    let marker = if index == selected { ">" } else { " " };
    let prefix = format!("{marker} [{}] ", state_status_label(state.status));
    let maximum = usize::from(width);
    let available = maximum.saturating_sub(display_width(&prefix));
    // Reserve one third for the value while keeping the complete compact ID before it.
    let value_reserve = if available >= 24 { available / 3 } else { 0 };
    let id_budget = available
        .saturating_sub(value_reserve)
        .saturating_sub(usize::from(value_reserve > 0));
    let id = compact_identifier(state.id.as_str(), id_budget);
    let text = if value_reserve == 0 {
        format!("{prefix}{id}")
    } else {
        format!("{prefix}{id} {}", state.value.as_str())
    };
    let style = if index == selected {
        Style::default()
            .fg(theme.ink)
            .bg(theme.accent)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };
    Line::from(Span::styled(
        truncate_width(&text, usize::from(width)),
        style,
    ))
}

fn push_evidence_page(
    lines: &mut Vec<Line<'static>>,
    state: &StateView,
    selected_page: usize,
    width: u16,
    theme: Theme,
) {
    let page_count = state.evidence_page_count();
    let selected_page = selected_page.min(page_count.saturating_sub(1));
    let page = state.evidence_page(selected_page);
    let start = selected_page
        .saturating_mul(StateView::evidence_page_size())
        .min(state.source_events.len());
    let end = start.saturating_add(page.len());
    // User-facing evidence ranges are one-based whenever the page contains data.
    let display_start = if page.is_empty() {
        0
    } else {
        start.saturating_add(1)
    };
    let receipt = if state.source_event_total > state.source_events.len() {
        format!(
            " · 保留 {}/{}",
            state.source_events.len(),
            state.source_event_total
        )
    } else {
        String::new()
    };
    lines.push(Line::from(Span::styled(
        truncate_width(
            &format!(
                "证据 {display_start}-{end}/{} · 页 {}/{} [←→]{}",
                state.source_events.len(),
                selected_page + 1,
                page_count,
                receipt
            ),
            usize::from(width),
        ),
        Style::default().fg(theme.muted),
    )));
    if page.is_empty() {
        lines.push(Line::from(Span::styled(
            "  · 无来源事件",
            Style::default().fg(theme.muted),
        )));
        return;
    }
    for event in page {
        let event = compact_identifier(event.as_str(), usize::from(width).saturating_sub(4));
        lines.push(Line::from(Span::styled(
            format!("  · {event}"),
            Style::default().fg(theme.text),
        )));
    }
}
