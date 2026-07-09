//! R4: cross-session recall — session1 writes a fact, session2 brief/recall hits it.

use tempfile::tempdir;
use tsukumo_soul::{
    assemble_delegation_prompt, assemble_with_trace, BriefCompiler, BriefOptions, FactKind,
    SkillSocket, SkillStub, SoulStore, TraceEvent, TraceLog, DEFAULT_BRIEF_CHAR_CAP,
};

#[test]
fn r4_cross_session_recall_hits_prior_fact() {
    let dir = tempdir().unwrap();
    let data = dir.path().join("soul-data");

    // --- Session 1: write a durable user fact ---
    {
        let mut store = SoulStore::open(&data).unwrap();
        store
            .remember_user(
                "pref-toolchain",
                "session-1",
                "Owner prefers the gnu Rust toolchain on Windows",
            )
            .unwrap();
        let snap = store.read_snapshot(FactKind::User).unwrap();
        assert!(snap.contains("gnu Rust toolchain"));
    }

    // --- Session 2: reopen same data dir; recall + brief must hit ---
    {
        let store = SoulStore::open(&data).unwrap();
        let hits = store.recall("gnu toolchain", 5).unwrap();
        assert!(
            !hits.is_empty(),
            "session-2 recall must find session-1 fact"
        );
        assert_eq!(hits[0].session_id, "session-1");
        assert!(hits[0].text.contains("gnu"));

        let mut trace = TraceLog::open(&data);
        trace
            .append(TraceEvent::Recall {
                query: "gnu toolchain".into(),
                hit_count: hits.len(),
                session_id: Some("session-2".into()),
            })
            .unwrap();

        let compiler = BriefCompiler::new(BriefOptions {
            char_cap: DEFAULT_BRIEF_CHAR_CAP,
            top_k: 5,
            query: "gnu".into(),
        });
        let brief = compiler.compile(&store).unwrap();
        assert!(
            brief.contains("gnu"),
            "brief must include recalled fact: {brief}"
        );

        let prompt = assemble_with_trace(
            &brief,
            "Run the theater fixture replay",
            Some("quest-r4"),
            Some(&mut trace),
        );
        assert!(prompt.contains("Relationship brief"));
        assert!(prompt.contains("Run the theater fixture replay"));
        assert!(!prompt.contains("傲娇"));
        assert!(!prompt.contains("本小姐"));

        let body = std::fs::read_to_string(trace.path()).unwrap();
        assert!(body.contains("\"type\":\"recall\""));
        assert!(body.contains("\"type\":\"inject\""));
    }

    // Skills socket present, empty, no precipitation UI surface.
    let stub = SkillStub::from_data_dir(&data).unwrap();
    assert!(stub.skills_dir().is_dir());
    assert!(stub.list().is_empty());
}

#[test]
fn assemble_hook_is_stable_for_a1() {
    let prompt = assemble_delegation_prompt(
        "[5% — 40/800]\n## USER\n- Owner prefers concise logs\n",
        "Ship Phase R",
    );
    assert!(prompt.starts_with("## Relationship brief"));
    assert!(prompt.contains("## Goal\nShip Phase R"));
}
