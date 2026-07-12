//! Bounded product read models assembled by host composition.

use super::model::{DisplayText, UiPermissionId};
use serde::Serialize;
use tsukumo_kernel::{
    CheckpointId, EventId, ExecutionId, ProjectionId, RuntimeBinding, SpiritId, StateId,
};

const STATE_EVIDENCE_PAGE_ITEMS: usize = 3;
// Six entries fit the minimum compact inspector with metadata and a truncation receipt.
const PROJECTION_PAGE_ITEMS: usize = 6;
const PERMISSION_RISK_PAGE_CHARS: usize = 80;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeHealth {
    #[default]
    Offline,
    Ready,
    Busy,
    Degraded,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RuntimeStatusView {
    pub binding: Option<RuntimeBinding>,
    pub source_spirit_id: Option<SpiritId>,
    pub health: RuntimeHealth,
    pub detail: DisplayText,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPhase {
    #[default]
    Idle,
    Running,
    WaitingPermission,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ExecutionStatusView {
    pub execution_id: Option<ExecutionId>,
    pub phase: ExecutionPhase,
    pub summary: DisplayText,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct HandoffStatusView {
    pub checkpoint_id: Option<CheckpointId>,
    pub projection_id: Option<ProjectionId>,
    pub version: Option<u64>,
    pub selected_count: usize,
    pub omitted_count: usize,
    pub budget_used: usize,
    pub budget_limit: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StateStatus {
    Active,
    Superseded,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StateView {
    pub id: StateId,
    pub value: DisplayText,
    pub scope: DisplayText,
    pub strength: DisplayText,
    pub status: StateStatus,
    pub source_events: Vec<EventId>,
    pub source_event_total: usize,
}

impl StateView {
    /// Counts stable evidence pages so every retained source event is inspectable.
    pub fn evidence_page_count(&self) -> usize {
        self.source_events
            .len()
            .max(1)
            .div_ceil(STATE_EVIDENCE_PAGE_ITEMS)
    }

    pub(crate) fn evidence_page(&self, selected: usize) -> &[EventId] {
        let page = selected.min(self.evidence_page_count().saturating_sub(1));
        let start = page.saturating_mul(STATE_EVIDENCE_PAGE_ITEMS);
        let end = start
            .saturating_add(STATE_EVIDENCE_PAGE_ITEMS)
            .min(self.source_events.len());
        &self.source_events[start..end]
    }

    pub(crate) const fn evidence_page_size() -> usize {
        STATE_EVIDENCE_PAGE_ITEMS
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectionStateRefView {
    pub state_id: StateId,
    pub version: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectionView {
    pub projection_id: ProjectionId,
    pub checkpoint_id: CheckpointId,
    pub projection_version: u64,
    pub renderer_version: u64,
    pub checkpoint_version: Option<u64>,
    pub selected_refs: Vec<ProjectionStateRefView>,
    pub omissions: Vec<DisplayText>,
    pub selected_total: usize,
    pub omissions_total: usize,
    pub budget_used: usize,
    pub budget_limit: usize,
}

impl ProjectionView {
    /// Counts stable pages across retained selected and omitted receipt entries.
    pub fn entry_page_count(&self) -> usize {
        self.retained_entry_count()
            .max(1)
            .div_ceil(PROJECTION_PAGE_ITEMS)
    }

    pub(crate) fn retained_entry_count(&self) -> usize {
        self.selected_refs.len() + self.omissions.len()
    }

    pub(crate) fn entry_page_bounds(&self, selected: usize) -> std::ops::Range<usize> {
        let page = selected.min(self.entry_page_count().saturating_sub(1));
        let start = page.saturating_mul(PROJECTION_PAGE_ITEMS);
        let end = start
            .saturating_add(PROJECTION_PAGE_ITEMS)
            .min(self.retained_entry_count());
        start..end
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PermissionView {
    pub id: UiPermissionId,
    pub tool: DisplayText,
    pub arguments: super::model::PermissionEvidenceText,
    pub cwd: super::model::PermissionEvidenceText,
    pub risk_reasons: Vec<super::model::PermissionEvidenceText>,
    pub runtime: DisplayText,
}

pub(crate) struct PermissionEvidencePage {
    pub label: &'static str,
    pub text: String,
    pub item_index: usize,
    pub item_count: usize,
    pub part_index: usize,
    pub part_count: usize,
}

impl PermissionView {
    /// Counts pages occupied by risk reasons alone for stable review receipts.
    pub fn risk_page_count(&self) -> usize {
        self.risk_reasons
            .iter()
            .map(|reason| permission_part_count(reason.as_str()))
            .sum()
    }

    /// Counts every risk, argument, and directory page exposed by the modal.
    pub fn evidence_page_count(&self) -> usize {
        self.risk_page_count()
            .saturating_add(permission_part_count(self.arguments.as_str()))
            .saturating_add(permission_part_count(self.cwd.as_str()))
            .max(1)
    }

    pub(crate) fn evidence_page(&self, selected: usize) -> PermissionEvidencePage {
        let selected = selected.min(self.evidence_page_count().saturating_sub(1));
        let mut offset = 0;
        for (item_index, reason) in self.risk_reasons.iter().enumerate() {
            let part_count = permission_part_count(reason.as_str());
            if selected < offset + part_count {
                return permission_page(
                    "风险依据",
                    reason.as_str(),
                    item_index,
                    self.risk_reasons.len(),
                    selected - offset,
                );
            }
            offset += part_count;
        }

        let argument_parts = permission_part_count(self.arguments.as_str());
        if selected < offset + argument_parts {
            return permission_page("参数", self.arguments.as_str(), 0, 1, selected - offset);
        }
        offset += argument_parts;
        permission_page("目录", self.cwd.as_str(), 0, 1, selected - offset)
    }
}

fn permission_page(
    label: &'static str,
    value: &str,
    item_index: usize,
    item_count: usize,
    part_index: usize,
) -> PermissionEvidencePage {
    let part_count = permission_part_count(value);
    let text = value
        .chars()
        .skip(part_index * PERMISSION_RISK_PAGE_CHARS)
        .take(PERMISSION_RISK_PAGE_CHARS)
        .collect();
    PermissionEvidencePage {
        label,
        text,
        item_index,
        item_count,
        part_index,
        part_count,
    }
}

fn permission_part_count(value: &str) -> usize {
    value
        .chars()
        .count()
        .max(1)
        .div_ceil(PERMISSION_RISK_PAGE_CHARS)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NoticeView {
    pub level: NoticeLevel,
    pub text: DisplayText,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ProductView {
    pub runtime: RuntimeStatusView,
    pub execution: ExecutionStatusView,
    pub handoff: HandoffStatusView,
    pub states: Vec<StateView>,
    pub projection: Option<ProjectionView>,
    pub pending_permission: Option<PermissionView>,
    pub notices: Vec<NoticeView>,
}
