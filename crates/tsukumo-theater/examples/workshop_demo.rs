//! Print-mode workshop demo: fixture → Director → StageWorld → HalfBlock frame.
//!
//! ```text
//! cargo +stable-x86_64-pc-windows-gnu run -p tsukumo-theater --example workshop_demo
//! ```
//!
//! Optional: `--ticks N` advances walk animation before printing the final frame.

use std::env;
use std::path::PathBuf;
use tsukumo_kernel::read_jsonl_events;
use tsukumo_theater::{
    drive_kernel_events, render_frame_string, DirectorContext, StageWorld, DEFAULT_FRAME_HEIGHT,
    DEFAULT_FRAME_WIDTH,
};

fn main() {
    let ticks = env::args()
        .skip_while(|a| a != "--ticks")
        .nth(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/minimal_quest.jsonl");
    let events = read_jsonl_events(&fixture).expect("load minimal_quest.jsonl");

    let ctx = DirectorContext::default();
    let mut world = StageWorld::new().with_log_cap(24);
    world.ensure_placeholder("gina");
    drive_kernel_events(&mut world, &events, &ctx);

    for _ in 0..ticks {
        // Re-enter walk briefly so ticks are visible even after Celebrate settle.
        if let Some(a) = world.primary_mut() {
            if a.motion != tsukumo_theater::Motion::Walk {
                a.motion = tsukumo_theater::Motion::Walk;
            }
        }
        world.tick();
    }

    let frame = render_frame_string(&world, DEFAULT_FRAME_WIDTH, DEFAULT_FRAME_HEIGHT);
    println!("{frame}");
    println!();
    println!(
        "— demo: {} kernel events → stage (attention={:?}, log={})",
        events.len(),
        world.attention,
        world.log.len()
    );
}
