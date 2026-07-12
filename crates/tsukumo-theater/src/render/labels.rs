//! Stable product labels shared by terminal chrome.

use crate::app::{ExecutionPhase, RuntimeHealth};
use crate::stage::AttentionTier;

pub(super) const fn runtime_health_label(health: RuntimeHealth) -> &'static str {
    match health {
        RuntimeHealth::Offline => "离线",
        RuntimeHealth::Ready => "就绪",
        RuntimeHealth::Busy => "运行中",
        RuntimeHealth::Degraded => "降级",
    }
}

pub(super) const fn execution_phase_label(phase: ExecutionPhase) -> &'static str {
    match phase {
        ExecutionPhase::Idle => "空闲",
        ExecutionPhase::Running => "执行中",
        ExecutionPhase::WaitingPermission => "等待权限",
        ExecutionPhase::Completed => "已完成",
        ExecutionPhase::Failed => "失败",
    }
}

pub(super) const fn attention_label(tier: AttentionTier) -> &'static str {
    match tier {
        AttentionTier::Ambient => "环境",
        AttentionTier::Focus => "专注",
        AttentionTier::Urgent => "紧急",
    }
}
