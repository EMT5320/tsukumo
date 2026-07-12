//! Product chrome, factual log, footer, and undersized fallback.

use super::labels::{attention_label, execution_phase_label, runtime_health_label};
use super::text::{compact_identifier, display_width, truncate_width};
use super::theme::Theme;
use crate::app::{AppState, NoticeLevel, ProductView, Screen};
use crate::pack::ValidatedPresentationPack;
use crate::world::StageWorld;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

pub(super) fn render_header(
    pack: &ValidatedPresentationPack,
    world: &StageWorld,
    view: &ProductView,
    area: Rect,
    theme: Theme,
    buffer: &mut Buffer,
) {
    let source = view
        .runtime
        .source_spirit_id
        .as_ref()
        .map_or("未绑定", |id| id.as_str());
    let runtime = if view.runtime.detail.as_str().is_empty() {
        runtime_health_label(view.runtime.health)
    } else {
        view.runtime.detail.as_str()
    };
    let source = compact_identifier(source, 20);
    let protected = format!(
        "  │  执行者 {source}  │  {} ",
        attention_label(world.attention)
    );
    let presentation = format!(
        " {}  {} / {}  │  {}",
        pack.manifest().world.name,
        pack.companion().display_name,
        pack.companion().romanized_name,
        runtime,
    );
    let width = usize::from(area.width);
    let line = if display_width(&protected) >= width {
        truncate_width(&protected, width)
    } else {
        let presentation = truncate_width(&presentation, width - display_width(&protected));
        format!("{presentation}{protected}")
    };
    Paragraph::new(line)
        .style(
            Style::default()
                .fg(theme.text)
                .bg(theme.surface)
                .add_modifier(Modifier::BOLD),
        )
        .render(area, buffer);
}

pub(super) fn render_log(
    world: &StageWorld,
    view: &ProductView,
    area: Rect,
    theme: Theme,
    buffer: &mut Buffer,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(" 事实日志 ", Style::default().fg(theme.muted)));
    let inner = block.inner(area);
    block.render(area, buffer);
    let notice_rows = usize::from(!view.notices.is_empty());
    let log_capacity = usize::from(inner.height).saturating_sub(notice_rows);
    let mut entries = world
        .log
        .iter()
        .rev()
        .take(log_capacity)
        .collect::<Vec<_>>();
    entries.reverse();
    let mut lines = entries
        .into_iter()
        .map(|entry| {
            let source = compact_identifier(entry.attribution.source_spirit_id.as_str(), 20);
            let text = format!("[{source}] {}", entry.text);
            Line::from(Span::styled(
                truncate_width(&text, usize::from(inner.width)),
                Style::default().fg(theme.muted),
            ))
        })
        .collect::<Vec<_>>();
    if lines.is_empty() && log_capacity > 0 {
        let summary = if view.execution.summary.as_str().is_empty() {
            "等待已归一化的运行时事件。"
        } else {
            view.execution.summary.as_str()
        };
        lines.push(Line::from(Span::styled(
            truncate_width(summary, usize::from(inner.width)),
            Style::default().fg(theme.muted),
        )));
    }
    if let Some(notice) = view.notices.last() {
        let color = match notice.level {
            NoticeLevel::Info => theme.accent,
            NoticeLevel::Warning | NoticeLevel::Error => theme.urgent,
        };
        lines.push(Line::from(Span::styled(
            truncate_width(
                &format!("[回执] {}", notice.text.as_str()),
                usize::from(inner.width),
            ),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        )));
    }
    Paragraph::new(lines).render(inner, buffer);
}

pub(super) fn render_footer(
    pack: &ValidatedPresentationPack,
    view: &ProductView,
    app: &AppState,
    area: Rect,
    theme: Theme,
    buffer: &mut Buffer,
) {
    let execution_id = view
        .execution
        .execution_id
        .as_ref()
        .map(|id| truncate_width(id.as_str(), 16));
    let execution = execution_id.map_or_else(
        || format!("执行 {}", execution_phase_label(view.execution.phase)),
        |id| format!("执行 {}({id})", execution_phase_label(view.execution.phase)),
    );
    let handoff = format!(
        "{execution} · 状态 {} · 投影 {} · 预算 {}/{}",
        view.states.len(),
        if view.projection.is_some() {
            "就绪"
        } else {
            "空"
        },
        view.handoff.budget_used,
        view.handoff.budget_limit
    );
    let workshop = truncate_width(&pack.manifest().terminology.workshop, 10);
    let keys = match app.screen() {
        Screen::Workshop => format!("[S]状态  [P]投影  [R]刷新  [Q]退出  [W]{workshop}"),
        Screen::StateInspector { .. } => {
            format!("[↑↓]选择  [←→]证据  [X]撤销  [R]刷新  [Q]退出  [W]{workshop}")
        }
        Screen::ProjectionInspector => {
            format!("[↑↓←→]翻页  [S]状态  [R]刷新  [Q]退出  [W]{workshop}")
        }
    };
    let lines = vec![
        Line::from(Span::styled(
            truncate_width(&handoff, usize::from(area.width)),
            Style::default().fg(theme.muted),
        )),
        Line::from(Span::styled(
            truncate_width(&keys, usize::from(area.width)),
            Style::default()
                .fg(theme.text)
                .bg(theme.surface)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    Paragraph::new(lines).render(area, buffer);
}

pub(super) fn render_fallback(
    pack: &ValidatedPresentationPack,
    world: &StageWorld,
    view: &ProductView,
    area: Rect,
    theme: Theme,
    buffer: &mut Buffer,
) {
    let source = view
        .runtime
        .source_spirit_id
        .as_ref()
        .map_or("未绑定", |id| id.as_str());
    let source = compact_identifier(source, 20);
    let lines = vec![
        Line::from(Span::styled(
            format!(
                "{} · {}",
                pack.manifest().world.name,
                pack.companion().display_name
            ),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "运行时: {} · 执行者: {}",
            runtime_health_label(view.runtime.health),
            source
        )),
        Line::from(format!(
            "执行: {} · 注意力: {}",
            execution_phase_label(view.execution.phase),
            attention_label(world.attention)
        )),
        Line::from(format!(
            "连续性: 状态{} / 投影{} / 省略{}",
            view.states.len(),
            if view.projection.is_some() {
                "就绪"
            } else {
                "空"
            },
            view.handoff.omitted_count
        )),
        Line::from(""),
        Line::from(Span::styled(
            "终端尺寸不足，请调整到至少 72x22。",
            Style::default()
                .fg(theme.urgent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("[R]刷新  [Q]退出"),
    ];
    Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border)),
        )
        .style(Style::default().fg(theme.text).bg(theme.ink))
        .render(area, buffer);
}
