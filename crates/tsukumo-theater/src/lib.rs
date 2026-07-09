//! Tsukumo L4 theater: StageEvent surface + pure Director + thin HalfBlock stage.
//!
//! Rendering consumes [`StageEvent`] only. Vendor / ACP details never enter here.

pub mod director;
pub mod drive;
pub mod render;
pub mod stage;
pub mod world;

pub use director::{direct, DirectorContext, LineBook};
pub use drive::{drive_kernel_event, drive_kernel_events};
pub use render::{
    buffer_to_string, render_frame, render_frame_string, DEFAULT_FRAME_HEIGHT, DEFAULT_FRAME_WIDTH,
};
pub use stage::{ActorPose, AttentionTier, StageEvent};
pub use world::{ActorSlot, ActorSnapshot, Motion, StageSnapshot, StageWorld};
