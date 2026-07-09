//! A1 adapter demo: synthetic stream-json → KernelEvent → stage (print mode).
//!
//! ```text
//! cargo +stable-x86_64-pc-windows-gnu run -p tsukumo-adapters --example a1_stream_demo
//! ```
//!
//! Optional: `--stop-at-wait` freezes the world on WaitingPermission (Urgent).

use std::env;
use tsukumo_adapters::{
    assemble_prompt, synthetic_demo_events, BriefingSource, NullBriefing, PromptAssemblyContext,
};
use tsukumo_kernel::KernelEvent;
use tsukumo_theater::{
    drive_kernel_events, render_frame_string, DirectorContext, StageWorld, DEFAULT_FRAME_HEIGHT,
    DEFAULT_FRAME_WIDTH,
};

fn main() {
    let stop_at_wait = env::args().any(|a| a == "--stop-at-wait");

    // A1 briefing assembly point (content stub — Phase R fills Soul store).
    let briefing = NullBriefing.briefing_for(&PromptAssemblyContext {
        executor_id: Some("gina".into()),
        quest_id: Some("synth-a1".into()),
    });
    let _prompt = assemble_prompt("synthetic A1 workshop quest", briefing.as_deref());

    let events = synthetic_demo_events("gina");
    let slice: &[KernelEvent] = if stop_at_wait {
        let idx = events
            .iter()
            .position(|e| matches!(e, KernelEvent::WaitingPermission { .. }))
            .expect("synthetic includes waiting_permission");
        &events[..=idx]
    } else {
        &events
    };

    let ctx = DirectorContext::default();
    let mut world = StageWorld::new().with_log_cap(24);
    world.ensure_placeholder("gina");
    drive_kernel_events(&mut world, slice, &ctx);

    let frame = render_frame_string(&world, DEFAULT_FRAME_WIDTH, DEFAULT_FRAME_HEIGHT);
    println!("{frame}");
    println!();
    println!(
        "— a1 demo: {} kernel events → stage (attention={:?}, pose={:?}, log={})",
        slice.len(),
        world.attention,
        world.primary().map(|a| a.pose),
        world.log.len()
    );
    if stop_at_wait {
        println!("  (stopped at waiting_permission — Urgent / Wait)");
    }
}
