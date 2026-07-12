//! Pending permission reconstruction and bounded modal view assembly.

use super::projection::receipt_for_execution;
use super::runtime::runtime_label;
use super::{PendingPermission, UiCoordinates};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tsukumo_kernel::{KernelEvent, KernelEventPayload, VendorEventRef};
use tsukumo_soul::{PersistedEvent, SoulStore};
use tsukumo_theater::{DisplayText, PermissionEvidenceText, PermissionView, UiPermissionId};

use crate::{
    PermissionController, PermissionRegistration, PermissionRequest, PermissionScope,
    ProductControllerError,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ScopedPermissionKey {
    scope: PermissionScope,
    vendor_request: VendorEventRef,
}

impl ScopedPermissionKey {
    fn new(scope: &PermissionScope, vendor_request: &VendorEventRef) -> Self {
        Self {
            scope: scope.clone(),
            vendor_request: vendor_request.clone(),
        }
    }
}

pub(super) fn rebuild_permissions(
    store: &SoulStore,
    events: &[PersistedEvent],
) -> Result<
    (
        PermissionController,
        HashMap<UiPermissionId, PendingPermission>,
    ),
    ProductControllerError,
> {
    let mut controller = PermissionController::default();
    let mut by_request = HashMap::<ScopedPermissionKey, PendingPermission>::new();
    for persisted in events {
        match &persisted.event.payload {
            payload @ KernelEventPayload::PermissionRequested { .. } => {
                let Some(scope) = permission_scope(&persisted.event) else {
                    continue;
                };
                let execution_id = scope.execution_id.clone();
                let request = PermissionRequest::from_payload(scope, payload)?;
                let key = ScopedPermissionKey::new(&request.scope, &request.vendor_request);
                match controller.register(request.clone())? {
                    PermissionRegistration::Pending => {
                        let ui_id = permission_id(&key)?;
                        by_request.insert(
                            key,
                            PendingPermission {
                                sequence: persisted.sequence,
                                ui_id,
                                receipt: receipt_for_execution(store, &execution_id)?,
                                coordinates: UiCoordinates::from_event(&persisted.event),
                                request,
                            },
                        );
                    }
                    PermissionRegistration::Covered(_) => {}
                }
            }
            // Tail replay can contain a decision whose older request fell outside the limit.
            KernelEventPayload::PermissionDecided {
                vendor_request,
                decision,
            } => {
                let Some(scope) = permission_scope(&persisted.event) else {
                    continue;
                };
                let key = ScopedPermissionKey::new(&scope, vendor_request);
                if by_request.remove(&key).is_some() {
                    controller.decide_scoped(&scope, vendor_request, *decision)?;
                }
            }
            _ => {}
        }
    }
    let pending = by_request
        .into_values()
        .map(|item| (item.ui_id.clone(), item))
        .collect();
    Ok((controller, pending))
}

fn permission_scope(event: &KernelEvent) -> Option<PermissionScope> {
    Some(PermissionScope::new(
        event.execution_id.clone()?,
        event.session_id.clone(),
        event.runtime.clone()?,
    ))
}

fn permission_id(key: &ScopedPermissionKey) -> Result<UiPermissionId, ProductControllerError> {
    let mut digest = Sha256::new();
    digest.update(key.scope.execution_id.as_str().as_bytes());
    digest.update([0]);
    digest.update(key.scope.session_id.as_str().as_bytes());
    digest.update([0]);
    digest.update(serde_json::to_vec(&key.scope.runtime)?);
    digest.update([0]);
    digest.update(key.vendor_request.namespace.as_bytes());
    digest.update([0]);
    digest.update(key.vendor_request.id.as_bytes());
    let bytes = digest.finalize();
    let mut value = String::from("permission-");
    for byte in bytes.iter().take(12) {
        use std::fmt::Write;
        let _ = write!(&mut value, "{byte:02x}");
    }
    Ok(UiPermissionId::try_from(value.as_str())?)
}

pub(super) fn permission_view(
    pending: &PendingPermission,
) -> Result<PermissionView, ProductControllerError> {
    let request = &pending.request;
    let arguments = request
        .arguments
        .as_ref()
        .map(|value| serde_json::to_string(value.as_value()))
        .transpose()?
        .unwrap_or_else(|| "无".to_owned());
    let cwd = request
        .cwd
        .as_ref()
        .map_or("未提供", |value| value.as_str());
    let mut reasons = request
        .risk_reasons
        .iter()
        .map(|reason| PermissionEvidenceText::from_untrusted(reason.as_str()))
        .collect::<Vec<_>>();
    if reasons.is_empty() {
        reasons.push(PermissionEvidenceText::from_untrusted(
            request.reason.as_str(),
        ));
    }
    Ok(PermissionView {
        id: pending.ui_id.clone(),
        tool: DisplayText::from_untrusted(&request.tool),
        arguments: PermissionEvidenceText::from_untrusted(&arguments),
        cwd: PermissionEvidenceText::from_untrusted(cwd),
        risk_reasons: reasons,
        runtime: DisplayText::from_untrusted(runtime_label(&request.scope.runtime)),
    })
}
