//! Deterministic permission state machine and vendor bridge seam.

use std::collections::{HashMap, HashSet};
use thiserror::Error;
use tsukumo_kernel::{
    ExecutionId, KernelEventPayload, PermissionDecision, PersistedJson, PersistedText,
    RuntimeBinding, SessionId, VendorEventRef,
};

/// Execution/session/runtime coordinates that constrain permission reuse.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PermissionScope {
    pub execution_id: ExecutionId,
    pub session_id: SessionId,
    pub runtime: RuntimeBinding,
}

impl PermissionScope {
    /// Creates the immutable scope attached to one vendor request.
    pub const fn new(
        execution_id: ExecutionId,
        session_id: SessionId,
        runtime: RuntimeBinding,
    ) -> Self {
        Self {
            execution_id,
            session_id,
            runtime,
        }
    }
}

/// Redacted request retained while a human decision is pending.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionRequest {
    pub vendor_request: VendorEventRef,
    pub scope: PermissionScope,
    pub tool: String,
    pub arguments: Option<PersistedJson>,
    pub cwd: Option<PersistedText>,
    pub risk_reasons: Vec<PersistedText>,
    pub reason: PersistedText,
}

impl PermissionRequest {
    /// Converts only a normalized permission payload into controller state.
    pub fn from_payload(
        scope: PermissionScope,
        payload: &KernelEventPayload,
    ) -> Result<Self, SafetyError> {
        let KernelEventPayload::PermissionRequested {
            vendor_request,
            tool,
            arguments,
            cwd,
            risk_reasons,
            reason,
        } = payload
        else {
            return Err(SafetyError::NotPermissionRequest);
        };
        Ok(Self {
            vendor_request: vendor_request.clone(),
            scope,
            tool: tool.clone(),
            arguments: arguments.clone(),
            cwd: cwd.clone(),
            risk_reasons: risk_reasons.clone(),
            reason: reason.clone(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SessionGrant {
    session_id: SessionId,
    runtime: RuntimeBinding,
    tool: String,
}

impl SessionGrant {
    fn from_request(request: &PermissionRequest) -> Self {
        Self {
            session_id: request.scope.session_id.clone(),
            runtime: request.scope.runtime.clone(),
            tool: request.tool.clone(),
        }
    }
}

/// Provenance for one applied permission decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionResolutionSource {
    HumanDecision,
    SessionGrant,
}

/// A request paired with the human decision or prior session grant that resolved it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionResolution {
    pub request: PermissionRequest,
    pub decision: PermissionDecision,
    pub source: PermissionResolutionSource,
}

impl PermissionResolution {
    /// Produces durable decision evidence without carrying request arguments again.
    pub fn into_payload(self) -> KernelEventPayload {
        KernelEventPayload::PermissionDecided {
            vendor_request: self.request.vendor_request,
            decision: self.decision,
        }
    }
}

/// Result of admitting a new vendor permission request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionRegistration {
    Pending,
    Covered(Box<PermissionResolution>),
}

/// Internal identity for one vendor request inside its host-owned execution scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PermissionRequestKey {
    scope: PermissionScope,
    vendor_request: VendorEventRef,
}

impl PermissionRequestKey {
    fn from_request(request: &PermissionRequest) -> Self {
        Self {
            scope: request.scope.clone(),
            vendor_request: request.vendor_request.clone(),
        }
    }

    fn new(scope: &PermissionScope, vendor_request: &VendorEventRef) -> Self {
        Self {
            scope: scope.clone(),
            vendor_request: vendor_request.clone(),
        }
    }
}

/// Human-owned pending requests and explicit session grants.
#[derive(Debug, Default)]
pub struct PermissionController {
    pending: HashMap<PermissionRequestKey, PermissionRequest>,
    resolved: HashSet<PermissionRequestKey>,
    session_grants: HashSet<SessionGrant>,
}

impl PermissionController {
    /// Registers a request exactly once inside its scope or applies a session grant.
    pub fn register(
        &mut self,
        request: PermissionRequest,
    ) -> Result<PermissionRegistration, SafetyError> {
        let key = PermissionRequestKey::from_request(&request);
        if self.resolved.contains(&key) {
            return Err(SafetyError::StaleRequest {
                vendor_request: request.vendor_request,
            });
        }
        if self.pending.contains_key(&key) {
            return Err(SafetyError::DuplicateRequest {
                vendor_request: request.vendor_request,
            });
        }
        if self
            .session_grants
            .contains(&SessionGrant::from_request(&request))
        {
            self.resolved.insert(key);
            return Ok(PermissionRegistration::Covered(Box::new(
                PermissionResolution {
                    request,
                    decision: PermissionDecision::AllowSession,
                    source: PermissionResolutionSource::SessionGrant,
                },
            )));
        }
        self.pending.insert(key, request);
        Ok(PermissionRegistration::Pending)
    }

    /// Applies one human decision to a request identified by host scope and vendor ref.
    pub fn decide_scoped(
        &mut self,
        scope: &PermissionScope,
        vendor_request: &VendorEventRef,
        decision: PermissionDecision,
    ) -> Result<PermissionResolution, SafetyError> {
        let key = PermissionRequestKey::new(scope, vendor_request);
        if self.resolved.contains(&key) {
            return Err(SafetyError::StaleRequest {
                vendor_request: vendor_request.clone(),
            });
        }
        let Some(request) = self.pending.remove(&key) else {
            return Err(SafetyError::UnknownRequest {
                vendor_request: vendor_request.clone(),
            });
        };
        if decision == PermissionDecision::AllowSession {
            self.session_grants
                .insert(SessionGrant::from_request(&request));
        }
        self.resolved.insert(key);
        Ok(PermissionResolution {
            request,
            decision,
            source: PermissionResolutionSource::HumanDecision,
        })
    }

    /// Compatibility entrypoint for one globally unique pending vendor reference.
    pub fn decide(
        &mut self,
        vendor_request: &VendorEventRef,
        decision: PermissionDecision,
    ) -> Result<PermissionResolution, SafetyError> {
        let scopes = self
            .pending
            .keys()
            .filter(|key| &key.vendor_request == vendor_request)
            .map(|key| key.scope.clone())
            .collect::<Vec<_>>();
        match scopes.as_slice() {
            [scope] => self.decide_scoped(scope, vendor_request, decision),
            [] if self
                .resolved
                .iter()
                .any(|key| &key.vendor_request == vendor_request) =>
            {
                Err(SafetyError::StaleRequest {
                    vendor_request: vendor_request.clone(),
                })
            }
            [] => Err(SafetyError::UnknownRequest {
                vendor_request: vendor_request.clone(),
            }),
            _ => Err(SafetyError::AmbiguousRequest {
                vendor_request: vendor_request.clone(),
            }),
        }
    }
    /// Returns the number of requests still waiting for a human decision.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

/// Vendor permission callback seam; C1 intentionally ships an unwired implementation.
pub trait PermissionBridge {
    fn apply(&mut self, resolution: &PermissionResolution) -> Result<(), BridgeError>;
}

/// Fail-closed bridge used until a real Claude MCP callback is proven live.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnwiredPermissionBridge;

impl PermissionBridge for UnwiredPermissionBridge {
    fn apply(&mut self, _resolution: &PermissionResolution) -> Result<(), BridgeError> {
        Err(BridgeError::Unsupported)
    }
}

/// Permission controller contract violations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SafetyError {
    #[error("payload is not a permission request")]
    NotPermissionRequest,
    #[error("permission request is already pending")]
    DuplicateRequest { vendor_request: VendorEventRef },
    #[error("permission request was already resolved")]
    StaleRequest { vendor_request: VendorEventRef },
    #[error("permission request is unknown")]
    UnknownRequest { vendor_request: VendorEventRef },
    #[error("permission request is ambiguous across execution scopes")]
    AmbiguousRequest { vendor_request: VendorEventRef },
}

/// Vendor bridge application failure.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum BridgeError {
    #[error("runtime permission bridge is unsupported")]
    Unsupported,
}
