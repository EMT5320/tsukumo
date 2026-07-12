//! Host-owned product read models and durable UI action routing.

mod actions;
mod read_model;

use read_model::{assemble, PendingPermission, UiCoordinates};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tsukumo_kernel::{PermissionDecision, StateId};
use tsukumo_soul::{SoulError, SoulStore};
use tsukumo_theater::{
    DirectorContext, DisplayText, NoticeLevel, NoticeView, ProductView, StageWorld, UiAction,
    UiPermissionId, ValidatedPresentationPack, ViewModelError,
};

use crate::local_path::LocalDirectoryGuard;
use crate::{ClockError, HostError, PermissionController, SafetyError, SystemClock};

const NOTICE_CAP: usize = 8;

#[derive(Debug, Clone)]
pub struct ProductSnapshot {
    pub view: ProductView,
    pub world: StageWorld,
    pub revision: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductControl {
    Continue,
    Quit,
}

/// Host authority consumed by the terminal loop without exposing storage handles to theater.
pub trait ProductController {
    fn refresh(&mut self) -> Result<ProductSnapshot, ProductControllerError>;
    fn apply(&mut self, action: UiAction) -> Result<ProductControl, ProductControllerError>;
}

pub struct HostProductController {
    store: SoulStore,
    // Keep path and critical-file handles alive until after the SQLite connection closes.
    _data_guard: LocalDirectoryGuard,
    director: DirectorContext,
    walk_bounds: (i32, i32),
    clock: SystemClock,
    permissions: PermissionController,
    pending: HashMap<UiPermissionId, PendingPermission>,
    coordinates: UiCoordinates,
    notices: Vec<NoticeView>,
    event_counter: u64,
}

impl HostProductController {
    /// Opens the durable authority before the terminal enters raw or alternate-screen mode.
    pub fn open(
        data_dir: impl AsRef<Path>,
        pack: &ValidatedPresentationPack,
    ) -> Result<Self, ProductControllerError> {
        let bounds = pack.scene().walk_bounds;
        let mut data_guard = LocalDirectoryGuard::prepare(data_dir.as_ref())
            .map_err(|error| ProductControllerError::LocalPath(error.to_string()))?;
        data_guard
            .validate_tree()
            .map_err(|error| ProductControllerError::LocalPath(error.to_string()))?;
        data_guard
            .ensure_directory(Path::new("skills"))
            .and_then(|()| data_guard.ensure_guarded_file(Path::new("soul.db"), b""))
            .and_then(|()| data_guard.ensure_guarded_file(Path::new("soul.db-journal"), b""))
            .and_then(|()| data_guard.ensure_guarded_file(Path::new("soul.db-wal"), b""))
            .and_then(|()| data_guard.ensure_guarded_file(Path::new("soul.db-shm"), b""))
            .and_then(|()| data_guard.ensure_guarded_file(Path::new("MEMORY.md"), b"# MEMORY\n\n"))
            .and_then(|()| data_guard.ensure_guarded_file(Path::new("USER.md"), b"# USER\n\n"))
            .map_err(|error| ProductControllerError::LocalPath(error.to_string()))?;
        let data_dir = data_guard.root().to_path_buf();
        Ok(Self {
            store: SoulStore::open(&data_dir)?,
            _data_guard: data_guard,
            director: DirectorContext::from_pack(pack),
            walk_bounds: (i32::from(bounds.min_x), i32::from(bounds.max_x)),
            clock: SystemClock,
            permissions: PermissionController::default(),
            pending: HashMap::new(),
            coordinates: UiCoordinates::fallback(),
            notices: Vec::new(),
            event_counter: 0,
        })
    }

    fn push_notice(&mut self, level: NoticeLevel, message: &str) {
        if self.notices.len() >= NOTICE_CAP {
            self.notices.remove(0);
        }
        self.notices.push(NoticeView {
            level,
            text: DisplayText::from_untrusted(message),
        });
    }

    fn apply_revoke(&mut self, state_id: StateId) -> Result<(), ProductControllerError> {
        actions::revoke_state(self, &state_id)?;
        self.push_notice(
            NoticeLevel::Info,
            &format!("状态 {state_id} 已撤销并重新读取。"),
        );
        Ok(())
    }

    fn apply_permission(
        &mut self,
        permission_id: UiPermissionId,
        decision: PermissionDecision,
    ) -> Result<(), ProductControllerError> {
        let Some(pending) = self.pending.get(&permission_id).cloned() else {
            self.push_notice(NoticeLevel::Warning, "权限请求已变化，请刷新后重试。");
            return Ok(());
        };
        actions::record_permission(self, &pending, decision)?;
        let label = match decision {
            PermissionDecision::AllowOnce => "仅本次允许",
            PermissionDecision::AllowSession => "本次会话允许",
            PermissionDecision::Deny => "已拒绝",
        };
        self.push_notice(
            NoticeLevel::Info,
            &format!("权限裁定“{label}”已写入 Chronicle；当前运行时桥保持闭合。"),
        );
        Ok(())
    }
}

impl ProductController for HostProductController {
    fn refresh(&mut self) -> Result<ProductSnapshot, ProductControllerError> {
        let assembly = assemble(&self.store, &self.director, self.walk_bounds, &self.notices)?;
        self.permissions = assembly.permissions;
        self.pending = assembly.pending;
        self.coordinates = assembly.coordinates;
        Ok(assembly.snapshot)
    }

    fn apply(&mut self, action: UiAction) -> Result<ProductControl, ProductControllerError> {
        match action {
            UiAction::Refresh => {
                self.push_notice(
                    NoticeLevel::Info,
                    "已从 Chronicle 与 Soul 重新读取产品视图。",
                );
            }
            UiAction::RevokeState(state_id) => self.apply_revoke(state_id)?,
            UiAction::DecidePermission(permission_id, decision) => {
                self.apply_permission(permission_id, decision)?;
            }
            UiAction::Quit => return Ok(ProductControl::Quit),
        }
        Ok(ProductControl::Continue)
    }
}

#[derive(Debug, Error)]
pub enum ProductControllerError {
    #[error("product local path rejected: {0}")]
    LocalPath(String),
    #[error("Soul product state failed: {0}")]
    Soul(#[from] SoulError),
    #[error("product timestamp failed: {0}")]
    Clock(#[from] ClockError),
    #[error("permission controller failed: {0}")]
    Safety(#[from] SafetyError),
    #[error("host permission recording failed: {0}")]
    Host(#[from] HostError),
    #[error("product view validation failed: {0}")]
    View(#[from] ViewModelError),
    #[error("product JSON formatting failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("pending permission has no durable projection receipt")]
    MissingPermissionReceipt,
    #[error("product action has no Chronicle source Spirit")]
    MissingSourceSpirit,
}
