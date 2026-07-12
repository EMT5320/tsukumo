//! Top-level deterministic product frame composition.

use super::halfblock::ColorCapability;
use super::inspectors::{render_projection_inspector, render_state_inspector};
use super::layout::{frame_layout, LayoutMode};
use super::panels::{render_fallback, render_footer, render_header, render_log};
use super::permission::render_permission;
use super::theme::Theme;
use super::workshop::render_workshop;
use crate::app::{AppState, ProductView, Screen};
use crate::pack::ValidatedPresentationPack;
use crate::world::StageWorld;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

pub struct ProductWidget<'a> {
    pack: &'a ValidatedPresentationPack,
    world: &'a StageWorld,
    view: &'a ProductView,
    app: &'a AppState,
    capability: ColorCapability,
}

impl<'a> ProductWidget<'a> {
    pub const fn new(
        pack: &'a ValidatedPresentationPack,
        world: &'a StageWorld,
        view: &'a ProductView,
        app: &'a AppState,
        capability: ColorCapability,
    ) -> Self {
        Self {
            pack,
            world,
            view,
            app,
            capability,
        }
    }
}

impl Widget for ProductWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        render_product_into(
            self.pack,
            self.world,
            self.view,
            self.app,
            area,
            self.capability,
            buffer,
        );
    }
}

pub fn render_product_frame(
    pack: &ValidatedPresentationPack,
    world: &StageWorld,
    view: &ProductView,
    app: &AppState,
    width: u16,
    height: u16,
    capability: ColorCapability,
) -> Buffer {
    let area = Rect::new(0, 0, width.max(1), height.max(1));
    let mut buffer = Buffer::empty(area);
    render_product_into(pack, world, view, app, area, capability, &mut buffer);
    buffer
}

fn render_product_into(
    pack: &ValidatedPresentationPack,
    world: &StageWorld,
    view: &ProductView,
    app: &AppState,
    area: Rect,
    capability: ColorCapability,
    buffer: &mut Buffer,
) {
    let theme = Theme::from_palette(pack.palette(), capability);
    buffer.set_style(area, Style::default().fg(theme.text).bg(theme.ink));
    let layout = frame_layout(area);

    if layout.mode == LayoutMode::Fallback {
        render_fallback(pack, world, view, area, theme, buffer);
    } else {
        render_header(pack, world, view, layout.header, theme, buffer);
        match app.screen() {
            Screen::Workshop => {
                render_workshop(pack, world, view, app, layout.body, capability, buffer);
            }
            Screen::StateInspector { selected } => {
                render_state_inspector(
                    pack,
                    view,
                    selected,
                    app.inspector_page(),
                    layout.body,
                    theme,
                    buffer,
                );
            }
            Screen::ProjectionInspector => {
                render_projection_inspector(
                    pack,
                    view,
                    app.inspector_page(),
                    layout.body,
                    theme,
                    buffer,
                );
            }
        }
        render_log(world, view, layout.log, theme, buffer);
        render_footer(pack, view, app, layout.footer, theme, buffer);
    }

    if let Some(permission) = &view.pending_permission {
        render_permission(
            &pack.manifest().terminology.permission_station,
            permission,
            app.permission_page(),
            area,
            Theme::safety(capability),
            buffer,
        );
    }
}
