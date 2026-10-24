/// IRRS Kernel V8 — Canonical Implementation
///
/// ═══════════════════════════════════════════════════════════════════════════
/// THE FOUNDATIONAL PRIMITIVE
/// ═══════════════════════════════════════════════════════════════════════════
///
/// This crate introduces one primitive that has not previously existed in
/// mathematics:
///
///   A DIRECTED RELATION DECLARED FROM OBSERVABLE INFORMATION.
///
/// Every structure in this crate — every operator, every invariant, every
/// admissibility condition — exists because of this primitive and cannot be
/// used without it.
///
/// The primitive is not derived. It is the starting point. It arises from
/// the distillation of human experience into three irreducible characteristics:
///
///   1. COMPLEXITY
///      Observable systems are complex. They are not reducible to a single
///      governing variable or a universal parameter set. The structure of a
///      hydrogen atom, a magnetospheric current sheet, a biochemical oscillator,
///      and a social network are incommensurable. Complexity makes domain-
///      specific declaration structurally unavoidable — not a limitation to
///      be overcome, but the correct description of what is actually there.
///
///   2. CHANGE AS THE ONLY OBSERVABLE CONSTANT
///      What is consistently observable across all domains is not a stable
///      object, but change. The directed relation captures change at its most
///      elementary form: a declared difference between two loci, with direction.
///      Nothing in this framework assumes a stable substrate beneath the change.
///      Change is what is declared. Change is what the operators act on.
///
///   3. BOUNDED DOMAIN OF ANALYSIS
///      Every declaration is bounded. D contains only what has been declared
///      through M from observable evidence. There is no unbounded extension,
///      no inference beyond the declared boundary, no silent default. The
///      boundary is not a deficiency — it is what makes the result correspond
///      to what was actually observed rather than to a model of it.
///
/// These three characteristics — complexity, change, and bounded domain —
/// jointly determine that the correct primitive is a DIRECTED RELATION, not
/// a point, not a set, not an object with properties. Standard mathematics
/// constructs relations from objects. This framework takes the directed
/// relation itself as primitive and derives everything else from it.
///
/// ───────────────────────────────────────────────────────────────────────────
///
/// THE OPERATORS
///
/// The operators (A, B, R, Δ, Σ) are not independent mathematical objects.
/// They are the forms that act on declared directed relations. They cannot
/// be applied to anything that was not declared through M, and they cannot
/// produce output that carries meaning beyond the declared boundary.
///
/// The operators exist because of the primitive. The primitive does not exist
/// because of the operators.
///
/// ───────────────────────────────────────────────────────────────────────────
///
/// File responsibilities (separation rule, section 13 of kernel_reference.md):
///   `operators.rs`          — what acts: operators, data structures, ρ computation
///   `derived_invariants.rs` — what is produced: confirmed Part I invariant functions
///   `region.rs`             — M Regional Instantiation Prerequisite enforcement
///
/// ───────────────────────────────────────────────────────────────────────────
///
/// DOMAIN-SPECIFICITY
///
/// Complexity makes domain-specific parameters structurally unavoidable.
/// The characteristic scale at which antisymmetric circulation saturates (χ₀),
/// the ceiling of the coupling coefficient (ρ_base), and the cross-topology
/// coupling magnitudes (cc) are properties of the declared domain — not
/// universal constants with unknown numerical values. A domain that has not
/// established these parameters through M has not completed its declaration;
/// the kernel correctly holds no value and supplies no default.
///
/// ───────────────────────────────────────────────────────────────────────────
///
/// Metatron Dynamics Kernel V8 (2026-09-27; discipline 2026-09-29).
/// This kernel is frozen. No modification is authorized.
/// Where this crate and `kernel_reference.md` differ, the source file governs;
/// a difference is a finding to be returned to Origin.

pub mod operators;
pub mod derived_invariants;
pub mod region;
