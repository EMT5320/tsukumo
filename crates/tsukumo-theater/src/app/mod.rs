//! Pure product view model and keyboard reducer.

mod model;
mod reducer;
mod view;

pub use model::{
    AppState, DisplayText, PermissionEvidenceText, Screen, UiAction, UiInput, UiKey,
    UiPermissionId, ViewModelError,
};
pub use reducer::reduce_app;
pub use view::*;
