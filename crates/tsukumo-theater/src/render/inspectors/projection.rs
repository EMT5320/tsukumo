//! Projection receipt metadata and bounded entry pagination.

use super::super::text::{compact_identifier, display_width, truncate_spans, truncate_width};
use super::super::theme::Theme;
use super::shared::inspector_block;
use crate::app::{ProductView, ProjectionView};
use crate::pack::ValidatedPresentationPack;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

pub(crate) fn render_projection_inspector(
    pack: &ValidatedPresentationPack,
    view: &ProductView,
    selected_page: usize,
    area: Rect,
    theme: Theme,
    buffer: &mut Buffer,
) {
    let block = inspector_block(&pack.manifest().terminology.projection_desk, theme);
    let inner = block.inner(area);
    block.render(area, buffer);
    let Some(projection) = &view.projection else {
        let checkpoint = view.handoff.checkpoint_id.as_ref().map_or_else(
            || "尚无检查点或投影。".to_owned(),
            |id| {
                format!(
                    "检查点 {id} · v{}；尚无投影。",
                    view.handoff.version.unwrap_or_default()
                )
            },
        );
        Paragraph::new(checkpoint)
            .style(Style::default().fg(theme.muted))
            .render(inner, buffer);
        return;
    };

    let lines = projection_lines(projection, selected_page, inner.width, theme)
        .into_iter()
        .take(usize::from(inner.height))
        .collect::<Vec<_>>();
    Paragraph::new(lines).render(inner, buffer);
}

fn projection_lines(
    projection: &ProjectionView,
    selected_page: usize,
    width: u16,
    theme: Theme,
) -> Vec<Line<'static>> {
    let maximum = usize::from(width);
    let page_count = projection.entry_page_count();
    let selected_page = selected_page.min(page_count.saturating_sub(1));
    let bounds = projection.entry_page_bounds(selected_page);
    let mut lines = vec![
        id_line(
            "投影",
            projection.projection_id.as_str(),
            maximum,
            theme.accent,
            theme,
        ),
        id_line(
            "检查点",
            projection.checkpoint_id.as_str(),
            maximum,
            theme.text,
            theme,
        ),
        Line::from(Span::styled(
            truncate_width(
                &format!(
                    "版本 投影v{} · 渲染v{} · 检查点v{} · 预算 {}/{}",
                    projection.projection_version,
                    projection.renderer_version,
                    projection
                        .checkpoint_version
                        .map_or_else(|| "?".to_owned(), |value| value.to_string()),
                    projection.budget_used,
                    projection.budget_limit,
                ),
                maximum,
            ),
            Style::default().fg(theme.text),
        )),
        Line::from(Span::styled(
            format!(
                "选中 {} · 省略 {}",
                projection.selected_total, projection.omissions_total
            ),
            Style::default().fg(theme.text),
        )),
    ];
    let item_start = if bounds.is_empty() {
        0
    } else {
        bounds.start + 1
    };
    lines.push(Line::from(Span::styled(
        format!(
            "条目 {item_start}-{}/{} · 页 {}/{} [←→]",
            bounds.end,
            projection.retained_entry_count(),
            selected_page + 1,
            page_count
        ),
        Style::default().fg(theme.muted),
    )));
    push_truncation_receipt(&mut lines, projection, maximum, theme);
    push_entries(&mut lines, projection, bounds, maximum, theme);
    lines
}

fn push_truncation_receipt(
    lines: &mut Vec<Line<'static>>,
    projection: &ProjectionView,
    width: usize,
    theme: Theme,
) {
    if projection.selected_total <= projection.selected_refs.len()
        && projection.omissions_total <= projection.omissions.len()
    {
        return;
    }
    lines.push(Line::from(Span::styled(
        truncate_width(
            &format!(
                "回执 保留选中 {}/{} · 省略 {}/{}",
                projection.selected_refs.len(),
                projection.selected_total,
                projection.omissions.len(),
                projection.omissions_total
            ),
            width,
        ),
        Style::default().fg(theme.urgent),
    )));
}

fn push_entries(
    lines: &mut Vec<Line<'static>>,
    projection: &ProjectionView,
    bounds: std::ops::Range<usize>,
    width: usize,
    theme: Theme,
) {
    for index in bounds {
        let line = if let Some(state) = projection.selected_refs.get(index) {
            let id = compact_identifier(state.state_id.as_str(), width.saturating_sub(16));
            Line::from(Span::styled(
                format!("  + 状态 {id}@v{}", state.version),
                Style::default().fg(theme.text),
            ))
        } else {
            let omission_index = index.saturating_sub(projection.selected_refs.len());
            let text = projection
                .omissions
                .get(omission_index)
                .map_or("省略记录缺失", |item| item.as_str());
            Line::from(Span::styled(
                format!("  - 省略 {text}"),
                Style::default().fg(theme.muted),
            ))
        };
        lines.push(truncate_spans(line.spans, width));
    }
}

fn id_line(
    label: &str,
    value: &str,
    width: usize,
    value_color: Color,
    theme: Theme,
) -> Line<'static> {
    let id = compact_identifier(value, width.saturating_sub(display_width(label) + 1));
    Line::from(vec![
        Span::styled(format!("{label} "), Style::default().fg(theme.muted)),
        Span::styled(id, Style::default().fg(value_color)),
    ])
}
