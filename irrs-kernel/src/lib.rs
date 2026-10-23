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
/// ─── Domain-specificity principle ────────────────────────────────────────────
///
/// Complexity makes domain-specific parameters structurally unavoidable.
///
/// The relational structure of a hydrogen atom, a magnetospheric current sheet,
/// a biochemical oscillator, and a social network are incommensurable. The
/// characteristic scale at which antisymmetric circulation saturates (χ₀), the
/// ceiling of the coupling coefficient (ρ_base), and the cross-topology coupling
/// magnitudes (cc) are properties of the declared domain — not universal constants
/// with unknown numerical values.
///
/// This is not an incomplete specification. It is the correct structure of the
/// framework. Complexity of the declared domain is what determines the applicable
/// values. A domain that has not established these parameters through M has not
/// completed its declaration; the kernel correctly holds no value and supplies no
/// default.
///
/// Every application of Kernel V8 operators to a new domain requires M to
/// establish domain-specific values for all operator parameters before any
/// operator is applied. An application that assigns universal or cross-domain
/// parameter values violates the admissibility condition.
///
/// ─────────────────────────────────────────────────────────────────────────────
///
/// This kernel is frozen. No modification is authorized.
/// Where this crate and `kernel_reference.md` differ, the source file governs;
/// a difference is a finding to be returned to Origin.

pub mod operators;
pub mod derived_invariants;
pub mod region;
