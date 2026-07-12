//! Runtime and execution status reduction for the product read model.

use tsukumo_kernel::{
    KernelEventPayload, OutcomeStatus, RuntimeBinding, RuntimeKind, RuntimeMode, RuntimePhase,
};
use tsukumo_soul::PersistedEvent;
use tsukumo_theater::{DisplayText, ExecutionPhase, ProductView, RuntimeHealth};

pub(super) fn apply_runtime(
    view: &mut ProductView,
    latest_status: Option<&PersistedEvent>,
    chronicle_has_events: bool,
) {
    // Runtime facts are reduced from one coherent event envelope.
    if let Some(item) = latest_status {
        let event = &item.event;
        view.runtime.binding = event.runtime.clone();
        view.runtime.source_spirit_id = Some(event.spirit_id.clone());
        view.runtime.detail = DisplayText::from_untrusted(
            event
                .runtime
                .as_ref()
                .map_or("等待运行时连接", runtime_label),
        );
        view.execution.execution_id = event.execution_id.clone();
        match &event.payload {
            KernelEventPayload::Outcome {
                status, summary, ..
            } => {
                let (health, phase, fallback) = outcome_status(*status);
                view.runtime.health = health;
                view.execution.phase = phase;
                view.execution.summary = DisplayText::from_untrusted(
                    summary.as_ref().map_or(fallback, |text| text.as_str()),
                );
            }
            KernelEventPayload::RuntimeLifecycle { phase } => {
                let (health, execution, summary) = lifecycle_status(*phase);
                view.runtime.health = health;
                view.execution.phase = execution;
                view.execution.summary = DisplayText::from_untrusted(summary);
            }
            _ => {}
        }
    } else if chronicle_has_events {
        view.runtime.health = RuntimeHealth::Ready;
        view.execution.phase = ExecutionPhase::Idle;
        view.execution.summary = DisplayText::from_untrusted("Chronicle 已载入");
    } else {
        view.runtime.health = RuntimeHealth::Offline;
        view.execution.phase = ExecutionPhase::Idle;
        view.execution.summary = DisplayText::from_untrusted("当前无运行时事件");
    }
}
pub(super) fn runtime_label(binding: &RuntimeBinding) -> &'static str {
    match (binding.kind, binding.mode) {
        (RuntimeKind::Builtin, RuntimeMode::OwnedProcess) => "builtin/owned",
        (RuntimeKind::Builtin, RuntimeMode::Fixture) => "builtin/fixture",
        (RuntimeKind::Builtin, RuntimeMode::Synthetic) => "builtin/synthetic",
        (RuntimeKind::Acp, RuntimeMode::OwnedProcess) => "acp/owned",
        (RuntimeKind::Acp, RuntimeMode::Fixture) => "acp/fixture",
        (RuntimeKind::Acp, RuntimeMode::Synthetic) => "acp/synthetic",
        (RuntimeKind::ClaudeCli, RuntimeMode::OwnedProcess) => "claude/owned",
        (RuntimeKind::ClaudeCli, RuntimeMode::Fixture) => "claude/fixture",
        (RuntimeKind::ClaudeCli, RuntimeMode::Synthetic) => "claude/synthetic",
        (RuntimeKind::CodexCli, RuntimeMode::OwnedProcess) => "codex/owned",
        (RuntimeKind::CodexCli, RuntimeMode::Fixture) => "codex/fixture",
        (RuntimeKind::CodexCli, RuntimeMode::Synthetic) => "codex/synthetic",
    }
}

const fn lifecycle_status(phase: RuntimePhase) -> (RuntimeHealth, ExecutionPhase, &'static str) {
    match phase {
        RuntimePhase::Starting | RuntimePhase::Started | RuntimePhase::Stopping => (
            RuntimeHealth::Busy,
            ExecutionPhase::Running,
            "运行时正在执行",
        ),
        RuntimePhase::Completed => (
            RuntimeHealth::Ready,
            ExecutionPhase::Completed,
            "运行时已完成",
        ),
        RuntimePhase::Failed | RuntimePhase::Cancelled => (
            RuntimeHealth::Degraded,
            ExecutionPhase::Failed,
            "运行时已停止",
        ),
    }
}

const fn outcome_status(status: OutcomeStatus) -> (RuntimeHealth, ExecutionPhase, &'static str) {
    match status {
        OutcomeStatus::Succeeded => (
            RuntimeHealth::Ready,
            ExecutionPhase::Completed,
            "执行已完成",
        ),
        OutcomeStatus::PermissionDenied
        | OutcomeStatus::SafetyUnsupported
        | OutcomeStatus::Failed
        | OutcomeStatus::Cancelled
        | OutcomeStatus::Degraded
        | OutcomeStatus::TimedOut
        | OutcomeStatus::MalformedOutput
        | OutcomeStatus::NonZeroExit
        | OutcomeStatus::LaunchFailed => (
            RuntimeHealth::Degraded,
            ExecutionPhase::Failed,
            "执行需要检查",
        ),
    }
}
