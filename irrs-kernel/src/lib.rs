/// IRRS Kernel V8 — Frozen Implementation
///
/// No modifications are authorized.
/// Consumed exclusively through irrs-admission.
/// Application code (datafeel-v05) has no direct dependency on this crate.
///
/// This module contains the operator mathematics for Δ→Σ and A→B→R.
/// All kernel entry points are pub(crate) to irrs-admission; they are not
/// exposed to application code.

pub mod topology;
pub mod operators;
pub mod region;
