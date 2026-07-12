//! Bounded, redacted data accepted by the terminal product surface.

use serde::Serialize;
use thiserror::Error;
use tsukumo_kernel::{redact_sensitive_text, PermissionDecision, StateId};

const MAX_DISPLAY_CHARS: usize = 512;
const MAX_PERMISSION_ID_CHARS: usize = 128;
const MAX_PERMISSION_EVIDENCE_CHARS: usize = 65_536;

/// Redacted copy with a stable terminal rendering bound.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct DisplayText(String);

impl DisplayText {
    pub fn from_untrusted(input: &str) -> Self {
        let redacted = redact_sensitive_text(input);
        if redacted.chars().count() <= MAX_DISPLAY_CHARS {
            Self(redacted)
        } else {
            Self(redacted.chars().take(509).collect::<String>() + "...")
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Redacted permission evidence that preserves the full kernel text budget for paging.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct PermissionEvidenceText(String);

impl PermissionEvidenceText {
    pub fn from_untrusted(input: &str) -> Self {
        let redacted = redact_sensitive_text(input);
        if redacted.chars().count() <= MAX_PERMISSION_EVIDENCE_CHARS {
            Self(redacted)
        } else {
            let retained = MAX_PERMISSION_EVIDENCE_CHARS.saturating_sub(3);
            Self(redacted.chars().take(retained).collect::<String>() + "...")
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// Stable UI correlation ID for one pending permission.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct UiPermissionId(String);

impl UiPermissionId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for UiPermissionId {
    type Error = ViewModelError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let count = value.chars().count();
        if count == 0 || count > MAX_PERMISSION_ID_CHARS || value.chars().any(char::is_control) {
            return Err(ViewModelError::InvalidPermissionId);
        }
        Ok(Self(value.to_owned()))
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum ViewModelError {
    #[error("permission id must be non-empty, bounded, and free of control characters")]
    InvalidPermissionId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Workshop,
    StateInspector {
        selected: usize,
    },
    ProjectionInspector,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    Refresh,
    RevokeState(StateId),
    DecidePermission(UiPermissionId, PermissionDecision),
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiKey {
    OpenWorkshop,
    OpenStates,
    OpenProjection,
    Up,
    Down,
    PreviousPage,
    NextPage,
    Revoke,
    Refresh,
    AllowOnce,
    AllowSession,
    Deny,
    Escape,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiInput {
    Key(UiKey),
    Tick,
    Resize { width: u16, height: u16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppState {
    pub(super) screen: Screen,
    pub(super) reduced_motion: bool,
    pub(super) animation_frame: u64,
    pub(super) permission_page: usize,
    pub(super) inspector_page: usize,
    pub(super) dirty: bool,
}

impl AppState {
    pub const fn new(reduced_motion: bool) -> Self {
        Self {
            screen: Screen::Workshop,
            reduced_motion,
            animation_frame: 0,
            permission_page: 0,
            inspector_page: 0,
            dirty: true,
        }
    }

    pub const fn screen(&self) -> Screen {
        self.screen
    }

    pub const fn animation_frame(&self) -> u64 {
        self.animation_frame
    }

    pub const fn permission_page(&self) -> usize {
        self.permission_page
    }

    pub const fn inspector_page(&self) -> usize {
        self.inspector_page
    }

    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub const fn reduced_motion(&self) -> bool {
        self.reduced_motion
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Starts each distinct permission request at its first evidence page.
    pub fn reset_permission_page(&mut self) {
        self.permission_page = 0;
    }

    /// Keeps permission evidence navigation valid after a host-owned refresh.
    pub fn clamp_permission_page(&mut self, reason_count: usize) {
        self.permission_page = self.permission_page.min(reason_count.saturating_sub(1));
    }

    /// Keeps a bounded detail page valid after a host-owned refresh.
    pub fn clamp_inspector_page(&mut self, page_count: usize) {
        self.inspector_page = self.inspector_page.min(page_count.saturating_sub(1));
    }

    /// Keeps state navigation valid after a host-owned refresh or revocation.
    pub fn clamp_selection(&mut self, state_count: usize) {
        if let Screen::StateInspector { selected } = &mut self.screen {
            *selected = (*selected).min(state_count.saturating_sub(1));
        }
    }
}
