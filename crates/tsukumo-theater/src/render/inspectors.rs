//! State and projection inspection panes.

mod projection;
mod shared;
mod state;

pub(super) use projection::render_projection_inspector;
pub(super) use state::render_state_inspector;
