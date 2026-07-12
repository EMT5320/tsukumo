//! Direct logical-pixel composition into terminal HalfBlock cells.

use crate::pack::{MonochromeTone, Palette, PaletteIndex, PixelGrid};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    TrueColor,
    Ansi256,
    Monochrome,
}

pub(super) struct LogicalSurface {
    width: u16,
    height: u16,
    pixels: Vec<Option<PaletteIndex>>,
}

impl LogicalSurface {
    pub fn for_area(area: Rect) -> Self {
        let height = area.height.saturating_mul(2);
        Self {
            width: area.width,
            height,
            pixels: vec![None; usize::from(area.width) * usize::from(height)],
        }
    }

    pub fn blit(&mut self, grid: &PixelGrid, origin_x: i32, origin_y: i32) {
        for source_y in 0..grid.height {
            for source_x in 0..grid.width {
                let Some(index) = grid.pixel(source_x, source_y) else {
                    continue;
                };
                let target_x = origin_x + i32::from(source_x);
                let target_y = origin_y + i32::from(source_y);
                if target_x < 0
                    || target_y < 0
                    || target_x >= i32::from(self.width)
                    || target_y >= i32::from(self.height)
                {
                    continue;
                }
                let Ok(target_x) = u16::try_from(target_x) else {
                    continue;
                };
                let Ok(target_y) = u16::try_from(target_y) else {
                    continue;
                };
                let offset =
                    usize::from(target_y) * usize::from(self.width) + usize::from(target_x);
                self.pixels[offset] = Some(index);
            }
        }
    }

    pub fn render(
        &self,
        area: Rect,
        palette: &Palette,
        capability: ColorCapability,
        buffer: &mut Buffer,
    ) {
        let colors = palette
            .colors
            .iter()
            .map(|color| resolve_color(color.rgb, color.ansi256, color.monochrome, capability))
            .collect::<Vec<_>>();
        for cell_y in 0..area.height {
            for cell_x in 0..area.width {
                let top = self.pixel(cell_x, cell_y.saturating_mul(2));
                let bottom = self.pixel(cell_x, cell_y.saturating_mul(2).saturating_add(1));
                if top.is_none() && bottom.is_none() {
                    continue;
                }
                if let Some(cell) = buffer.cell_mut((area.x + cell_x, area.y + cell_y)) {
                    // Transparent halves inherit the already-resolved Pack backdrop.
                    let backdrop = cell.bg;
                    let foreground = color_at(top, &colors).unwrap_or(backdrop);
                    let background = color_at(bottom, &colors).unwrap_or(backdrop);
                    cell.set_symbol("▀");
                    cell.set_style(Style::default().fg(foreground).bg(background));
                }
            }
        }
    }

    fn pixel(&self, x: u16, y: u16) -> Option<PaletteIndex> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.pixels[usize::from(y) * usize::from(self.width) + usize::from(x)]
    }
}

fn color_at(index: Option<PaletteIndex>, colors: &[Color]) -> Option<Color> {
    index.and_then(|value| colors.get(value.as_usize()).copied())
}

pub(super) const fn resolve_color(
    rgb: [u8; 3],
    ansi256: u8,
    monochrome: MonochromeTone,
    capability: ColorCapability,
) -> Color {
    match capability {
        ColorCapability::TrueColor => Color::Rgb(rgb[0], rgb[1], rgb[2]),
        ColorCapability::Ansi256 => Color::Indexed(ansi256),
        ColorCapability::Monochrome => match monochrome {
            MonochromeTone::Black => Color::Black,
            MonochromeTone::DarkGray => Color::DarkGray,
            MonochromeTone::Gray => Color::Gray,
            MonochromeTone::White => Color::White,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pack::{ContentId, PaletteColor, PaletteRoles, PixelGrid};

    fn palette() -> Palette {
        let index = PaletteIndex(0);
        Palette {
            colors: vec![PaletteColor {
                name: ContentId("pixel".to_owned()),
                rgb: [10, 20, 30],
                ansi256: 24,
                monochrome: MonochromeTone::White,
            }],
            roles: PaletteRoles {
                ink: index,
                surface: index,
                border: index,
                text_primary: index,
                text_muted: index,
                accent: index,
                urgent: index,
            },
        }
    }

    #[test]
    fn transparent_half_when_rendered_inherits_pack_backdrop() {
        // Given: a logical top pixel over a buffer already painted by the Pack theme.
        let area = Rect::new(0, 0, 1, 1);
        let mut buffer = Buffer::empty(area);
        buffer.set_style(area, Style::default().bg(Color::Indexed(42)));
        let mut surface = LogicalSurface::for_area(area);
        surface.blit(
            &PixelGrid {
                width: 1,
                height: 2,
                pixels: vec![Some(PaletteIndex(0)), None],
            },
            0,
            0,
        );

        // When: HalfBlock composition resolves the transparent lower half.
        surface.render(area, &palette(), ColorCapability::Ansi256, &mut buffer);

        // Then: the lower half retains Pack ink instead of terminal Reset.
        let cell = buffer.cell((0, 0)).expect("rendered cell");
        assert_eq!(cell.fg, Color::Indexed(24));
        assert_eq!(cell.bg, Color::Indexed(42));
    }
}
