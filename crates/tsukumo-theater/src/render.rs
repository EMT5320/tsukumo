//! Thin HalfBlock workshop renderer (S0 / S1 spectacle ticket).
//!
//! Renders into a ratatui [`Buffer`] so tests and `print` demos need no TTY.
//! Director stays untouched — this module only reads [`StageWorld`].

use crate::stage::{ActorPose, AttentionTier};
use crate::world::{Motion, StageWorld};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Points};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::symbols::Marker;

/// Default demo frame size (fits a modest Windows Terminal pane).
pub const DEFAULT_FRAME_WIDTH: u16 = 72;
pub const DEFAULT_FRAME_HEIGHT: u16 = 22;

/// Render one workshop frame into a ratatui buffer.
pub fn render_frame(world: &StageWorld, width: u16, height: u16) -> Buffer {
    let area = Rect::new(0, 0, width.max(20), height.max(10));
    let mut buf = Buffer::empty(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    render_stage(world, chunks[0], &mut buf);
    render_log(world, chunks[1], &mut buf);
    buf
}

/// Convenience: buffer → plain string for print demos / snapshot asserts.
///
/// Skips wide-char continuation cells so CJK titles/bubbles stay contiguous
/// (Windows Terminal / print mode).
pub fn buffer_to_string(buf: &Buffer) -> String {
    use unicode_width::UnicodeWidthStr;

    let area = buf.area();
    let mut out = String::with_capacity((area.width as usize + 1) * area.height as usize);
    for y in 0..area.height {
        let mut x = 0u16;
        while x < area.width {
            let cell = buf.cell((x, y)).expect("in-bounds cell");
            let symbol = cell.symbol();
            if symbol.is_empty() {
                out.push(' ');
                x += 1;
                continue;
            }
            out.push_str(symbol);
            let w = UnicodeWidthStr::width(symbol).max(1) as u16;
            x = x.saturating_add(w);
        }
        if y + 1 < area.height {
            out.push('\n');
        }
    }
    out
}

pub fn render_frame_string(world: &StageWorld, width: u16, height: u16) -> String {
    buffer_to_string(&render_frame(world, width, height))
}

fn attention_style(tier: AttentionTier) -> Style {
    match tier {
        AttentionTier::Ambient => Style::default().fg(Color::DarkGray),
        AttentionTier::Focus => Style::default().fg(Color::Cyan),
        AttentionTier::Urgent => Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::BOLD),
    }
}

fn pose_label(pose: ActorPose) -> &'static str {
    match pose {
        ActorPose::Idle => "idle",
        ActorPose::Walk => "walk",
        ActorPose::Work => "work",
        ActorPose::Wait => "wait",
        ActorPose::Celebrate => "yay",
        ActorPose::Upset => "upset",
    }
}

fn render_stage(world: &StageWorld, area: Rect, buf: &mut Buffer) {
    let title = match world.primary() {
        Some(a) => format!(
            " 工房 · {} · {}/{} ",
            a.id.as_str(),
            a.motion.as_str(),
            pose_label(a.pose)
        ),
        None => " 工房 · (empty) ".to_string(),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(attention_style(world.attention))
        .title(Span::styled(
            title,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    block.render(area, buf);

    // Floor wash via HalfBlock canvas (S0 static workshop).
    let floor = Canvas::default()
        .marker(Marker::HalfBlock)
        .x_bounds([0.0, 40.0])
        .y_bounds([0.0, 12.0])
        .paint(|ctx| {
            // Floor strip
            let mut floor_pts = Vec::new();
            for x in 0..40 {
                for y in 0..3 {
                    floor_pts.push((x as f64, y as f64));
                }
            }
            ctx.draw(&Points {
                coords: &floor_pts,
                color: Color::Rgb(40, 40, 48),
            });

            // Back wall
            let mut wall = Vec::new();
            for x in 0..40 {
                wall.push((x as f64, 11.0));
                wall.push((x as f64, 10.0));
            }
            ctx.draw(&Points {
                coords: &wall,
                color: Color::Rgb(70, 55, 40),
            });

            // Workbench block
            let mut bench = Vec::new();
            for x in 26..34 {
                for y in 3..6 {
                    bench.push((x as f64, y as f64));
                }
            }
            ctx.draw(&Points {
                coords: &bench,
                color: Color::Rgb(120, 90, 50),
            });

            // Sprite placeholder (HalfBlock cluster) — position from actor.
            if let Some(actor) = world.primary() {
                let ax = actor.x.clamp(0, 38) as f64;
                let ay = (actor.y.clamp(0, 10) + 3) as f64;
                let color = sprite_color(actor.motion, actor.pose);
                let body = sprite_points(ax, ay, actor.motion);
                ctx.draw(&Points {
                    coords: &body,
                    color,
                });
            }
        });
    floor.render(inner, buf);

    // Bubble as overlay text (not Canvas) so CJK stays readable.
    // Clear the row first so HalfBlock floor wash does not leak into print dumps.
    if let Some(actor) = world.primary() {
        if let Some(bubble) = actor.bubble.as_deref() {
            let bubble_y = inner.y.saturating_add(1);
            let bubble_area = Rect {
                x: inner.x,
                y: bubble_y,
                width: inner.width,
                height: 1,
            };
            for x in bubble_area.left()..bubble_area.right() {
                if let Some(cell) = buf.cell_mut((x, bubble_y)) {
                    cell.set_symbol(" ");
                    cell.set_style(Style::default());
                }
            }
            let text = truncate_chars(bubble, (inner.width.saturating_sub(4)) as usize);
            let line = Line::from(vec![
                Span::styled("「", Style::default().fg(Color::White)),
                Span::styled(text, Style::default().fg(Color::White)),
                Span::styled("」", Style::default().fg(Color::White)),
            ]);
            let para = Paragraph::new(line);
            let text_area = Rect {
                x: inner.x.saturating_add(2),
                y: bubble_y,
                width: inner.width.saturating_sub(4).max(1),
                height: 1,
            };
            para.render(text_area, buf);
        }
    }
}

fn sprite_color(motion: Motion, pose: ActorPose) -> Color {
    match (motion, pose) {
        (_, ActorPose::Upset) => Color::Red,
        (_, ActorPose::Celebrate) => Color::LightYellow,
        (_, ActorPose::Wait) => Color::LightMagenta,
        (Motion::Work, _) => Color::LightCyan,
        (Motion::Walk, _) => Color::Green,
        (Motion::Idle, _) => Color::Gray,
    }
}

/// Tiny HalfBlock "person" — a few points, not a full art pipeline.
fn sprite_points(ax: f64, ay: f64, motion: Motion) -> Vec<(f64, f64)> {
    let mut pts = vec![
        (ax, ay + 2.0),
        (ax + 1.0, ay + 2.0),
        (ax, ay + 1.0),
        (ax + 1.0, ay + 1.0),
        (ax + 0.0, ay),
        (ax + 1.0, ay),
    ];
    // Arms / tool hint when working.
    if motion == Motion::Work {
        pts.push((ax + 2.0, ay + 1.0));
        pts.push((ax + 2.0, ay + 2.0));
    }
    // Stride offset when walking.
    if motion == Motion::Walk {
        pts.push((ax - 0.5, ay));
        pts.push((ax + 1.5, ay));
    }
    pts
}

fn render_log(world: &StageWorld, area: Rect, buf: &mut Buffer) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " 日志 ",
            Style::default().fg(Color::Blue),
        ));
    let inner = block.inner(area);
    block.render(area, buf);

    let max_lines = inner.height as usize;
    let lines: Vec<Line> = world
        .log
        .iter()
        .rev()
        .take(max_lines)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|l| {
            Line::from(Span::styled(
                truncate_chars(l, inner.width as usize),
                Style::default().fg(Color::Gray),
            ))
        })
        .collect();

    Paragraph::new(lines).render(inner, buf);
}

fn truncate_chars(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    if max <= 1 {
        return "…".to_string();
    }
    s.chars().take(max - 1).collect::<String>() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stage::StageEvent;
    use tsukumo_kernel::ExecutorId;

    #[test]
    fn static_workshop_shows_title_and_sprite_region() {
        let mut world = StageWorld::new();
        world.ensure_placeholder("gina");
        let frame = render_frame_string(&world, DEFAULT_FRAME_WIDTH, DEFAULT_FRAME_HEIGHT);
        assert!(
            frame.contains("工房"),
            "workshop title missing:\n{frame}"
        );
        assert!(
            frame.contains("gina"),
            "executor soft-id missing:\n{frame}"
        );
        // Border / block art should leave non-space content in the stage pane.
        let non_space = frame.chars().filter(|c| !c.is_whitespace()).count();
        assert!(non_space > 40, "frame looks empty:\n{frame}");
    }

    #[test]
    fn split_log_consumes_stage_events() {
        let mut world = StageWorld::new();
        world.ensure_placeholder("gina");
        world.apply(&StageEvent::Bubble {
            text: "干活中".into(),
            executor_id: Some(ExecutorId::new("gina")),
        });
        world.apply(&StageEvent::LogLine {
            text: "tool_start read (c1)".into(),
            executor_id: None,
        });
        let frame = render_frame_string(&world, DEFAULT_FRAME_WIDTH, DEFAULT_FRAME_HEIGHT);
        assert!(frame.contains("干活中"), "bubble missing:\n{frame}");
        assert!(frame.contains("日志"), "log pane title missing:\n{frame}");
        assert!(
            frame.contains("tool_start"),
            "log line missing:\n{frame}"
        );
    }
}
