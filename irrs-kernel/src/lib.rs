/// IRRS Kernel V8 — Canonical Implementation
///
/// Implements the operator mathematics from `kernel_reference.md` (Metatron Dynamics,
/// Kernel V8, 2026-09-27; operational discipline added 2026-09-29).
///
/// File responsibilities (separation rule, section 13 of kernel_reference.md):
///   `operators.rs`          — what acts: operators, data structures, ρ computation
///   `derived_invariants.rs` — what is produced: confirmed Part I invariant functions
///   `region.rs`             — M Regional Instantiation Prerequisite enforcement
///
/// This kernel is frozen. No modification is authorized.
/// Where this crate and `kernel_reference.md` differ, the source file governs;
/// a difference is a finding to be returned to Origin.

pub mod operators;
pub mod derived_invariants;
pub mod region;
