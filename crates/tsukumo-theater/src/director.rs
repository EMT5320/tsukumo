//! Pure-function Director: KernelEvent → StageEvent(s).
//!
//! Realtime mode = state mapping (lossy). Narrative editing is post-P0.

use crate::stage::{ActorPose, AttentionTier, StageEvent};
use tsukumo_kernel::{ExecutorId, KernelEvent};

/// Optional line templates keyed by coarse situation. Empty book → fallback copy.
#[derive(Debug, Clone, Default)]
pub struct LineBook {
    pub tool_start: Option<String>,
    pub tool_end_ok: Option<String>,
    pub tool_end_err: Option<String>,
    pub waiting: Option<String>,
    pub turn_end: Option<String>,
    pub error: Option<String>,
}

/// Context the director may consult. World state is deferred; keep this thin.
#[derive(Debug, Clone, Default)]
pub struct DirectorContext {
    pub line_book: LineBook,
}

fn pick<'a>(custom: Option<&'a str>, fallback: &'a str) -> &'a str {
    custom.filter(|s| !s.is_empty()).unwrap_or(fallback)
}

fn executor_of(event: &KernelEvent) -> Option<ExecutorId> {
    match event {
        KernelEvent::ToolStart { executor_id, .. }
        | KernelEvent::ToolEnd { executor_id, .. }
        | KernelEvent::WaitingPermission { executor_id, .. }
        | KernelEvent::TurnOrQuestEnd { executor_id, .. }
        | KernelEvent::Error { executor_id, .. } => executor_id.clone(),
    }
}

/// Map one kernel event into zero or more stage events.
///
/// Pure: no I/O, no clocks, no mutable globals.
pub fn direct(event: &KernelEvent, ctx: &DirectorContext) -> Vec<StageEvent> {
    let executor_id = executor_of(event);
    let book = &ctx.line_book;

    match event {
        KernelEvent::ToolStart { tool, call_id, .. } => {
            let fallback = format!("using {tool}…");
            let bubble = pick(book.tool_start.as_deref(), &fallback).to_string();
            vec![
                StageEvent::AttentionTier {
                    tier: AttentionTier::Focus,
                },
                StageEvent::ActorPose {
                    pose: ActorPose::Work,
                    executor_id: executor_id.clone(),
                },
                StageEvent::Bubble {
                    text: bubble,
                    executor_id: executor_id.clone(),
                },
                StageEvent::LogLine {
                    text: format!("tool_start {tool} ({call_id})"),
                    executor_id,
                },
            ]
        }
        KernelEvent::ToolEnd {
            call_id,
            result,
            is_error,
            ..
        } => {
            let (pose, tier, bubble) = if *is_error {
                (
                    ActorPose::Upset,
                    AttentionTier::Urgent,
                    pick(book.tool_end_err.as_deref(), &result.summary),
                )
            } else {
                (
                    ActorPose::Work,
                    AttentionTier::Focus,
                    pick(book.tool_end_ok.as_deref(), &result.summary),
                )
            };
            vec![
                StageEvent::AttentionTier { tier },
                StageEvent::ActorPose {
                    pose,
                    executor_id: executor_id.clone(),
                },
                StageEvent::Bubble {
                    text: bubble.to_string(),
                    executor_id: executor_id.clone(),
                },
                StageEvent::LogLine {
                    text: format!(
                        "tool_end {call_id}{}: {}",
                        if *is_error { " ERR" } else { "" },
                        result.summary
                    ),
                    executor_id,
                },
            ]
        }
        KernelEvent::WaitingPermission { reason, request_id, .. } => {
            let bubble = pick(book.waiting.as_deref(), "need your OK…");
            vec![
                StageEvent::AttentionTier {
                    tier: AttentionTier::Urgent,
                },
                StageEvent::ActorPose {
                    pose: ActorPose::Wait,
                    executor_id: executor_id.clone(),
                },
                StageEvent::Bubble {
                    text: bubble.to_string(),
                    executor_id: executor_id.clone(),
                },
                StageEvent::LogLine {
                    text: format!("waiting_permission {request_id}: {reason}"),
                    executor_id,
                },
            ]
        }
        KernelEvent::TurnOrQuestEnd { summary, quest_id, .. } => {
            let default = summary
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("quest complete");
            let bubble = pick(book.turn_end.as_deref(), default);
            let log = match quest_id {
                Some(qid) => format!("turn_or_quest_end {qid}: {default}"),
                None => format!("turn_or_quest_end: {default}"),
            };
            vec![
                StageEvent::AttentionTier {
                    tier: AttentionTier::Ambient,
                },
                StageEvent::ActorPose {
                    pose: ActorPose::Celebrate,
                    executor_id: executor_id.clone(),
                },
                StageEvent::Bubble {
                    text: bubble.to_string(),
                    executor_id: executor_id.clone(),
                },
                StageEvent::LogLine {
                    text: log,
                    executor_id,
                },
            ]
        }
        KernelEvent::Error {
            message,
            recoverable,
            ..
        } => {
            let bubble = pick(book.error.as_deref(), message);
            vec![
                StageEvent::AttentionTier {
                    tier: AttentionTier::Urgent,
                },
                StageEvent::ActorPose {
                    pose: ActorPose::Upset,
                    executor_id: executor_id.clone(),
                },
                StageEvent::Bubble {
                    text: bubble.to_string(),
                    executor_id: executor_id.clone(),
                },
                StageEvent::LogLine {
                    text: format!(
                        "error{}: {message}",
                        if *recoverable { " (recoverable)" } else { "" }
                    ),
                    executor_id,
                },
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsukumo_kernel::{BackendKind, ExecutorId, ToolResult};

    fn ctx() -> DirectorContext {
        DirectorContext::default()
    }

    #[test]
    fn tool_start_maps_to_work_pose() {
        let ev = KernelEvent::ToolStart {
            call_id: "c1".into(),
            tool: "read".into(),
            args: None,
            executor_id: Some(ExecutorId::new("gina")),
            backend: Some(BackendKind::Fixture),
        };
        let out = direct(&ev, &ctx());
        assert!(out.iter().any(|e| matches!(
            e,
            StageEvent::ActorPose {
                pose: ActorPose::Work,
                ..
            }
        )));
        assert!(out.iter().any(|e| matches!(
            e,
            StageEvent::AttentionTier {
                tier: AttentionTier::Focus
            }
        )));
    }

    #[test]
    fn waiting_permission_raises_urgent() {
        let ev = KernelEvent::WaitingPermission {
            request_id: "p1".into(),
            reason: "shell: rm".into(),
            executor_id: None,
            backend: None,
        };
        let out = direct(&ev, &ctx());
        assert!(out.iter().any(|e| matches!(
            e,
            StageEvent::AttentionTier {
                tier: AttentionTier::Urgent
            }
        )));
        assert!(out.iter().any(|e| matches!(
            e,
            StageEvent::ActorPose {
                pose: ActorPose::Wait,
                ..
            }
        )));
    }

    #[test]
    fn tool_end_error_uses_upset() {
        let ev = KernelEvent::ToolEnd {
            call_id: "c1".into(),
            result: ToolResult::text("permission denied"),
            is_error: true,
            executor_id: None,
            backend: None,
        };
        let out = direct(&ev, &ctx());
        assert!(out.iter().any(|e| matches!(
            e,
            StageEvent::ActorPose {
                pose: ActorPose::Upset,
                ..
            }
        )));
    }

    #[test]
    fn turn_end_resets_ambient() {
        let ev = KernelEvent::TurnOrQuestEnd {
            quest_id: Some("q1".into()),
            summary: Some("all good".into()),
            executor_id: None,
            backend: None,
        };
        let out = direct(&ev, &ctx());
        assert!(out.iter().any(|e| matches!(
            e,
            StageEvent::AttentionTier {
                tier: AttentionTier::Ambient
            }
        )));
        assert!(out.iter().any(|e| matches!(
            e,
            StageEvent::ActorPose {
                pose: ActorPose::Celebrate,
                ..
            }
        )));
    }

    #[test]
    fn line_book_overrides_bubble() {
        let mut ctx = DirectorContext::default();
        ctx.line_book.waiting = Some("会长，求批准～".into());
        let ev = KernelEvent::WaitingPermission {
            request_id: "p1".into(),
            reason: "shell".into(),
            executor_id: None,
            backend: None,
        };
        let out = direct(&ev, &ctx);
        let bubble = out.iter().find_map(|e| match e {
            StageEvent::Bubble { text, .. } => Some(text.as_str()),
            _ => None,
        });
        assert_eq!(bubble, Some("会长，求批准～"));
    }
}
