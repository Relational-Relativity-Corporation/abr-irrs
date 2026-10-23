/// IRRS Session — Admission and Regression Tests
///
/// Covers:
///   BLOCKED:    topology missing, no interior locus, field missing, ρ missing,
///               dimension mismatch, invalid ρ parameter
///   PARTIAL:    topology+field declared, ρ not declared → A and B exposed
///   COMPLETE:   illustrative ρ, correct R values, CalculatedZero (flat field),
///               multi-component detection, kernel OPEN conditions present
///   PROVENANCE: three-state provenance (Illustrative / EvidenceDeclared /
///               ObservationallyVerified); failed correspondence; open items

use irrs_kernel::operators::{
    DeclaredRelations, EdgeProvenance, NodeField,
};
use irrs_session::admission::{
    AdmissionResult, DeclaredObservable, EvidenceRecord,
    ExecutionResult, KernelOpenCondition, ObservationalComparison, OpenCondition,
    ParameterProvenance, SessionOutput, VerificationRecord, DETECTION_TOLERANCE,
};
use irrs_session::session::Session;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Standard s→t→u topology: 3 nodes, 2 edges, interior locus at node 1.
fn stu_topology() -> DeclaredRelations {
    DeclaredRelations::from_edges_with_provenance(
        3,
        vec![(0, 1), (1, 2)],
        vec![EdgeProvenance::Continuation, EdgeProvenance::Continuation],
    )
}

fn stu_field_345() -> NodeField {
    NodeField::new(vec![vec![3.0, 5.0, 2.0]])
}

fn stu_field_flat() -> NodeField {
    NodeField::new(vec![vec![5.0, 5.0, 5.0]])
}

fn illustrative_rho() -> (f64, f64, ParameterProvenance) {
    (0.8, 1.0, ParameterProvenance::Illustrative)
}

// ─── BLOCKED: topology not declared ──────────────────────────────────────────

#[test]
fn test_session_open_topology_not_declared() {
    let session = Session::new();
    match session.admit() {
        AdmissionResult::Open(conditions) => {
            assert!(
                conditions.contains(&OpenCondition::TopologyNotDeclared),
                "expected TopologyNotDeclared, got {:?}", conditions
            );
        }
        AdmissionResult::Admitted { .. } => panic!("expected Open, got Admitted"),
    }
    // execute() should also return Blocked
    match session.execute() {
        SessionOutput::Blocked(conditions) => {
            assert!(conditions.contains(&OpenCondition::TopologyNotDeclared));
        }
        other => panic!("expected Blocked, got {:?}", match other {
            SessionOutput::Complete(_) => "Complete",
            SessionOutput::Partial { .. } => "Partial",
            SessionOutput::Blocked(_) => "Blocked",
        }),
    }
}

// ─── BLOCKED: no interior locus ───────────────────────────────────────────────
//
// 2 nodes, 1 edge s→t: no node is simultaneously target and source.

#[test]
fn test_session_open_no_interior_locus() {
    let mut session = Session::new();
    let rel = DeclaredRelations::from_edges_with_provenance(
        2, vec![(0, 1)], vec![EdgeProvenance::Continuation],
    );
    session.declare_topology(rel);
    session.declare_field(NodeField::new(vec![vec![3.0, 5.0]]));
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);

    match session.admit() {
        AdmissionResult::Open(conditions) => {
            assert!(conditions.contains(&OpenCondition::NoInteriorLocus),
                "expected NoInteriorLocus, got {:?}", conditions);
            assert!(!conditions.contains(&OpenCondition::TopologyNotDeclared),
                "topology was declared; unexpected TopologyNotDeclared");
        }
        AdmissionResult::Admitted { .. } => panic!("expected Open, got Admitted"),
    }
    match session.execute() {
        SessionOutput::Blocked(conditions) => {
            assert!(conditions.contains(&OpenCondition::NoInteriorLocus));
        }
        _ => panic!("expected Blocked for no interior locus"),
    }
}

// ─── BLOCKED: field not declared ─────────────────────────────────────────────

#[test]
fn test_session_open_field_not_declared() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);

    match session.admit() {
        AdmissionResult::Open(conditions) => {
            assert!(conditions.contains(&OpenCondition::NodeFieldNotDeclared),
                "expected NodeFieldNotDeclared, got {:?}", conditions);
        }
        AdmissionResult::Admitted { .. } => panic!("expected Open, got Admitted"),
    }
}

// ─── BLOCKED: ρ not declared ─────────────────────────────────────────────────

#[test]
fn test_session_open_rho_not_declared() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    // No rho declared

    match session.admit() {
        AdmissionResult::Open(conditions) => {
            assert!(conditions.contains(&OpenCondition::RhoNotDeclared),
                "expected RhoNotDeclared, got {:?}", conditions);
        }
        AdmissionResult::Admitted { .. } => panic!("expected Open, got Admitted"),
    }
}

// ─── PARTIAL: topology and field declared; ρ not declared ─────────────────────
//
// Verifier correction #3: expose A and B when R prerequisites are OPEN.
// A and B are computable from topology + field alone.

#[test]
fn test_partial_execution_rho_not_declared() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    session.declare_cc(vec![], vec![]);
    // No rho declared — expect Partial

    match session.execute() {
        SessionOutput::Partial { a_field, b_field, open_r_conditions, kernel_open } => {
            // A[e0] = x[0] − x[1] = 3 − 5 = −2
            // A[e1] = x[1] − x[2] = 5 − 2 = 3
            let tol = 1e-12_f64;
            assert!((a_field.spatial[0][0] - (-2.0)).abs() < tol,
                "A[e0] expected -2.0, got {}", a_field.spatial[0][0]);
            assert!((a_field.spatial[0][1] - 3.0).abs() < tol,
                "A[e1] expected 3.0, got {}", a_field.spatial[0][1]);

            // B[e0] = A[e0] + A[e1] (e1 is successor of e0) = -2 + 3 = 1
            // B[e1] = A[e1] (no successors) = 3
            assert!((b_field.spatial[0][0] - 1.0).abs() < tol,
                "B[e0] expected 1.0, got {}", b_field.spatial[0][0]);
            assert!((b_field.spatial[0][1] - 3.0).abs() < tol,
                "B[e1] expected 3.0, got {}", b_field.spatial[0][1]);

            // Open conditions should list RhoNotDeclared
            assert!(open_r_conditions.contains(&OpenCondition::RhoNotDeclared),
                "expected RhoNotDeclared in open_r_conditions, got {:?}", open_r_conditions);

            // Kernel OPEN conditions present regardless
            assert!(kernel_open.contains(&KernelOpenCondition::RhoMaxSelectionRule));
            assert!(kernel_open.contains(&KernelOpenCondition::RhoSaturatingForm));
        }
        SessionOutput::Complete(_) => panic!("expected Partial, got Complete"),
        SessionOutput::Blocked(c) => panic!("expected Partial, got Blocked({:?})", c),
    }
}

// ─── BLOCKED: dimension mismatch ─────────────────────────────────────────────
//
// Verifier correction #5: validate field/topology dimensions before executing.

#[test]
fn test_dimension_mismatch_blocked() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());  // 3 nodes
    // Field has 2 nodes — mismatch
    session.declare_field(NodeField::new(vec![vec![3.0, 5.0]]));
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);

    match session.admit() {
        AdmissionResult::Open(conditions) => {
            let has_mismatch = conditions.iter().any(|c| matches!(c,
                OpenCondition::DimensionMismatch { field_nodes: 2, topology_nodes: 3 }
            ));
            assert!(has_mismatch,
                "expected DimensionMismatch(2,3), got {:?}", conditions);
        }
        AdmissionResult::Admitted { .. } => panic!("expected Open for dimension mismatch"),
    }

    match session.execute() {
        SessionOutput::Blocked(conditions) => {
            let has_mismatch = conditions.iter().any(|c| matches!(c,
                OpenCondition::DimensionMismatch { field_nodes: 2, topology_nodes: 3 }
            ));
            assert!(has_mismatch, "execute() should report DimensionMismatch");
        }
        _ => panic!("expected Blocked for dimension mismatch"),
    }
}

// ─── BLOCKED: invalid ρ parameter ────────────────────────────────────────────
//
// χ₀ = 0 is inadmissible: it appears in the denominator.

#[test]
fn test_invalid_chi_zero_blocked() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    session.declare_rho(0.8, 0.0, ParameterProvenance::Illustrative);  // χ₀ = 0

    match session.admit() {
        AdmissionResult::Open(conditions) => {
            let has_invalid = conditions.iter().any(|c| matches!(c,
                OpenCondition::InvalidRhoParameter(_)
            ));
            assert!(has_invalid,
                "expected InvalidRhoParameter for χ₀=0, got {:?}", conditions);
        }
        AdmissionResult::Admitted { .. } => panic!("expected Open for χ₀=0"),
    }
}

// ─── ADMITTED: illustrative parameters ───────────────────────────────────────

#[test]
fn test_session_admitted_illustrative() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);
    session.declare_cc(vec![], vec![]);

    match session.admit() {
        AdmissionResult::Admitted { parameter_provenance, kernel_open } => {
            assert_eq!(parameter_provenance, ParameterProvenance::Illustrative);
            // Kernel OPEN conditions must be surfaced at admission
            assert!(kernel_open.contains(&KernelOpenCondition::RhoMaxSelectionRule),
                "RhoMaxSelectionRule should be surfaced");
            assert!(kernel_open.contains(&KernelOpenCondition::RhoSaturatingForm),
                "RhoSaturatingForm should be surfaced");
        }
        AdmissionResult::Open(conditions) => panic!("expected Admitted, got Open({:?})", conditions),
    }
}

// ─── COMPLETE: relational output values ──────────────────────────────────────
//
// s→t→u, x=[3,5,2], ρ_base=0.8, χ₀=1.0.
// Expected from abr-simple: E[e0]=2.600000, E[e1]=2.400000
// Detection indicators: 𝟙[e0]=1, 𝟙[e1]=1

#[test]
fn test_session_execute_relational_output() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);
    session.declare_cc(vec![], vec![]);

    match session.execute() {
        SessionOutput::Complete(ExecutionResult::RelationalOutput {
            edge_field, detection_indicators, parameter_provenance, kernel_open, ..
        }) => {
            let tol = 1e-6_f64;
            assert!((edge_field.spatial[0][0] - 2.6).abs() < tol,
                "E[e0] expected ~2.6, got {}", edge_field.spatial[0][0]);
            assert!((edge_field.spatial[0][1] - 2.4).abs() < tol,
                "E[e1] expected ~2.4, got {}", edge_field.spatial[0][1]);
            assert_eq!(detection_indicators[0], 1.0, "𝟙[e0] should be 1");
            assert_eq!(detection_indicators[1], 1.0, "𝟙[e1] should be 1");
            assert_eq!(parameter_provenance, ParameterProvenance::Illustrative);
            // Kernel OPEN conditions present in complete output
            assert_eq!(kernel_open.len(), 2, "both kernel OPEN conditions should be present");
        }
        SessionOutput::Complete(ExecutionResult::CalculatedZero { .. }) =>
            panic!("expected RelationalOutput for x=[3,5,2], got CalculatedZero"),
        SessionOutput::Partial { open_r_conditions, .. } =>
            panic!("expected Complete, got Partial({:?})", open_r_conditions),
        SessionOutput::Blocked(c) =>
            panic!("expected Complete, got Blocked({:?})", c),
    }
}

// ─── COMPLETE: CalculatedZero — flat field ────────────────────────────────────
//
// Verifier correction #1: "ObservedZero" → "CalculatedZero".
// s→t→u, x=[5,5,5]: A=0 everywhere → ρ·χ=0 → R=B=A → antisymmetric term=0.
// The kernel calculated zero. Physical observation is a separate question.

#[test]
fn test_session_calculated_zero_flat_field() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_flat());
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);
    session.declare_cc(vec![], vec![]);

    match session.execute() {
        SessionOutput::Complete(ExecutionResult::CalculatedZero {
            edge_field, parameter_provenance, kernel_open, ..
        }) => {
            let tol = DETECTION_TOLERANCE;
            assert!(edge_field.spatial[0][0].abs() < tol,
                "E[e0] expected 0 for flat field, got {}", edge_field.spatial[0][0]);
            assert!(edge_field.spatial[0][1].abs() < tol,
                "E[e1] expected 0 for flat field, got {}", edge_field.spatial[0][1]);
            assert_eq!(parameter_provenance, ParameterProvenance::Illustrative);
            assert_eq!(kernel_open.len(), 2);
        }
        SessionOutput::Complete(ExecutionResult::RelationalOutput { detection_indicators, .. }) =>
            panic!("expected CalculatedZero for flat field, got RelationalOutput({:?})", detection_indicators),
        _other => panic!("expected Complete(CalculatedZero), got unexpected variant"),
    }
}

// ─── COMPLETE: multi-component detection ─────────────────────────────────────
//
// Verifier correction #4: detection evaluated across ALL components.
// 2-component field: component 0 flat (A=0), component 1 non-flat (A≠0).
// 𝟙[e] must be 1 because component 1 has nonzero antisymmetric term.

#[test]
fn test_multicomponent_detection_any_nonzero() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    // Component 0: flat (all 5.0) — A=0, zero antisymmetric term
    // Component 1: non-flat — A≠0, nonzero antisymmetric term expected
    let field = NodeField::new(vec![
        vec![5.0, 5.0, 5.0],  // component 0: flat
        vec![3.0, 5.0, 2.0],  // component 1: non-flat
    ]);
    session.declare_field(field);
    session.declare_rho(0.8, 1.0, ParameterProvenance::Illustrative);
    session.declare_cc(vec![], vec![]);

    match session.execute() {
        SessionOutput::Complete(ExecutionResult::RelationalOutput {
            detection_indicators, ..
        }) => {
            // Component 1 is non-flat → antisymmetric term nonzero → 𝟙 = 1
            assert_eq!(detection_indicators[0], 1.0,
                "𝟙[e0] must be 1 when any component has nonzero antisymmetric term");
            assert_eq!(detection_indicators[1], 1.0,
                "𝟙[e1] must be 1 when any component has nonzero antisymmetric term");
        }
        SessionOutput::Complete(ExecutionResult::CalculatedZero { .. }) =>
            panic!("expected RelationalOutput (component 1 is non-flat), got CalculatedZero"),
        _other => panic!("expected Complete(RelationalOutput)"),
    }
}

// ─── PROVENANCE: EvidenceDeclared carries record, is not verified ─────────────
//
// An EvidenceRecord alone produces EVIDENCE DECLARED — VERIFICATION OPEN.
// It does not produce OBSERVATIONALLY VERIFIED.

#[test]
fn test_evidence_declared_carries_record_not_verified() {
    let evidence = EvidenceRecord::new(
        "ρ_base=0.75 and χ₀=1.2 fitted to R[e0]=2.51, R[e1]=2.38 \
         from measurement run 2026-10-20."
    ).with_reference("run-2026-10-20.dat");

    let prov = ParameterProvenance::EvidenceDeclared(evidence);
    assert!(prov.is_observational(),  "EvidenceDeclared is non-illustrative");
    assert!(prov.is_evidence_declared(), "should be EvidenceDeclared variant");
    assert!(!prov.is_verified(),  "EvidenceDeclared must not be verified");
    assert!(!prov.is_illustrative());
    assert_eq!(prov.label(), "EVIDENCE DECLARED — VERIFICATION OPEN");

    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    session.declare_rho(0.75, 1.2, prov);
    session.declare_cc(vec![], vec![]);

    match session.admit() {
        AdmissionResult::Admitted { parameter_provenance, .. } => {
            assert!(parameter_provenance.is_evidence_declared(),
                "admitted provenance should be EvidenceDeclared");
            assert!(!parameter_provenance.is_verified(),
                "EvidenceRecord alone must not produce verified status");
        }
        AdmissionResult::Open(c) => panic!("expected Admitted, got Open({:?})", c),
    }
}

// ─── KERNEL OPEN: surfaced on admitted session ────────────────────────────────
//
// Verifier correction #6: ρ max-selection and saturating form OPEN conditions
// are preserved and surfaced whenever admission is checked or R is evaluated.

#[test]
fn test_kernel_open_conditions_surfaced_at_admission() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);

    match session.admit() {
        AdmissionResult::Admitted { kernel_open, .. } => {
            assert_eq!(kernel_open.len(), 2,
                "exactly 2 kernel OPEN conditions expected, got {:?}", kernel_open);
            assert!(kernel_open.contains(&KernelOpenCondition::RhoMaxSelectionRule));
            assert!(kernel_open.contains(&KernelOpenCondition::RhoSaturatingForm));
        }
        AdmissionResult::Open(c) => panic!("expected Admitted, got Open({:?})", c),
    }
}

#[test]
fn test_kernel_open_conditions_surfaced_at_partial_execution() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    // No ρ — partial execution

    match session.execute() {
        SessionOutput::Partial { kernel_open, .. } => {
            assert!(kernel_open.contains(&KernelOpenCondition::RhoMaxSelectionRule));
            assert!(kernel_open.contains(&KernelOpenCondition::RhoSaturatingForm));
        }
        _other => panic!("expected Partial for missing ρ"),
    }
}

#[test]
fn test_kernel_open_conditions_surfaced_at_complete_execution() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);

    match session.execute() {
        SessionOutput::Complete(ExecutionResult::RelationalOutput { kernel_open, .. }) => {
            assert_eq!(kernel_open.len(), 2);
        }
        _ => panic!("expected Complete(RelationalOutput)"),
    }
}

// ─── Execution trace: partial then full execution ────────────────────────────
//
// Demonstrates all four SessionOutput variants in order.
// Run with: cargo test -p irrs-session -- trace_execution_demo --nocapture

#[test]
fn trace_execution_demo() {
    // ── TRACE 1: Partial execution — ρ not declared ───────────────────────
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  TRACE 1 — PARTIAL EXECUTION: ρ not declared              ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("  Verifier correction #3: topology + field declared → A and B");
    println!("  computed and exposed. R blocked; open_r = [RhoNotDeclared].");
    let mut s = Session::new();
    s.declare_topology(stu_topology());
    s.declare_field(NodeField::new(vec![vec![3.0, 5.0, 2.0]]));
    s.declare_cc(vec![], vec![]);
    let output = s.execute();
    s.trace_report(&output);
    assert!(matches!(output, SessionOutput::Partial { .. }));

    // ── TRACE 2: Dimension mismatch — blocked ─────────────────────────────
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  TRACE 2 — BLOCKED: dimension mismatch (field 2, topo 3)  ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("  Verifier correction #5: field.n=2 ≠ topology.n_nodes=3");
    println!("  → DimensionMismatch, blocked before any operator runs.");
    let mut s = Session::new();
    s.declare_topology(stu_topology());
    s.declare_field(NodeField::new(vec![vec![3.0, 5.0]]));
    s.declare_rho(0.8, 1.0, ParameterProvenance::Illustrative);
    let output = s.execute();
    s.trace_report(&output);
    assert!(matches!(output, SessionOutput::Blocked(_)));

    // ── TRACE 3: Complete — illustrative ρ ───────────────────────────────
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  TRACE 3 — COMPLETE: illustrative ρ_base=0.8, χ₀=1.0     ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    let mut s = Session::new();
    s.declare_topology(stu_topology());
    s.declare_field(NodeField::new(vec![vec![3.0, 5.0, 2.0]]));
    s.declare_rho(0.8, 1.0, ParameterProvenance::Illustrative);
    s.declare_cc(vec![], vec![]);
    let output = s.execute();
    s.trace_report(&output);
    assert!(matches!(&output, SessionOutput::Complete(ExecutionResult::RelationalOutput { .. })));

    // ── TRACE 4: Complete — EvidenceDeclared ─────────────────────────────
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  TRACE 4 — COMPLETE: evidence declared, verification open  ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("  EvidenceRecord present; provenance = EVIDENCE DECLARED —");
    println!("  VERIFICATION OPEN. EvidenceRecord alone is not verification.");
    let evidence = EvidenceRecord::new(
        "ρ_base=0.75 and χ₀=1.2 fitted to R[e0]=2.51, R[e1]=2.38 \
         from measurement run 2026-10-20 at node field x=[3.1, 5.2, 1.9]."
    ).with_reference("run-2026-10-20.dat");
    let mut s = Session::new();
    s.declare_topology(stu_topology());
    s.declare_field(NodeField::new(vec![vec![3.0, 5.0, 2.0]]));
    s.declare_rho(0.75, 1.2, ParameterProvenance::EvidenceDeclared(evidence));
    s.declare_cc(vec![], vec![]);
    let output = s.execute();
    s.trace_report(&output);
    assert!(matches!(&output, SessionOutput::Complete(ExecutionResult::RelationalOutput { .. })));

    // ── TRACE 5: CalculatedZero — flat field ─────────────────────────────
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  TRACE 5 — CALCULATED ZERO: flat field x=[5,5,5]          ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("  Verifier correction #1: CalculatedZero names what the kernel");
    println!("  computed — not a physical observation.");
    let mut s = Session::new();
    s.declare_topology(stu_topology());
    s.declare_field(NodeField::new(vec![vec![5.0, 5.0, 5.0]]));
    s.declare_rho(0.8, 1.0, ParameterProvenance::Illustrative);
    s.declare_cc(vec![], vec![]);
    let output = s.execute();
    s.trace_report(&output);
    assert!(matches!(&output, SessionOutput::Complete(ExecutionResult::CalculatedZero { .. })));

    println!("\n  All five traces completed.");
}

// ─── DETECTION_TOLERANCE is a named constant ─────────────────────────────────
//
// Verifier correction #4: tolerance must be explicit, not a magic number.

#[test]
fn test_detection_tolerance_constant_is_explicit() {
    // The constant must be accessible and have the documented value.
    assert_eq!(DETECTION_TOLERANCE, 1e-12_f64,
        "DETECTION_TOLERANCE should be 1e-12");
    // Confirm it's used consistently: a value just above tolerance triggers 𝟙=1.
    // (Manual verification that the constant is the boundary.)
    let just_above = DETECTION_TOLERANCE * 1.001;
    let just_below = DETECTION_TOLERANCE * 0.999;
    assert!(just_above > DETECTION_TOLERANCE);
    assert!(just_below < DETECTION_TOLERANCE);
}

// ─── PROVENANCE: ObservationallyVerified requires inspectable comparison ──────
//
// Task #10 — Verifier requirement #4:
// Verified status requires a VerificationRecord with declared observables,
// parameter derivation, comparison results, and a verification source.
// Synthetic fixture used; labeled as such in evidence description.

fn synthetic_verification_record_passing() -> VerificationRecord {
    // s→t→u, x=[3,5,2], ρ_base=0.8, χ₀=1.0
    // Expected: E[e0]≈2.6, E[e1]≈2.4
    VerificationRecord {
        evidence: EvidenceRecord::new(
            "Synthetic fixture — not physical experimental evidence. \
             ρ_base=0.8, χ₀=1.0 chosen to match calculated E[e0]=2.6, E[e1]=2.4 \
             at x=[3,5,2]."
        ).with_reference("synthetic-fixture-Task10"),
        declared_observables: vec![
            DeclaredObservable {
                label: "R[e0]".into(),
                value: 2.6,
                unit: None,
                measurement_provenance: "Synthetic fixture value".into(),
            },
            DeclaredObservable {
                label: "R[e1]".into(),
                value: 2.4,
                unit: None,
                measurement_provenance: "Synthetic fixture value".into(),
            },
        ],
        parameter_derivation:
            "ρ_base=0.8 and χ₀=1.0 chosen to reproduce declared fixture values.".into(),
        verification_source: "Task #10 synthetic fixture — internal consistency check".into(),
        comparisons: vec![
            ObservationalComparison {
                label:      "R[e0]".into(),
                observed:   2.6,
                tolerance:  1e-3,
                edge_index: 0,
                component:  0,
            },
            ObservationalComparison {
                label:      "R[e1]".into(),
                observed:   2.4,
                tolerance:  1e-3,
                edge_index: 1,
                component:  0,
            },
        ],
        open_items: vec![],
    }
}

#[test]
fn test_verified_requires_inspectable_comparison() {
    let rec = synthetic_verification_record_passing();

    let prov = ParameterProvenance::ObservationallyVerified(rec);
    assert!(prov.is_verified(), "should be verified");
    assert!(!prov.is_evidence_declared());
    assert!(!prov.is_illustrative());
    // label() without evaluation always returns base label; CORRESPONDENCE
    // FAILED is determined only after the session runs the kernel.
    assert_eq!(prov.label(), "OBSERVATIONALLY VERIFIED");

    // Execute; session computes evaluated_comparisons against actual kernel output.
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    session.declare_rho(0.8, 1.0, prov);
    session.declare_cc(vec![], vec![]);

    match session.execute() {
        SessionOutput::Complete(ExecutionResult::RelationalOutput {
            edge_field, parameter_provenance, evaluated_comparisons, ..
        }) => {
            // Math result is kernel-calculated; not rewritten by provenance.
            assert!((edge_field.spatial[0][0] - 2.6).abs() < 1e-6,
                "E[e0] should be ~2.6; got {}", edge_field.spatial[0][0]);
            assert!((edge_field.spatial[0][1] - 2.4).abs() < 1e-6,
                "E[e1] should be ~2.4; got {}", edge_field.spatial[0][1]);
            assert!(parameter_provenance.is_verified());

            // Session evaluated comparisons against actual kernel output.
            assert_eq!(evaluated_comparisons.len(), 2,
                "two comparisons declared; two should be evaluated");
            assert!(evaluated_comparisons.iter().all(|c| c.passes()),
                "all comparisons should pass for synthetic fixture");
            // label_with_evaluation reflects actual kernel output.
            assert_eq!(
                parameter_provenance.label_with_evaluation(&evaluated_comparisons),
                "OBSERVATIONALLY VERIFIED"
            );
        }
        other => panic!("expected Complete(RelationalOutput), got {}",
            match other {
                SessionOutput::Complete(_) => "Complete(other)",
                SessionOutput::Partial { .. } => "Partial",
                SessionOutput::Blocked(_) => "Blocked",
            }
        ),
    }
}

// ─── PROVENANCE: failed correspondence does not rewrite math ─────────────────
//
// Task #10 — Verifier requirement #5:
// A VerificationRecord with failing comparisons labels the output
// OBSERVATIONALLY VERIFIED — CORRESPONDENCE FAILED.
// The kernel-calculated result is NOT rewritten to match the declared observed value.

#[test]
fn test_failed_correspondence_does_not_rewrite_math() {
    // s→t→u, x=[3,5,2], ρ_base=0.8, χ₀=1.0 → E[e0]≈2.6
    // Declare observed=3.0 (deliberately mismatches kernel output 2.6) tol=0.01.
    // The session detects the mismatch; the mathematical result is not rewritten.
    let rec = VerificationRecord {
        evidence: EvidenceRecord::new(
            "Synthetic fixture — deliberate mismatch to test correspondence failure. \
             Observed R[e0] declared as 3.0; kernel computes ~2.6."
        ),
        declared_observables: vec![
            DeclaredObservable {
                label: "R[e0]".into(),
                value: 3.0,
                unit: None,
                measurement_provenance: "Synthetic fixture — deliberate mismatch".into(),
            },
        ],
        parameter_derivation: "ρ_base=0.8, χ₀=1.0 (not derived from declared 3.0)".into(),
        verification_source: "Task #11 mismatch fixture — internal".into(),
        comparisons: vec![
            ObservationalComparison {
                label:      "R[e0]".into(),
                observed:   3.0,   // declared: 3.0
                tolerance:  0.01,  // |3.0 − 2.6| ≈ 0.4 >> 0.01 → FAIL
                edge_index: 0,
                component:  0,
                // No `calculated` — session looks this up from actual kernel output
            },
        ],
        open_items: vec![],
    };

    let prov = ParameterProvenance::ObservationallyVerified(rec);
    // label() alone does not know about kernel output → base label only.
    assert_eq!(prov.label(), "OBSERVATIONALLY VERIFIED");
    assert!(prov.is_verified());

    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    session.declare_rho(0.8, 1.0, prov);
    session.declare_cc(vec![], vec![]);

    match session.execute() {
        SessionOutput::Complete(ExecutionResult::RelationalOutput {
            edge_field, parameter_provenance, evaluated_comparisons, ..
        }) => {
            // Kernel result stands at ~2.6; NOT rewritten to the declared 3.0.
            assert!((edge_field.spatial[0][0] - 2.6).abs() < 1e-6,
                "math result must not be rewritten: expected ~2.6, got {}",
                edge_field.spatial[0][0]);
            assert!((edge_field.spatial[0][0] - 3.0).abs() > 0.3,
                "result must differ from declared observed value 3.0");

            // Session detected the mismatch via evaluated_comparisons.
            assert_eq!(evaluated_comparisons.len(), 1);
            let ec = &evaluated_comparisons[0];
            assert!(!ec.passes(), "comparison must fail");
            assert!((ec.observed - 3.0).abs() < 1e-9, "observed = 3.0");
            assert!((ec.calculated - 2.6).abs() < 1e-3, "calculated ≈ 2.6 from kernel");
            assert!((ec.residual() - (3.0 - 2.6)).abs() < 1e-3);

            // label_with_evaluation reflects actual mismatch.
            assert_eq!(
                parameter_provenance.label_with_evaluation(&evaluated_comparisons),
                "OBSERVATIONALLY VERIFIED — CORRESPONDENCE FAILED"
            );
        }
        other => panic!("expected Complete(RelationalOutput), got {}",
            match other {
                SessionOutput::Complete(_) => "Complete(other)",
                SessionOutput::Partial { .. } => "Partial",
                SessionOutput::Blocked(_) => "Blocked",
            }
        ),
    }
}

// ─── PROVENANCE: open items in VerificationRecord are surfaced ────────────────
//
// open_items records unverified aspects. A record with open_items is not
// fully verified even if all comparisons pass.

#[test]
fn test_verification_record_with_open_items_not_fully_verified() {
    // open_items present → label_with_evaluation returns CORRESPONDENCE FAILED
    // even when all evaluated comparisons pass.
    let rec = VerificationRecord {
        evidence: EvidenceRecord::new(
            "Synthetic fixture — open item present to test surfacing."
        ),
        declared_observables: vec![],
        parameter_derivation: "illustrative derivation".into(),
        verification_source: "Task #11 open-item fixture".into(),
        comparisons: vec![
            ObservationalComparison {
                label:      "R[e0]".into(),
                observed:   2.6,
                tolerance:  1e-3,
                edge_index: 0,
                component:  0,
            },
        ],
        open_items: vec![
            "χ₀ derivation from first principles not completed (Kernel V8 §11.2 OPEN)".into(),
        ],
    };

    assert!(!rec.open_items.is_empty(), "open_items present");

    let prov = ParameterProvenance::ObservationallyVerified(rec);
    assert!(prov.is_verified());

    // Execute a session; comparison passes (observed=2.6 matches kernel ~2.6).
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    session.declare_rho(0.8, 1.0, prov);
    session.declare_cc(vec![], vec![]);

    match session.execute() {
        SessionOutput::Complete(ExecutionResult::RelationalOutput {
            parameter_provenance, evaluated_comparisons, ..
        }) => {
            // Comparison passes (kernel output ≈ 2.6 matches declared 2.6).
            assert!(evaluated_comparisons.iter().all(|c| c.passes()),
                "comparison passes");
            // But open_items is non-empty → label_with_evaluation must flag it.
            assert_eq!(
                parameter_provenance.label_with_evaluation(&evaluated_comparisons),
                "OBSERVATIONALLY VERIFIED — CORRESPONDENCE FAILED",
                "open_items present → not fully verified even when comparisons pass"
            );
        }
        other => panic!("expected Complete(RelationalOutput), got {}",
            match other {
                SessionOutput::Complete(_) => "Complete(other)",
                SessionOutput::Partial { .. } => "Partial",
                SessionOutput::Blocked(_) => "Blocked",
            }
        ),
    }
}

// ─── Provenance trace: three states ──────────────────────────────────────────
//
// Demonstrates ILLUSTRATIVE / EVIDENCE DECLARED — VERIFICATION OPEN /
// OBSERVATIONALLY VERIFIED in sequence.
// Run with: cargo test -p irrs-session -- trace_provenance_states --nocapture

#[test]
fn trace_provenance_states() {
    // ── TRACE A: Illustrative ─────────────────────────────────────────────
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  TRACE A — ILLUSTRATIVE provenance                        ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("  Parameters declared to demonstrate operator mechanics.");
    println!("  No observational correspondence is claimed or implied.");
    let mut s = Session::new();
    s.declare_topology(stu_topology());
    s.declare_field(stu_field_345());
    s.declare_rho(0.8, 1.0, ParameterProvenance::Illustrative);
    s.declare_cc(vec![], vec![]);
    let output = s.execute();
    s.trace_report(&output);
    assert!(matches!(&output, SessionOutput::Complete(ExecutionResult::RelationalOutput {
        parameter_provenance, ..
    }) if parameter_provenance.is_illustrative()));

    // ── TRACE B: Evidence declared, verification open ─────────────────────
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  TRACE B — EVIDENCE DECLARED — VERIFICATION OPEN          ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("  EvidenceRecord present; not independently evaluated.");
    println!("  EvidenceRecord alone does not constitute verification.");
    let evidence = EvidenceRecord::new(
        "ρ_base=0.75 and χ₀=1.2 fitted to R[e0]=2.51, R[e1]=2.38 \
         from measurement run 2026-10-20 at node field x=[3.1, 5.2, 1.9]."
    ).with_reference("run-2026-10-20.dat");
    let mut s = Session::new();
    s.declare_topology(stu_topology());
    s.declare_field(stu_field_345());
    s.declare_rho(0.75, 1.2, ParameterProvenance::EvidenceDeclared(evidence));
    s.declare_cc(vec![], vec![]);
    let output = s.execute();
    s.trace_report(&output);
    assert!(matches!(&output, SessionOutput::Complete(ExecutionResult::RelationalOutput {
        parameter_provenance, ..
    }) if parameter_provenance.is_evidence_declared()));

    // ── TRACE C: Observationally verified — synthetic fixture ────────────
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║  TRACE C — OBSERVATIONALLY VERIFIED (synthetic fixture)   ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!("  VerificationRecord with declared observables, derivation,");
    println!("  comparison results, and verification source. Fixture labeled.");
    let rec = synthetic_verification_record_passing();
    let mut s = Session::new();
    s.declare_topology(stu_topology());
    s.declare_field(stu_field_345());
    s.declare_rho(0.8, 1.0, ParameterProvenance::ObservationallyVerified(rec));
    s.declare_cc(vec![], vec![]);
    let output = s.execute();
    s.trace_report(&output);
    assert!(matches!(&output, SessionOutput::Complete(ExecutionResult::RelationalOutput {
        parameter_provenance, ..
    }) if parameter_provenance.is_verified()));

    println!("\n  All three provenance-state traces completed.");
}
