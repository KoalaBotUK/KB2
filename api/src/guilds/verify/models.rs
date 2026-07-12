//! Verify config types now live in `common::verify` (shared with the
//! `consumer` crate's reconciliation worker); re-exported here so existing
//! `crate::guilds::verify::models::*` paths keep working.
//!
//! The old `recompute_role_members` (which needed every link in the guild in
//! memory) is gone: per-user handlers apply ±1 deltas via
//! `Verify::bump_members`, and a completed reconciliation job overwrites the
//! covered roles' counts via `common::verify::fold_counts_into_guild`.

pub use common::verify::{Verify, VerifyRole};
