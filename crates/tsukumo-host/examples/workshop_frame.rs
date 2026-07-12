//! Print deterministic product frames for terminal visual QA.

use std::error::Error;
use tsukumo_host::{load_presentation_pack, PresentationPackSource};
use tsukumo_kernel::{CheckpointId, EventId, ProjectionId, SpiritId, StateId};
use tsukumo_theater::{
    buffer_to_ansi, buffer_to_string, reduce_app, render_product_frame, ActorPose, AppState,
    ColorCapability, DisplayText, ExecutionPhase, PermissionEvidenceText, PermissionView,
    ProductView, ProjectionStateRefView, ProjectionView, RuntimeHealth, StageAttribution,
    StageEvent, StageWorld, StateStatus, StateView, UiInput, UiKey, UiPermissionId,
};

fn main() -> Result<(), Box<dyn Error>> {
    let ansi = std::env::args().any(|argument| argument == "--ansi");
    let mode = std::env::args().nth(1).unwrap_or_else(|| "full".to_owned());
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)?;
    let attribution = StageAttribution {
        actor_id: pack.companion().actor_id.clone(),
        source_spirit_id: SpiritId::new("runtime-spirit"),
    };
    let bounds = pack.scene().walk_bounds;
    let mut world =
        StageWorld::new().with_walk_bounds(i32::from(bounds.min_x), i32::from(bounds.max_x));
    let pose = match mode.as_str() {
        "pose-idle" => ActorPose::Idle,
        "pose-wait" => ActorPose::Wait,
        "pose-urgent" | "mono-urgent" => ActorPose::Upset,
        "pose-celebrate" => ActorPose::Celebrate,
        "full" | "compact" | "fallback" | "states" | "projection" | "permission" | "mono"
        | "ansi256" | "reduced" | "pose-work" | "motion-start" | "motion-settled" => {
            ActorPose::Work
        }
        _ => ActorPose::Work,
    };
    world.apply(&StageEvent::ActorPose {
        pose,
        attribution: attribution.clone(),
    });
    world.apply(&StageEvent::Bubble {
        text: "会长，当前执行记录已完成核对。".into(),
        attribution: attribution.clone(),
    });
    world.apply(&StageEvent::LogLine {
        text: "tool_start cargo_test (call-17)".into(),
        attribution: attribution.clone(),
    });

    let mut view = sample_view();
    let reduced_motion = mode == "reduced";
    let mut app = AppState::new(reduced_motion);
    let (width, height) = match mode.as_str() {
        "compact" | "permission" | "mono" | "ansi256" | "mono-urgent" => (80, 24),
        "fallback" => (60, 18),
        "full" | "states" | "projection" | "reduced" | "pose-idle" | "pose-work" | "pose-wait"
        | "pose-urgent" | "pose-celebrate" | "motion-start" | "motion-settled" => (100, 30),
        _ => (100, 30),
    };
    match mode.as_str() {
        "states" => {
            reduce_app(&mut app, UiInput::Key(UiKey::OpenStates), &view);
        }
        "projection" => {
            reduce_app(&mut app, UiInput::Key(UiKey::OpenProjection), &view);
        }
        "permission" => {
            view.pending_permission = Some(sample_permission()?);
            view.runtime.health = RuntimeHealth::Degraded;
            view.execution.phase = ExecutionPhase::WaitingPermission;
            view.execution.summary = DisplayText::from_untrusted("等待会长裁定权限契约");
            world.apply(&StageEvent::AttentionTier {
                tier: tsukumo_theater::AttentionTier::Urgent,
            });
            world.apply(&StageEvent::ActorPose {
                pose: ActorPose::Upset,
                attribution: attribution.clone(),
            });
            world.apply(&StageEvent::Bubble {
                text: "会长，这份契约需要裁定。".into(),
                attribution: attribution.clone(),
            });
        }
        _ => {}
    }
    if mode == "motion-settled" {
        while world.tick() {}
    }
    if matches!(mode.as_str(), "pose-urgent" | "mono-urgent") {
        world.apply(&StageEvent::AttentionTier {
            tier: tsukumo_theater::AttentionTier::Urgent,
        });
    }
    let capability = match mode.as_str() {
        "mono" | "mono-urgent" => ColorCapability::Monochrome,
        "ansi256" => ColorCapability::Ansi256,
        _ => ColorCapability::TrueColor,
    };
    let buffer = render_product_frame(&pack, &world, &view, &app, width, height, capability);
    let output = if ansi {
        buffer_to_ansi(&buffer)
    } else {
        buffer_to_string(&buffer)
    };
    print!("{output}");
    Ok(())
}

fn sample_view() -> ProductView {
    let states = vec![
        StateView {
            id: StateId::new("state-focus"),
            value: DisplayText::from_untrusted("会长偏好先给结论，再列证据。"),
            scope: DisplayText::from_untrusted("relationship"),
            strength: DisplayText::from_untrusted("confirmed"),
            status: StateStatus::Active,
            source_events: vec![EventId::new("event-41")],
            source_event_total: 1,
        },
        StateView {
            id: StateId::new("state-workspace"),
            value: DisplayText::from_untrusted("当前工作区使用 GNU Rust 工具链。"),
            scope: DisplayText::from_untrusted("workspace"),
            strength: DisplayText::from_untrusted("observed"),
            status: StateStatus::Active,
            source_events: vec![EventId::new("event-42")],
            source_event_total: 1,
        },
    ];
    let projection = ProjectionView {
        projection_id: ProjectionId::new("projection-17"),
        checkpoint_id: CheckpointId::new("checkpoint-9"),
        projection_version: 4,
        renderer_version: 1,
        checkpoint_version: Some(2),
        selected_refs: states
            .iter()
            .map(|state| ProjectionStateRefView {
                state_id: state.id.clone(),
                version: 1,
            })
            .collect(),
        omissions: vec![DisplayText::from_untrusted("过期的临时偏好")],
        selected_total: 2,
        omissions_total: 1,
        budget_used: 318,
        budget_limit: 512,
    };
    let mut view = ProductView {
        states,
        projection: Some(projection),
        ..ProductView::default()
    };
    view.runtime.health = RuntimeHealth::Ready;
    view.runtime.source_spirit_id = Some(SpiritId::new("runtime-spirit"));
    view.runtime.detail = DisplayText::from_untrusted("codex/acp ready");
    view.execution.phase = ExecutionPhase::Running;
    view.execution.summary = DisplayText::from_untrusted("正在执行工作区测试");
    view.handoff.selected_count = 2;
    view.handoff.omitted_count = 1;
    view.handoff.budget_used = 318;
    view.handoff.budget_limit = 512;
    view
}

fn sample_permission() -> Result<PermissionView, Box<dyn Error>> {
    Ok(PermissionView {
        id: UiPermissionId::try_from("permission-17")?,
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted("cargo test --workspace"),
        cwd: PermissionEvidenceText::from_untrusted("D:/WorkSpace/tsukumo"),
        risk_reasons: vec![
            PermissionEvidenceText::from_untrusted("会写入 target 构建产物"),
            PermissionEvidenceText::from_untrusted("预计持续约十秒"),
        ],
        runtime: DisplayText::from_untrusted("codex/acp"),
    })
}
