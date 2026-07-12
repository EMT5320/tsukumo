//! Chronicle and Soul projection into bounded product and theater snapshots.

mod permissions;
mod projection;
mod runtime;

use permissions::{permission_view, rebuild_permissions};
use projection::{apply_projection, latest_projection};
use runtime::{apply_runtime, runtime_label};
use std::collections::HashMap;
use tsukumo_kernel::{
    ExecutionId, KernelEvent, KernelEventPayload, QuestId, RuntimeBinding, SessionId, SpiritId,
};
use tsukumo_soul::{
    EvidenceStrength, OperatingSystem, ProjectionReceipt, SoulStore, StateRecord, StateSubject,
};
use tsukumo_theater::{
    drive_kernel_event, DirectorContext, DisplayText, NoticeLevel, NoticeView, ProductView,
    StageWorld, StateStatus, StateView, UiPermissionId,
};

use crate::{PermissionController, PermissionRequest, ProductControllerError};

const RECENT_EVENT_LIMIT: usize = 1_000;
const PERMISSION_EVENT_LIMIT: usize = 4_096;
const PERMISSION_EVENT_BYTE_LIMIT: usize = 32 * 1024 * 1024;
const STATE_VIEW_CAP: usize = 256;
const STATE_EVIDENCE_CAP: usize = 64;
const NOTICE_VIEW_CAP: usize = 8;

#[derive(Debug, Clone)]
pub(super) struct UiCoordinates {
    pub quest_id: QuestId,
    pub session_id: SessionId,
    pub spirit_id: Option<SpiritId>,
    pub execution_id: Option<ExecutionId>,
    pub runtime: Option<RuntimeBinding>,
}

impl UiCoordinates {
    pub(super) fn fallback() -> Self {
        Self {
            quest_id: QuestId::new("quest-tui"),
            session_id: SessionId::new("session-tui"),
            spirit_id: None,
            execution_id: None,
            runtime: None,
        }
    }

    pub(super) fn from_event(event: &KernelEvent) -> Self {
        Self {
            quest_id: event.quest_id.clone(),
            session_id: event.session_id.clone(),
            spirit_id: Some(event.spirit_id.clone()),
            execution_id: event.execution_id.clone(),
            runtime: event.runtime.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct PendingPermission {
    pub sequence: i64,
    pub ui_id: UiPermissionId,
    pub request: PermissionRequest,
    pub receipt: Option<ProjectionReceipt>,
    pub coordinates: UiCoordinates,
}

pub(super) struct Assembly {
    pub snapshot: super::ProductSnapshot,
    pub permissions: PermissionController,
    pub pending: HashMap<UiPermissionId, PendingPermission>,
    pub coordinates: UiCoordinates,
}

pub(super) fn assemble(
    store: &SoulStore,
    director: &DirectorContext,
    walk_bounds: (i32, i32),
    notices: &[tsukumo_theater::NoticeView],
) -> Result<Assembly, ProductControllerError> {
    let events = store.replay_recent_events(RECENT_EVENT_LIMIT)?;
    let coordinates = events
        .last()
        .map(|event| UiCoordinates::from_event(&event.event))
        .unwrap_or_else(UiCoordinates::fallback);
    let mut world = StageWorld::new()
        .with_log_cap(32)
        .with_walk_bounds(walk_bounds.0, walk_bounds.1);
    for persisted in &events {
        drive_kernel_event(&mut world, &persisted.event, director);
    }
    world.ensure_placeholder(director.actor_id.clone());

    let (states, states_truncated) = state_views(store)?;
    let mut bounded_notices = notices.to_vec();
    if states_truncated {
        if bounded_notices.len() >= NOTICE_VIEW_CAP {
            bounded_notices.remove(0);
        }
        bounded_notices.push(NoticeView {
            level: NoticeLevel::Warning,
            text: DisplayText::from_untrusted(&format!("状态视图已截取前 {STATE_VIEW_CAP} 项。")),
        });
    }
    let mut view = ProductView {
        states,
        notices: bounded_notices,
        ..ProductView::default()
    };
    let checkpoint_event = store.latest_checkpoint_event()?;
    apply_latest_checkpoint(&mut view, checkpoint_event.as_ref());
    let projection = latest_projection(store)?;
    let checkpoint_version = projection
        .as_ref()
        .map(|receipt| store.checkpoint(&receipt.checkpoint_id))
        .transpose()?
        .flatten()
        .map(|checkpoint| checkpoint.version);
    apply_projection(&mut view, projection.as_ref(), checkpoint_version);
    let runtime_status = store.latest_runtime_status_event()?;
    apply_runtime(&mut view, runtime_status.as_ref(), !events.is_empty());

    let permission_events =
        store.replay_permission_events(PERMISSION_EVENT_LIMIT, PERMISSION_EVENT_BYTE_LIMIT)?;
    let (permissions, pending) = rebuild_permissions(store, &permission_events)?;
    if let Some(current) = pending.values().max_by_key(|item| item.sequence) {
        view.runtime.binding = current.coordinates.runtime.clone();
        view.runtime.source_spirit_id = current.coordinates.spirit_id.clone();
        view.runtime.detail = DisplayText::from_untrusted(
            current
                .coordinates
                .runtime
                .as_ref()
                .map_or("等待运行时连接", runtime_label),
        );
        view.execution.execution_id = current.coordinates.execution_id.clone();
        view.pending_permission = Some(permission_view(current)?);
        view.runtime.health = tsukumo_theater::RuntimeHealth::Degraded;
        view.execution.phase = tsukumo_theater::ExecutionPhase::WaitingPermission;
        view.execution.summary = DisplayText::from_untrusted(current.request.reason.as_str());
    }
    Ok(Assembly {
        snapshot: super::ProductSnapshot {
            view,
            world,
            revision: events.last().map_or(0, |event| event.sequence),
        },
        permissions,
        pending,
        coordinates,
    })
}

fn apply_latest_checkpoint(view: &mut ProductView, latest: Option<&tsukumo_soul::PersistedEvent>) {
    let checkpoint = latest.and_then(|item| match &item.event.payload {
        KernelEventPayload::CheckpointCreated {
            checkpoint_id,
            version,
        } => Some((checkpoint_id.clone(), *version)),
        _ => None,
    });
    if let Some((checkpoint_id, version)) = checkpoint {
        view.handoff.checkpoint_id = Some(checkpoint_id);
        view.handoff.version = Some(version);
    }
}
fn state_views(store: &SoulStore) -> Result<(Vec<StateView>, bool), ProductControllerError> {
    let mut states = store.list_active_states_limited(STATE_VIEW_CAP.saturating_add(1))?;
    let truncated = states.len() > STATE_VIEW_CAP;
    states.truncate(STATE_VIEW_CAP);
    let views = states
        .into_iter()
        .map(|state| {
            let scope = scope_label(&state);
            let source_event_total = state.evidence_refs.len();
            let mut source_events = state.evidence_refs;
            source_events.truncate(STATE_EVIDENCE_CAP);
            Ok(StateView {
                id: state.state_id,
                value: DisplayText::from_untrusted(state.content.as_str()),
                scope: DisplayText::from_untrusted(&scope),
                strength: DisplayText::from_untrusted(strength_label(state.strength)),
                status: StateStatus::Active,
                source_events,
                source_event_total,
            })
        })
        .collect::<Result<Vec<_>, ProductControllerError>>()?;
    Ok((views, truncated))
}

fn scope_label(state: &StateRecord) -> String {
    let subject = match &state.scope.subject {
        StateSubject::Owner { owner_id } => format!("owner:{owner_id}"),
        StateSubject::Workspace { workspace_id } => format!("workspace:{workspace_id}"),
        StateSubject::Spirit { spirit_id } => format!("spirit:{spirit_id}"),
        StateSubject::Relationship {
            owner_id,
            spirit_id,
        } => format!("relationship:{owner_id}<->{spirit_id}"),
        StateSubject::Unresolved => "unresolved".to_owned(),
    };
    let applicability = &state.scope.applicability;
    let mut coordinates = vec![subject];
    if let Some(workspace) = &applicability.workspace {
        coordinates.push(format!("workspace={workspace}"));
    }
    if let Some(operating_system) = applicability.operating_system {
        coordinates.push(format!("os={}", operating_system_label(operating_system)));
    }
    if !applicability.task_tags.is_empty() {
        coordinates.push(format!("task={}", applicability.task_tags.join(",")));
    }
    if !applicability.language_tags.is_empty() {
        coordinates.push(format!("lang={}", applicability.language_tags.join(",")));
    }
    if !applicability.required_capabilities.is_empty() {
        coordinates.push(format!(
            "capability={}",
            applicability.required_capabilities.join(",")
        ));
    }
    coordinates.join(" | ")
}

const fn operating_system_label(operating_system: OperatingSystem) -> &'static str {
    match operating_system {
        OperatingSystem::Windows => "windows",
        OperatingSystem::Linux => "linux",
        OperatingSystem::Macos => "macos",
    }
}

const fn strength_label(strength: EvidenceStrength) -> &'static str {
    match strength {
        EvidenceStrength::Imported => "imported",
        EvidenceStrength::Inferred => "inferred",
        EvidenceStrength::Repeated => "repeated",
        EvidenceStrength::Explicit => "explicit",
    }
}
