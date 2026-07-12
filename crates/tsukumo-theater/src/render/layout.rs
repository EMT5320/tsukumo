//! Deterministic terminal layout thresholds and rectangles.

use ratatui::layout::Rect;

pub const FULL_MIN_WIDTH: u16 = 100;
pub const FULL_MIN_HEIGHT: u16 = 30;
pub const COMPACT_MIN_WIDTH: u16 = 72;
pub const COMPACT_MIN_HEIGHT: u16 = 22;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    Full,
    Compact,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct FrameLayout {
    pub mode: LayoutMode,
    pub header: Rect,
    pub body: Rect,
    pub log: Rect,
    pub footer: Rect,
}

pub const fn select_layout(width: u16, height: u16) -> LayoutMode {
    if width >= FULL_MIN_WIDTH && height >= FULL_MIN_HEIGHT {
        LayoutMode::Full
    } else if width >= COMPACT_MIN_WIDTH && height >= COMPACT_MIN_HEIGHT {
        LayoutMode::Compact
    } else {
        LayoutMode::Fallback
    }
}

pub(super) const fn frame_layout(area: Rect) -> FrameLayout {
    match select_layout(area.width, area.height) {
        LayoutMode::Full => {
            let header_height = 2;
            let log_height = 6;
            let footer_height = 3;
            let body_height = area
                .height
                .saturating_sub(header_height + log_height + footer_height);
            FrameLayout {
                mode: LayoutMode::Full,
                header: Rect::new(area.x, area.y, area.width, header_height),
                body: Rect::new(area.x, area.y + header_height, area.width, body_height),
                log: Rect::new(
                    area.x,
                    area.y + header_height + body_height,
                    area.width,
                    log_height,
                ),
                footer: Rect::new(
                    area.x,
                    area.y + header_height + body_height + log_height,
                    area.width,
                    footer_height,
                ),
            }
        }
        LayoutMode::Compact => {
            let header_height = 2;
            let log_height = 4;
            let footer_height = 2;
            let body_height = area
                .height
                .saturating_sub(header_height + log_height + footer_height);
            FrameLayout {
                mode: LayoutMode::Compact,
                header: Rect::new(area.x, area.y, area.width, header_height),
                body: Rect::new(area.x, area.y + header_height, area.width, body_height),
                log: Rect::new(
                    area.x,
                    area.y + header_height + body_height,
                    area.width,
                    log_height,
                ),
                footer: Rect::new(
                    area.x,
                    area.y + header_height + body_height + log_height,
                    area.width,
                    footer_height,
                ),
            }
        }
        LayoutMode::Fallback => FrameLayout {
            mode: LayoutMode::Fallback,
            header: area,
            body: area,
            log: Rect::new(0, 0, 0, 0),
            footer: Rect::new(0, 0, 0, 0),
        },
    }
}
