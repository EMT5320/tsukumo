//! Shared real-pack product surface for host renderer integration tests.

use tsukumo_host::{load_presentation_pack, PresentationPackSource};
use tsukumo_kernel::SpiritId;
use tsukumo_theater::{
    AppState, DisplayText, ExecutionPhase, ProductView, RuntimeHealth, StageWorld,
    ValidatedPresentationPack,
};

pub fn default_surface() -> (ValidatedPresentationPack, StageWorld, ProductView, AppState) {
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)
        .expect("embedded presentation pack");
    let mut world = StageWorld::new();
    world.ensure_placeholder(pack.companion().actor_id.clone());
    let mut view = ProductView::default();
    view.runtime.health = RuntimeHealth::Ready;
    view.runtime.source_spirit_id = Some(SpiritId::new("runtime-spirit"));
    view.runtime.detail = DisplayText::from_untrusted("codex/acp ready");
    view.execution.phase = ExecutionPhase::Idle;
    view.execution.summary = DisplayText::from_untrusted("等待下一份委托");
    (pack, world, view, AppState::new(false))
}
