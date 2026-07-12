//! Generic pack-free renderer retained for fixtures and print examples.

use super::text::truncate_width;
use crate::world::StageWorld;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use unicode_width::UnicodeWidthStr;

pub const DEFAULT_FRAME_WIDTH: u16 = 72;
pub const DEFAULT_FRAME_HEIGHT: u16 = 22;

pub fn render_frame(world: &StageWorld, width: u16, height: u16) -> Buffer {
    let area = Rect::new(0, 0, width.max(1), height.max(1));
    let mut buffer = Buffer::empty(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);
    render_stage(world, chunks[0], &mut buffer);
    render_log(world, chunks[1], &mut buffer);
    buffer
}

pub fn buffer_to_string(buffer: &Buffer) -> String {
    let area = buffer.area();
    let mut output =
        String::with_capacity((usize::from(area.width) + 1) * usize::from(area.height));
    for y in 0..area.height {
        let mut x = 0;
        while x < area.width {
            let Some(cell) = buffer.cell((x, y)) else {
                break;
            };
            let symbol = cell.symbol();
            if symbol.is_empty() {
                output.push(' ');
                x = x.saturating_add(1);
                continue;
            }
            output.push_str(symbol);
            let width = UnicodeWidthStr::width(symbol).max(1);
            let Ok(cell_width) = u16::try_from(width) else {
                break;
            };
            x = x.saturating_add(cell_width);
        }
        if y + 1 < area.height {
            output.push('\n');
        }
    }
    output
}

pub fn render_frame_string(world: &StageWorld, width: u16, height: u16) -> String {
    buffer_to_string(&render_frame(world, width, height))
}

fn render_stage(world: &StageWorld, area: Rect, buffer: &mut Buffer) {
    let title = world.primary().map_or_else(
        || " 工房 · 空 ".to_owned(),
        |actor| {
            format!(
                " 工房 · {} · 来源 {} · {}/{} ",
                actor.id.as_str(),
                actor
                    .source_spirit_id
                    .as_ref()
                    .map_or("unbound", tsukumo_kernel::SpiritId::as_str),
                actor.motion.as_str(),
                pose_label(actor.pose)
            )
        },
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    block.render(area, buffer);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    if let Some(actor) = world.primary() {
        let x = inner.x + inner.width / 2;
        let y = inner.y + inner.height / 2;
        for (offset_x, offset_y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            if let Some(cell) = buffer.cell_mut((x + offset_x, y + offset_y)) {
                cell.set_symbol("▀");
                cell.set_style(Style::default().fg(Color::LightCyan).bg(Color::Blue));
            }
        }
        if let Some(bubble) = actor.bubble.as_deref() {
            Paragraph::new(truncate_width(bubble, usize::from(inner.width)))
                .style(Style::default().fg(Color::White))
                .render(Rect::new(inner.x, inner.y, inner.width, 1), buffer);
        }
    }
}

fn render_log(world: &StageWorld, area: Rect, buffer: &mut Buffer) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" 日志 ", Style::default().fg(Color::Blue)));
    let inner = block.inner(area);
    block.render(area, buffer);
    let mut entries = world
        .log
        .iter()
        .rev()
        .take(usize::from(inner.height))
        .collect::<Vec<_>>();
    entries.reverse();
    let lines = entries
        .into_iter()
        .map(|entry| {
            let text = format!("[{}] {}", entry.attribution.source_spirit_id, entry.text);
            Line::from(truncate_width(&text, usize::from(inner.width)))
        })
        .collect::<Vec<_>>();
    Paragraph::new(lines).render(inner, buffer);
}

const fn pose_label(pose: crate::stage::ActorPose) -> &'static str {
    match pose {
        crate::stage::ActorPose::Idle => "idle",
        crate::stage::ActorPose::Walk => "walk",
        crate::stage::ActorPose::Work => "work",
        crate::stage::ActorPose::Wait => "wait",
        crate::stage::ActorPose::Celebrate => "celebrate",
        crate::stage::ActorPose::Upset => "urgent",
    }
}
