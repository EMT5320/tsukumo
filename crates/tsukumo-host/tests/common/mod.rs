//! Shared deterministic fixtures for Host integration contracts.

// Integration test binaries intentionally consume different support subsets.
#![allow(dead_code, unused_imports)]

mod cross_runtime;
mod ledger;
mod process;
mod projection;

pub use cross_runtime::{
    materialize_cross_runtime_repository, prepare_post_revoke_projection,
    prepared_cross_runtime_comparison, CrossRuntimePrepared,
};
pub use ledger::TestLedger;
pub use process::{successful_outputs, FakeRunner, FixedClock};
pub use projection::{
    prepared_dual_runtime_live_fixture, prepared_fixture, prepared_fixture_with_goal,
};
