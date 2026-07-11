//! Cross-session recall demo (Phase R / F3 probe).
//!
//! ```text
//! cargo +stable-x86_64-pc-windows-gnu run -p tsukumo-soul --example recall_demo
//! ```
//!
//! Simulates session-1 write → session-2 brief/recall on the same data dir.

use std::env;
use std::path::PathBuf;
use tsukumo_kernel::{QuestId, SessionId};
use tsukumo_soul::{
    assemble_with_trace, BriefCompiler, BriefOptions, SkillSocket, SkillStub, SoulStore,
    TraceEvent, TraceLog, DEFAULT_BRIEF_CHAR_CAP,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data: PathBuf = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("tsukumo-soul-recall-demo"));

    println!("soul data dir: {}", data.display());

    // Session 1
    {
        let mut store = SoulStore::open(&data)?;
        store.remember_user(
            "pref-toolchain",
            "session-1",
            "Owner prefers the gnu Rust toolchain on Windows",
        )?;
        store.remember_memory(
            "quest-note",
            "session-1",
            "Last workshop check used HalfBlock fixture replay",
        )?;
        println!("session-1: wrote USER + MEMORY facts");
        println!(
            "--- USER.md ---\n{}",
            store.read_snapshot(tsukumo_soul::FactKind::User)?
        );
    }

    // Session 2
    {
        let store = SoulStore::open(&data)?;
        let hits = store.recall("gnu", 5)?;
        println!("session-2 recall hits: {}", hits.len());
        for h in &hits {
            println!("  [{} / {}] {}", h.kind.as_str(), h.session_id, h.text);
        }

        let mut trace = TraceLog::open(&data);
        trace.append(TraceEvent::Recall {
            query: "gnu".into(),
            hit_count: hits.len(),
            session_id: Some(SessionId::new("session-2")),
        })?;

        let brief = BriefCompiler::new(BriefOptions {
            char_cap: DEFAULT_BRIEF_CHAR_CAP,
            top_k: 5,
            query: "gnu".into(),
        })
        .compile(&store)?;

        let prompt = assemble_with_trace(
            &brief,
            "Continue the workshop check",
            Some(&QuestId::new("demo-quest")),
            Some(&mut trace),
        )?;

        println!("--- brief ---\n{brief}");
        println!("--- assembled prompt ---\n{prompt}");
        println!("trace log: {}", trace.path().display());

        let skills = SkillStub::from_data_dir(&data)?;
        println!(
            "skills socket: {} ({} skills)",
            skills.skills_dir().display(),
            skills.list().len()
        );
    }

    Ok(())
}
