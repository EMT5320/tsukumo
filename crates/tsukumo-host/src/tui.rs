//! Interactive terminal composition and bounded event loop.

mod input;
mod lifecycle;
mod local;

pub use input::{color_capability_from_env, map_terminal_key};

use crate::{ProductControl, ProductController, ProductControllerError, ProductSnapshot};
use crossterm::event::{self, Event};
use input::detect_color_capability;
use lifecycle::{install_panic_restoration_hook, TerminalGuard};
use local::refresh_snapshot;
use std::io;
use std::time::{Duration, Instant};
use thiserror::Error;
use tsukumo_theater::{reduce_app, AppState, ProductWidget, UiInput, ValidatedPresentationPack};

const LOGIC_TICK: Duration = Duration::from_millis(100);
const RENDER_INTERVAL: Duration = Duration::from_millis(50);
const INPUT_POLL: Duration = Duration::from_millis(25);
const PRODUCT_REFRESH: Duration = Duration::from_secs(1);

#[derive(Debug, Error)]
pub enum TuiError {
    #[error("terminal I/O failed: {0}")]
    Io(#[from] io::Error),
    #[error("host product controller failed: {0}")]
    Product(#[from] ProductControllerError),
}

/// Runs one interactive product session after pack, storage, and read-model preflight succeeds.
pub fn run_tui(
    pack: &ValidatedPresentationPack,
    controller: &mut dyn ProductController,
    snapshot: ProductSnapshot,
    reduced_motion: bool,
) -> Result<(), TuiError> {
    install_panic_restoration_hook();
    let mut terminal = TerminalGuard::enter()?;
    let capability = detect_color_capability();
    let mut world = snapshot.world;
    let mut view = snapshot.view;
    let mut revision = snapshot.revision;
    let mut app = AppState::new(reduced_motion);
    let animated_sprites = pack
        .sprites()
        .animations
        .iter()
        .any(|animation| animation.frame_indices.len() > 1);
    let mut last_tick = Instant::now();
    let mut last_refresh = Instant::now();
    let mut last_render = None;

    loop {
        let now = Instant::now();
        if now.duration_since(last_tick) >= LOGIC_TICK {
            let world_changed = if app.reduced_motion() {
                false
            } else {
                world.tick()
            };
            reduce_app(&mut app, UiInput::Tick, &view);
            if world_changed || (!app.reduced_motion() && animated_sprites) {
                app.mark_dirty();
            }
            last_tick = now;
        }
        if now.duration_since(last_refresh) >= PRODUCT_REFRESH {
            refresh_snapshot(controller, &mut view, &mut world, &mut revision, &mut app)?;
            // Authority changes must be drawn before another destructive input is accepted.
            last_render = None;
            last_refresh = now;
        }

        let render_due = app.is_dirty()
            && last_render
                .map(|rendered: Instant| now.duration_since(rendered) >= RENDER_INTERVAL)
                .unwrap_or(true);
        if render_due {
            terminal.terminal_mut().draw(|frame| {
                let area = frame.area();
                frame.render_widget(
                    ProductWidget::new(pack, &world, &view, &app, capability),
                    area,
                );
            })?;
            app.mark_clean();
            last_render = Some(Instant::now());
        }

        if !event::poll(INPUT_POLL)? {
            continue;
        }
        let input = match event::read()? {
            Event::Key(key) => map_terminal_key(key),
            Event::Resize(width, height) => Some(UiInput::Resize { width, height }),
            Event::FocusGained | Event::FocusLost | Event::Mouse(_) | Event::Paste(_) => None,
        };
        let Some(input) = input else {
            continue;
        };
        let action = reduce_app(&mut app, input, &view);
        if app.is_dirty() {
            // Navigation becomes authoritative only after its resulting frame is visible.
            last_render = None;
        }
        if let Some(action) = action {
            if controller.apply(action)? == ProductControl::Quit {
                break;
            }
            refresh_snapshot(controller, &mut view, &mut world, &mut revision, &mut app)?;
            last_render = None;
            last_refresh = Instant::now();
        }
    }
    Ok(())
}
