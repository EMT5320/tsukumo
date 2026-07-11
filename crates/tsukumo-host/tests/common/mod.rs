//! Shared deterministic fixtures for Host integration contracts.

// Integration test binaries intentionally consume different support subsets.
#![allow(dead_code, unused_imports)]

mod ledger;
mod process;
mod projection;

pub use ledger::TestLedger;
pub use process::{successful_outputs, FakeRunner, FixedClock};
pub use projection::{prepared_fixture, prepared_fixture_with_goal};
