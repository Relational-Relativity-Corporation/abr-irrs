# IRRS V9 Provenance Admission — Phase 1 Architecture Assessment

**Repository:** Relational-Relativity-Corporation/abr-irrs  
**Governing specification:** abr-kernel `main`, `docs/kernel/v9_specification.md`  
**Base commit:** `1400d3bb33767a53a393346ab9dcf1fa5a3aa155`  
**Assessment date:** 2026-10-24  
**Generator:** Claude Sonnet 4.6  
**Status:** HANDOFF TO VERIFIER — awaiting Origin approval before Phase 2

---

## Purpose

This document is the Phase 1 deliverable for the IRRS V9 Provenance Admission milestone. It records the architecture assessment of the existing IRRS workspace against the seven V9 requirements (Req-1 through Req-7) defined in the governing specification, and proposes the minimal additive Rust interface that Phase 2 would implement.

No implementation has been performed. No existing files have been modified. No V8 operators have been touched. This document is the entire Phase 1 output.

---

## Workspace Structure

The IRRS workspace (`abr-irrs`) comprises five crates plus two examples:

| Crate | Role |
|---|---|
| `irrs-kernel` | Frozen Kernel V8 operators. Must not be modified. |
| `irrs-session` | D/M-driven Session layer orchestrating A→B→R via V8 operators. |
| `relational-gate` | General-purpose scalar admission gate (`AdmissionGate<O,V>`). |
| `irrs-eval` | Pre-build JSON declaration evaluator (runs before gate construction). |
| `examples/irrs-e2e-tests` | End-to-end test driver (Tests A–E). |

The orchestrator (`irrs_orchestrator.py`) runs the M1→M4 pipeline: commit pin, Rust execution, evidence package, OpenAI verification.

---

## Files Inspected

| File | Lines | Content |
|---|---|---|
| `irrs-kernel/src/operators.rs` | ~700 | Frozen V8: `DeclaredRelations`, `NodeField`, `EdgeField`, `PersistenceState`, all operators |
| `irrs-session/src/session.rs` | ~350 | `Session`, `ParameterProvenance`, A→B→R chain, OPEN condition surfacing |
| `relational-gate/src/gate.rs` | 190 | `AdmissionGate`, `ProvenanceRecord`, `AdmissionOutcome`, `AdmissionError` |
| `irrs-eval/src/declaration.rs` | 307 | `Declaration`, `InputDeclaration`, `RegionalPrerequisiteDecl`, `PersistenceDeclaration` |
| `irrs-eval/src/evaluator.rs` | ~120 | Pre-build `Evaluator` — runs before gate construction, does not execute mappings |
| `examples/irrs-e2e-tests/src/main.rs` | 448 | Tests A–E, OPEN E-1 documented |
| `irrs_orchestrator.py` | 1091 | M1–M4 pipeline; authorized commit `1400d3bb`; stop conditions |

---

## Relevant Types — Existing

### irrs-kernel (frozen V8)

```rust
// EdgeProvenance — provenance label only; not a declaration record.
pub enum EdgeProvenance { Continuation, Coupling, Persistence }

// DeclaredRelations — topology with per-edge provenance labels.
// from_edges_with_provenance assigns labels at construction.
// No observable_source, direction_basis, m_mapping_ref, declared_domain.
pub struct DeclaredRelations { n_nodes, n_edges, edges, provenance, out, inc, adj_plus, adj_minus }

// NodeField — numerical projection, no provenance.
pub struct NodeField { data: Vec<Vec<f64>>, k, n }

// EdgeField — bare operator output, no provenance wrapper.
pub struct EdgeField { spatial: Vec<Vec<f64>>, comp: Vec<Vec<f64>>, comp_pairs, k }

// PersistenceState — constructed only from a prior EdgeField; no cold-start.
pub struct PersistenceState { e_prior: EdgeField }
```

Operators `operator_a`, `operator_b`, `operator_r`, `operator_e`, `operator_e_v5` all return bare `EdgeField` or `(EdgeField, PersistenceOutput)` with no calculation record attached.

### relational-gate (scalar gate)

```rust
// DeclarationPackage — submitted by the Generator for scalar gate admission.
// Has topology_id, mapping_name, provenance_chain, source (Observable).
// No ObservableRelationDeclaration: no observable_source, direction_basis,
// m_mapping_ref, declared_domain.

// ProvenanceRecord — records gate execution metadata.
pub struct ProvenanceRecord {
    source_id, topology_id, mapping_applied,
    primary_input, reference_input, gate_computed_result,
}
// Not a V9 declaration record: carries what was computed, not what was declared.

// AdmissionGate — independently executes named mappings (step 4 in submit()).
// Rejects: missing topology_id, mapping_name, empty provenance_chain,
//          silent transformations, topology mismatch, topology violation,
//          unknown mapping, mapping failure.
// Does NOT check for ObservableRelationDeclaration or RPPI failure modes.
```

### irrs-eval (pre-build declaration evaluator)

```rust
// Declaration — JSON-loaded domain declaration. Runs before gate construction.
pub struct Declaration {
    domain, topology_id, version,
    inputs: Vec<InputDeclaration>,
    regional_prerequisite: RegionalPrerequisiteDecl,
    mappings: Vec<MappingDeclaration>,
    persistence: Option<PersistenceDeclaration>,
}

// InputDeclaration — per-input provenance metadata.
// Structurally related to ObservableRelationDeclaration but distinct:
//   - Per-input, not per-edge.
//   - Not linked into the admission chain.
//   - Evaluated before gate construction, not at admission time.
pub struct InputDeclaration {
    name, classification: InputClassification,
    source_identity,       // approaches observable_source
    acquisition_method,    // approaches direction_basis (partially)
    transformation_history,
    relationship_to_problem,  // approaches declared_domain (partially)
    supporting_evidence: Vec<EvidenceRef>,
    provenance_status: ProvenanceStatus,  // Declared | Verified | Unverified
    submitted_to_gate: bool,
}

// PriorEvidenceStatus — Available | Open | Synthetic (Synthetic is PROHIBITED).
// This is the only existing rejection of a fabricated provenance (for persistence).
```

### irrs-session

```rust
pub enum ParameterProvenance {
    Illustrative,
    EvidenceDeclared(EvidenceRecord),
    ObservationallyVerified(VerificationRecord),
}
// VerificationRecord carries declared_observables and comparisons.
// Provenance of ρ_base and χ₀ — not provenance of observable relations.

// Session::execute() returns SessionOutput::Complete(ExecutionResult).
// ExecutionResult::RelationalOutput { edge_field, parameter_provenance,
//                                     evaluated_comparisons }
// edge_field is a bare EdgeField — no calculation record, no derivation binding.
```

---

## Gap Matrix — V9 Requirements

### Req-1: Every admitted relation has a declared observable relation in D.

**Status: OPEN**

The existing types admit relations based on topology and mapping validity but do not require an `ObservableRelationDeclaration` per edge. `EdgeProvenance` labels (`Continuation`, `Coupling`, `Persistence`) describe a class of relation, not an observable identity. `InputDeclaration` in `irrs-eval` records per-input provenance metadata but is not per-edge and is evaluated before gate construction, not at admission time.

No admission path checks for the presence of an `ObservableRelationDeclaration` before admitting a relation.

Gap: `ObservableRelationDeclaration` type does not exist.

---

### Req-2: The admitted relation traces to an ObservableRelationDeclaration.

**Status: OPEN**

`ProvenanceRecord` (gate) records execution metadata: `source_id`, `topology_id`, `mapping_applied`, `primary_input`, `reference_input`, `gate_computed_result`. This is a calculation record for the scalar gate, not a link to a declaration record for an observable relation.

`ExecutionResult` (session) carries `parameter_provenance` (the ρ/χ₀ provenance) and `edge_field` (a bare `EdgeField`). There is no link from the `EdgeField` to any declaration record.

Gap: No traceable chain from `AdmittedObservation<V>` or `EdgeField` to an `ObservableRelationDeclaration`.

---

### Req-3: ObservableRelationDeclaration has four required fields: observable_source, direction_basis, m_mapping_ref, declared_domain.

**Status: OPEN**

None of these four fields exist in any struct. Related concepts exist in other forms:

| V9 field | Nearest existing concept | Structural gap |
|---|---|---|
| `observable_source` | `InputDeclaration.source_identity` (irrs-eval, per-input) | Not per-edge; not in admission chain |
| `direction_basis` | `InputDeclaration.acquisition_method` (partial) | Does not record edge directionality |
| `m_mapping_ref` | `MappingDeclaration.name` (irrs-eval) | Not linked to per-edge declarations at admission time |
| `declared_domain` | `RegionalPrerequisiteDecl.region_name` + extents (irrs-eval) | Per-topology, not per-edge; in pre-build eval, not admission |

Gap: `ObservableRelationDeclaration` struct does not exist. No struct unifies these four fields per edge within the admission path.

---

### Req-4: Operator outputs wrapped in ProvenancedOutput<T> carrying the calculation record.

**Status: OPEN**

`operator_a/b/r/e/e_v5` all return bare `EdgeField` or `Vec<Vec<f64>>`. No wrapper type exists. `PrimaryEdgeField`, `EdgeField`, and `PersistenceOutput` carry only numerical values.

The `ExecutionResult` in the session carries `parameter_provenance` for ρ/χ₀, but this is provenance of the parameters, not a per-output calculation record linking results back to declared observable relations.

`ProvenanceRecord` in the gate records scalar execution metadata but is for the scalar gate path, not kernel operator outputs.

Gap: `ProvenancedOutput<T>` type does not exist. No calculation record type exists. No operator output is wrapped in a provenance-carrying container.

---

### Req-5: V8 constructors frozen; RPPI enforcement in separate V9 admission interface.

**Status: PARTIAL — frozen V8 confirmed; V9 admission interface absent**

`irrs-kernel/src/operators.rs` header: "Frozen at Kernel V8. No modification is authorized." The repository's governance (orchestrator stop conditions, irrs-eval authorization) enforces this. The V8 operators themselves are not modified.

The V9 admission interface does not exist. The existing `AdmissionGate<O,V>` operates on scalar values submitted via `DeclarationPackage<O>`. It does not handle kernel operator chains or `ProvenancedOutput<T>`.

No admission interface currently enforces RPPI failure modes (§6.1 missing declaration, §6.2 fabricated provenance, §6.3 disconnected output, §6.4 misattributed derivation) for kernel operator outputs.

Gap: V9 admission interface does not exist as a separate crate or module.

---

### Req-6: IRRS independently verifies calculation records (not self-certification).

**Status: PARTIAL — scalar gate independent; kernel chain self-certifying**

The `AdmissionGate` independently re-executes named mappings from raw inputs (step 4 of `submit()`). This satisfies independent execution for scalar gate values. The gate does not trust the Generator's submitted result; it computes the result itself and rejects on mismatch or failure. This is the correct structure for scalar admission.

For kernel operator chains (A→B→R), the session `execute()` method runs the operators and returns results. There is no independent verification of the operator chain against declaration records. The caller receives `ExecutionResult` without any independent check that the outputs trace to declared observable relations.

Critically: the session does not carry calculation records through the operator chain. There is nothing for an independent verifier to check against. The absence of `ProvenancedOutput<T>` (Req-4) is what makes independent verification structurally impossible at present.

Gap: No independent verification path for kernel operator chain outputs. Self-certification is the current mode for A→B→R results.

---

### Req-7: Admission interface rejects RPPI failure modes (missing, fabricated, disconnected, misattributed).

**Status: PARTIAL — persistence fabrication rejected; kernel chain RPPI unchecked**

**Missing (§6.1):** No rejection path exists. An `EdgeField` can be admitted without any `ObservableRelationDeclaration` for any edge. The V9 admission interface is absent (Req-5).

**Fabricated (§6.2):** `irrs-eval` rejects `PriorEvidenceStatus::Synthetic` for persistence declarations. This is the only existing rejection of fabricated provenance, and it applies to the persistence path only. No equivalent check exists for spatial A→B→R outputs.

**Disconnected (§6.3):** No rejection path exists. Operator outputs are bare `EdgeField` values, not wrapped in `ProvenancedOutput<T>`. There is no wrapper to check.

**Misattributed (§6.4):** No rejection path exists. Derivation binding is not tracked.

Gap: Three of four RPPI failure modes (missing, disconnected, misattributed) have no rejection path. Fabricated provenance is rejected only for the persistence path in the pre-build evaluator.

---

## Summary Table

| Req | Description | Status | Primary gap |
|---|---|---|---|
| Req-1 | Every admitted relation has an ObservableRelationDeclaration | OPEN | `ObservableRelationDeclaration` type absent |
| Req-2 | Admitted relation traces to declaration record | OPEN | No traceable link from output to declaration |
| Req-3 | ObservableRelationDeclaration has four required fields | OPEN | None of the four fields exist per-edge in admission |
| Req-4 | Operator outputs wrapped in ProvenancedOutput<T> | OPEN | `ProvenancedOutput<T>` type absent; bare EdgeField returned |
| Req-5 | V8 frozen; V9 admission interface separate | PARTIAL | V8 frozen ✓; V9 admission interface absent |
| Req-6 | Independent verification of calculation records | PARTIAL | Scalar gate independent ✓; kernel chain self-certifying |
| Req-7 | Rejection of four RPPI failure modes | PARTIAL | Only fabricated persistence rejected; three modes unchecked |

---

## Architecture Observations

### What the existing architecture does well

1. **V8 operator independence.** The kernel operators are correctly frozen and governance (orchestrator, irrs-eval authorization, stop conditions) enforces this boundary. This is the correct substrate for V9 to build on.

2. **Scalar gate independent execution.** `AdmissionGate` independently re-executes named mappings. This is the right architectural pattern for independent verification, and it can serve as a model for the V9 admission interface.

3. **Persistence fabrication rejection.** `irrs-eval` correctly rejects `PriorEvidenceStatus::Synthetic` and distinguishes "unavailable" from "zero." This is the right structure for the fabricated-provenance failure mode.

4. **ParameterProvenance differentiation.** The session distinguishes `Illustrative`, `EvidenceDeclared`, and `ObservationallyVerified` for ρ/χ₀. This provenance vocabulary can be extended or analogized for observable relations.

5. **OPEN condition surfacing.** The session surfaces `RhoMaxSelectionRule` and `RhoSaturatingForm` as kernel OPEN conditions in every execution. This is the correct behavior for unresolved mathematical declarations.

### Where the structural gap is

The fundamental gap is the absence of a traceable chain from declared observable relations through to operator outputs. The chain requires:

```
ObservableRelationDeclaration  →  declared M  →  projected input
  →  authorized operator execution  →  ProvenancedOutput<T>  →  independent IRRS verification
```

Currently:
- Step 1 (`ObservableRelationDeclaration`): absent.
- Steps 2–4 (M projection, operator execution): implemented in session.
- Step 5 (`ProvenancedOutput<T>`): absent; bare `EdgeField` returned.
- Step 6 (independent IRRS verification): absent for kernel chain; present for scalar gate only.

The `irrs-eval` `InputDeclaration` captures declaration-time metadata per input but is (a) not per-edge, (b) not inside the admission chain, and (c) evaluated before gate construction. It is a pre-build check, not an admission gate.

### Proposed addition scope (Phase 2, not yet authorized)

The following is the proposed additive interface for Verifier review. Nothing below is implemented.

**New crate: `irrs-v9-admission`** (or new module in `irrs-session`)

```rust
// ObservableRelationDeclaration — per-edge declaration record.
// Authored by Origin at construction time, before operators run.
pub struct ObservableRelationDeclaration {
    pub edge_index:      usize,
    pub observable_source: String,   // what observable entity this edge represents
    pub direction_basis:  String,    // what determines edge directionality
    pub m_mapping_ref:    String,    // reference to M that maps this observable into D
    pub declared_domain:  String,    // the region D within which this relation is bounded
}

// CalculationRecord — per-output record of operator execution.
// Authored by the kernel operator during computation.
// Carried through the admission interface.
pub struct CalculationRecord {
    pub edge_index:       usize,
    pub operator_chain:   Vec<String>,    // e.g. ["A", "B", "R"]
    pub declaration_ref:  usize,          // index into declared ObservableRelationDeclarations
    pub input_values:     Vec<f64>,       // actual input values at execution time
    pub output_value:     f64,            // calculated result
    pub rho_applied:      f64,            // ρ value used
}

// ProvenancedOutput<T> — wrapper carrying the calculation record over T.
pub struct ProvenancedOutput<T> {
    pub value:              T,
    pub calculation_record: Vec<CalculationRecord>,
}

// V9AdmissionRequest — submitted to V9 admission interface.
pub struct V9AdmissionRequest {
    pub declarations:   Vec<ObservableRelationDeclaration>,
    pub output:         ProvenancedOutput<EdgeField>,
}

// V9AdmissionOutcome — result of V9 admission.
pub enum V9AdmissionOutcome {
    Admitted { verified_output: EdgeField },
    Rejected { failure_mode: RppiFailureMode, edge_index: usize, detail: String },
}

pub enum RppiFailureMode { Missing, Fabricated, Disconnected, Misattributed }
```

The V9 admission interface independently verifies `CalculationRecord` entries against `ObservableRelationDeclaration` entries. It does not re-execute the kernel operators itself (they are frozen), but it checks that:
- Every edge has a corresponding `ObservableRelationDeclaration` (§6.1).
- Every `m_mapping_ref` corresponds to a declared authorized mapping (§6.2).
- Every `CalculationRecord.declaration_ref` is within bounds and matches the declaration (§6.4).
- Every output is wrapped in `ProvenancedOutput<T>` (§6.3).

**Existing files unchanged**: `irrs-kernel/src/operators.rs`, `relational-gate/src/gate.rs`, `irrs-session/src/session.rs`, `irrs-eval/**`, orchestrator.

---

## Remaining OPEN Conditions

These are carried forward from the existing codebase and are not resolved by this phase:

| Condition | Source | Status |
|---|---|---|
| RhoMaxSelectionRule — max-selection rule for χ requires Origin declaration | `irrs-kernel/src/operators.rs` §OPEN | OPEN |
| RhoSaturatingForm — saturating functional form requires Origin declaration | `irrs-kernel/src/operators.rs` §OPEN | OPEN |
| OPEN E-1 — session API does not expose PersistenceState across calls | `examples/irrs-e2e-tests/src/main.rs` Test E | OPEN |

These are not in scope for V9 Provenance Admission and must not be closed as a side-effect of Phase 2.

---

## Decisions Requiring Origin Authorization

Before Phase 2 can begin, the following require explicit Origin authorization:

1. **New crate vs. new module.** The V9 admission interface may be added as a new crate `irrs-v9-admission` or as a new module within `irrs-session`. This is a structural decision with consequences for dependency graph and separation of concerns.

2. **Calculation record authorship.** The current proposal has the Generator (caller of `operator_a/b/r`) populate `CalculationRecord` entries and wrap them in `ProvenancedOutput<T>`. An alternative is for the V9 admission interface to re-derive calculation records from raw inputs using the frozen operators (stricter self-verification). The choice determines whether Phase 2 requires any modification to how `Session::execute()` calls operators.

3. **ObservableRelationDeclaration placement.** In the current proposal, `ObservableRelationDeclaration` is carried in `V9AdmissionRequest` alongside `ProvenancedOutput<T>`. An alternative is to attach declarations to `DeclaredRelations` at construction time (closer to `from_edges_with_provenance`). This would require modification to a frozen V8 type (not authorized by Req-5) unless a separate declaration registry is maintained alongside.

4. **Conformance test scope.** Phase 3 specifies 11 conformance scenarios. Origin authorization is required to confirm the test contract before implementation. The 4 `#[ignore]` tests in `abr-kernel/tests/v9_conformance.rs` define the V9 enforcement boundary that IRRS must satisfy.

---

## Test Commands (Existing Baseline)

Run before and after any Phase 2 changes to confirm preservation:

```bash
# Workspace tests
cargo test --workspace

# End-to-end test driver
cargo run -p irrs-e2e-tests

# irrs-eval tests
cargo test -p irrs-eval

# irrs-session admission tests
cargo test -p irrs-session
```

All must pass with zero regressions before any Phase 2 change is committed.

---

## Handoff Summary

**Base commit:** `1400d3bb33767a53a393346ab9dcf1fa5a3aa155`  
**Files changed in Phase 1:** one new file — `docs/v9_provenance/phase1_gap_matrix.md` (this document). No source files modified.  
**Gap matrix:** Req-1 OPEN, Req-2 OPEN, Req-3 OPEN, Req-4 OPEN, Req-5 PARTIAL, Req-6 PARTIAL, Req-7 PARTIAL.  
**Proposed interface:** minimal additive types listed above — `ObservableRelationDeclaration`, `CalculationRecord`, `ProvenancedOutput<T>`, `V9AdmissionRequest`, `V9AdmissionOutcome`. All additive, no V8 modifications.  
**OPEN conditions preserved:** RhoMaxSelectionRule, RhoSaturatingForm, OPEN E-1.  
**Decisions requiring Origin authorization:** 4 items listed above.  
**Phase 2 status:** NOT STARTED — awaiting Origin approval.

Bounded over D. No claim beyond D.
