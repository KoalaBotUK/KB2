//! Link-pattern matching now lives in `common::verify` (shared with the
//! `consumer` crate's reconciliation worker); re-exported here so existing
//! `crate::users::utils::*` paths keep working. The never-panic-on-invalid-
//! pattern regression tests moved to `common::verify` with the code.

pub use common::verify::{link_arr_match, link_match};
