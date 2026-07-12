//! Pack-driven workshop scene and companion rendering.

use super::halfblock::{ColorCapability, LogicalSurface};
use super::text::{compact_identifier, truncate_width};
use super::theme::Theme;
use crate::app::{AppState, ProductView};
use crate::pack::{SemanticPose, ValidatedPresentationPack};
use crate::stage::{ActorPose, AttentionTier};
use crate::world::{ActorSlot, StageWorld};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget};

pub(super) fn render_workshop(
    pack: &ValidatedPresentationPack,
    world: &StageWorld,
    view: &ProductView,
    app: &AppState,
    area: Rect,
    capability: ColorCapability,
    buffer: &mut Buffer,
) {
    let theme = Theme::from_palette(pack.palette(), capability);
    let title = format!(
        " {} · {} · {} ",
        pack.manifest().terminology.workshop,
        pack.companion().display_name,
        attention_label(world.attention)
    );
    let border = if world.attention == AttentionTier::Urgent {
        theme.urgent
    } else {
        theme.border
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(border))
        .title(Span::styled(
            title,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    block.render(area, buffer);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let mut surface = LogicalSurface::for_area(inner);
    let scene = pack.scene();
    let origin_x = (i32::from(inner.width) - i32::from(scene.width)) / 2;
    let origin_y = (i32::from(inner.height.saturating_mul(2)) - i32::from(scene.height)) / 2;
    for layer in &scene.layers {
        surface.blit(
            &layer.pixels,
            origin_x + i32::from(layer.x),
            origin_y + i32::from(layer.y),
        );
    }

    if let Some(actor) = world.primary() {
        if let Some(frame) = sprite_frame(pack, actor.pose, app.animation_frame()) {
            let actor_x = actor_scene_x(pack, actor);
            let sprite_x = i32::from(actor_x) - i32::from(frame.pixels.width / 2);
            let sprite_y = i32::from(scene.walk_bounds.y) - i32::from(frame.pixels.height);
            surface.blit(&frame.pixels, origin_x + sprite_x, origin_y + sprite_y);
        }
    }
    surface.render(inner, pack.palette(), capability, buffer);

    render_runtime_plaque(pack, view, inner, origin_x, origin_y, theme, buffer);
    render_actor_copy(pack, world, inner, theme, buffer);
    render_facility_legend(pack, inner, theme, buffer);
}

fn sprite_frame(
    pack: &ValidatedPresentationPack,
    pose: ActorPose,
    animation_frame: u64,
) -> Option<&crate::pack::SpriteFrame> {
    let semantic = match pose {
        ActorPose::Idle | ActorPose::Walk => SemanticPose::Idle,
        ActorPose::Work => SemanticPose::Work,
        ActorPose::Wait => SemanticPose::Wait,
        ActorPose::Celebrate => SemanticPose::Celebrate,
        ActorPose::Upset => SemanticPose::Urgent,
    };
    let animation = pack
        .sprites()
        .animations
        .iter()
        .find(|animation| animation.pose == semantic)?;
    let frame_count = u64::try_from(animation.frame_indices.len()).ok()?;
    let step = animation_frame / u64::from(animation.frame_ticks);
    let selected = usize::try_from(step % frame_count).ok()?;
    let frame_index = *animation.frame_indices.get(selected)?;
    pack.sprites().frames.get(frame_index)
}

fn actor_scene_x(pack: &ValidatedPresentationPack, actor: &ActorSlot) -> u16 {
    let bounds = pack.scene().walk_bounds;
    if matches!(actor.pose, ActorPose::Wait | ActorPose::Upset) {
        return pack
            .scene()
            .facilities
            .iter()
            .find(|facility| facility.id.as_str() == "permission_station")
            .map_or(bounds.max_x, |facility| facility.x);
    }
    let position = actor
        .x
        .clamp(i32::from(bounds.min_x), i32::from(bounds.max_x));
    u16::try_from(position).unwrap_or(bounds.min_x)
}

fn render_runtime_plaque(
    pack: &ValidatedPresentationPack,
    view: &ProductView,
    area: Rect,
    origin_x: i32,
    origin_y: i32,
    theme: Theme,
    buffer: &mut Buffer,
) {
    let Some(portal) = pack
        .scene()
        .facilities
        .iter()
        .find(|facility| facility.id.as_str() == "runtime_portal")
    else {
        return;
    };
    // Runtime signage consumes the authoritative read model, independent of lossy stage events.
    let source = view
        .runtime
        .source_spirit_id
        .as_ref()
        .map_or("未绑定", |id| id.as_str());
    let label = compact_identifier(source, 16);
    let width = u16::try_from(super::text::display_width(&label))
        .unwrap_or(16)
        .saturating_add(2)
        .min(area.width);
    let center = origin_x + i32::from(portal.x);
    let relative_x = center.saturating_sub(i32::from(width / 2)).max(0);
    let x = area
        .x
        .saturating_add(u16::try_from(relative_x).unwrap_or_default())
        .min(area.right().saturating_sub(width));
    let logical_y = (origin_y + i32::from(portal.y)).max(0);
    let y = area
        .y
        .saturating_add(u16::try_from(logical_y / 2).unwrap_or_default())
        .min(area.bottom().saturating_sub(1));
    Paragraph::new(format!(" {label} "))
        .style(
            Style::default()
                .fg(theme.ink)
                .bg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .render(Rect::new(x, y, width, 1), buffer);
}

fn render_actor_copy(
    pack: &ValidatedPresentationPack,
    world: &StageWorld,
    area: Rect,
    theme: Theme,
    buffer: &mut Buffer,
) {
    let identity = format!(
        "{} · {}",
        pack.companion().display_name,
        pack.companion().title
    );
    let identity_area = Rect::new(area.x, area.y, area.width, 1);
    Clear.render(identity_area, buffer);
    Paragraph::new(truncate_width(&identity, usize::from(area.width)))
        .style(
            Style::default()
                .fg(theme.text)
                .bg(theme.ink)
                .add_modifier(Modifier::BOLD),
        )
        .render(identity_area, buffer);
    if let Some(bubble) = world.primary().and_then(|actor| actor.bubble.as_deref()) {
        let line = Line::from(vec![
            Span::styled("「", Style::default().fg(theme.accent)),
            Span::styled(
                truncate_width(bubble, usize::from(area.width.saturating_sub(2))),
                Style::default().fg(theme.text),
            ),
            Span::styled("」", Style::default().fg(theme.accent)),
        ]);
        let bubble_area = Rect::new(area.x, area.y.saturating_add(1), area.width, 1);
        Clear.render(bubble_area, buffer);
        Paragraph::new(line)
            .style(Style::default().bg(theme.ink))
            .render(bubble_area, buffer);
    }
}

fn render_facility_legend(
    pack: &ValidatedPresentationPack,
    area: Rect,
    theme: Theme,
    buffer: &mut Buffer,
) {
    let terms = &pack.manifest().terminology;
    let legend = format!(
        "{} · {} · {} · {} · {}",
        terms.quest_board,
        terms.runtime_portal,
        terms.memory_cabinet,
        terms.permission_station,
        terms.projection_desk
    );
    let legend_area = Rect::new(
        area.x,
        area.y + area.height.saturating_sub(1),
        area.width,
        1,
    );
    Clear.render(legend_area, buffer);
    Paragraph::new(truncate_width(&legend, usize::from(area.width)))
        .style(Style::default().fg(theme.text).bg(theme.ink))
        .render(legend_area, buffer);
}

const fn attention_label(tier: AttentionTier) -> &'static str {
    match tier {
        AttentionTier::Ambient => "静默值守",
        AttentionTier::Focus => "登记中",
        AttentionTier::Urgent => "需要裁定",
    }
}
