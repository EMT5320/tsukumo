//! Tsukumo L4 theater: StageEvent surface + pure Director + thin HalfBlock stage.
//!
//! Rendering consumes [`StageEvent`] only. Vendor / ACP details never enter here.

pub mod app;
pub mod director;
pub mod drive;
pub mod pack;
pub mod render;
pub mod stage;
pub mod world;

pub use app::{
    reduce_app, AppState, DisplayText, ExecutionPhase, ExecutionStatusView, HandoffStatusView,
    NoticeLevel, NoticeView, PermissionEvidenceText, PermissionView, ProductView,
    ProjectionStateRefView, ProjectionView, RuntimeHealth, RuntimeStatusView, Screen, StateStatus,
    StateView, UiAction, UiInput, UiKey, UiPermissionId, ViewModelError,
};
pub use director::{direct, DirectorContext, LineBook};
pub use drive::{drive_kernel_event, drive_kernel_events};
pub use pack::{
    parse_presentation_pack, presentation_pack_assets, PackDocuments, PresentationActorId,
    PresentationPackError, ValidatedPresentationPack, PACK_SCHEMA_VERSION,
};
pub use render::{
    buffer_to_ansi, buffer_to_string, render_frame, render_frame_string, render_product_frame,
    select_layout, ColorCapability, LayoutMode, ProductWidget, DEFAULT_FRAME_HEIGHT,
    DEFAULT_FRAME_WIDTH,
};
pub use stage::{ActorPose, AttentionTier, StageAttribution, StageEvent};
pub use world::{ActorSlot, ActorSnapshot, Motion, StageLogLine, StageSnapshot, StageWorld};
