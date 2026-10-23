/// IRRS Session — Admission Tests
///
/// Verifies the D/M-driven admission check and execution path.
///
/// Tests:
///   OPEN cases:     topology not declared, no interior locus, field not declared,
///                   rho not declared
///   ADMITTED cases: illustrative parameters (s→t→u), relational output values,
///                   observed zero (flat field)

use irrs_kernel::operators::{
    DeclaredRelations, EdgeProvenance, NodeField,
};
use irrs_session::admission::{
    AdmissionResult, ExecutionResult, OpenCondition, ParameterProvenance,
};
use irrs_session::session::Session;

// ─── Helper: standard s→t→u topology ─────────────────────────────────────────
//
// 3 nodes: s(0) → t(1) → u(2)
// 2 edges: e0=(0,1), e1=(1,2), both Continuation
// Interior locus: node 1 — target of e0, source of e1

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

// ─── OPEN: topology not declared ─────────────────────────────────────────────

#[test]
fn test_session_open_topology_not_declared() {
    let session = Session::new();
    // No declarations at all
    let result = session.admit();
    match result {
        AdmissionResult::Open(conditions) => {
            assert!(
                conditions.contains(&OpenCondition::TopologyNotDeclared),
                "expected TopologyNotDeclared, got {:?}", conditions
            );
        }
        AdmissionResult::Admitted { .. } => {
            panic!("expected Open, got Admitted");
        }
    }
}

// ─── OPEN: no interior locus ─────────────────────────────────────────────────
//
// 2 nodes, 1 edge: s(0) → t(1)
// Node 0 has no incoming edge; node 1 has no outgoing edge.
// No interior locus exists.

#[test]
fn test_session_open_no_interior_locus() {
    let mut session = Session::new();
    let rel = DeclaredRelations::from_edges_with_provenance(
        2,
        vec![(0, 1)],
        vec![EdgeProvenance::Continuation],
    );
    session.declare_topology(rel);
    session.declare_field(NodeField::new(vec![vec![3.0, 5.0]]));
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);

    let result = session.admit();
    match result {
        AdmissionResult::Open(conditions) => {
            assert!(
                conditions.contains(&OpenCondition::NoInteriorLocus),
                "expected NoInteriorLocus, got {:?}", conditions
            );
            // Should not also claim topology is missing
            assert!(
                !conditions.contains(&OpenCondition::TopologyNotDeclared),
                "topology was declared; unexpected TopologyNotDeclared"
            );
        }
        AdmissionResult::Admitted { .. } => {
            panic!("expected Open (no interior locus), got Admitted");
        }
    }
}

// ─── OPEN: field not declared ────────────────────────────────────────────────

#[test]
fn test_session_open_field_not_declared() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    // No field declared
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);

    let result = session.admit();
    match result {
        AdmissionResult::Open(conditions) => {
            assert!(
                conditions.contains(&OpenCondition::NodeFieldNotDeclared),
                "expected NodeFieldNotDeclared, got {:?}", conditions
            );
        }
        AdmissionResult::Admitted { .. } => {
            panic!("expected Open (field missing), got Admitted");
        }
    }
}

// ─── OPEN: rho not declared ──────────────────────────────────────────────────

#[test]
fn test_session_open_rho_not_declared() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    // No rho declared

    let result = session.admit();
    match result {
        AdmissionResult::Open(conditions) => {
            assert!(
                conditions.contains(&OpenCondition::RhoNotDeclared),
                "expected RhoNotDeclared, got {:?}", conditions
            );
        }
        AdmissionResult::Admitted { .. } => {
            panic!("expected Open (rho missing), got Admitted");
        }
    }
}

// ─── ADMITTED: illustrative parameters ───────────────────────────────────────
//
// Full s→t→u declaration with illustrative ρ_base=0.8, χ₀=1.0.
// Expect Admitted with Illustrative provenance.

#[test]
fn test_session_admitted_illustrative() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);
    session.declare_cc(vec![], vec![]);

    let result = session.admit();
    match result {
        AdmissionResult::Admitted { parameter_provenance } => {
            assert_eq!(
                parameter_provenance,
                ParameterProvenance::Illustrative,
                "expected Illustrative provenance"
            );
        }
        AdmissionResult::Open(conditions) => {
            panic!("expected Admitted, got Open({:?})", conditions);
        }
    }
}

// ─── EXECUTE: relational output values ───────────────────────────────────────
//
// s→t→u, x=[3,5,2], ρ_base=0.8, χ₀=1.0.
// Expected from abr-simple example: E[e0]=2.600000, E[e1]=2.400000
// Detection indicators: 𝟙[e0]=1, 𝟙[e1]=1 (antisymmetric term ≠ 0)

#[test]
fn test_session_execute_relational_output() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_345());
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);
    session.declare_cc(vec![], vec![]);

    let result = session.execute().expect("execution should succeed for admitted session");
    match result {
        ExecutionResult::RelationalOutput { edge_field, detection_indicators, parameter_provenance } => {
            let tol = 1e-6_f64;
            assert!(
                (edge_field.spatial[0][0] - 2.6).abs() < tol,
                "E[e0] expected ~2.6, got {}", edge_field.spatial[0][0]
            );
            assert!(
                (edge_field.spatial[0][1] - 2.4).abs() < tol,
                "E[e1] expected ~2.4, got {}", edge_field.spatial[0][1]
            );
            assert_eq!(detection_indicators[0], 1.0, "𝟙[e0] should be 1");
            assert_eq!(detection_indicators[1], 1.0, "𝟙[e1] should be 1");
            assert_eq!(parameter_provenance, ParameterProvenance::Illustrative);
        }
        ExecutionResult::ObservedZero { .. } => {
            panic!("expected RelationalOutput for x=[3,5,2], got ObservedZero");
        }
    }
}

// ─── EXECUTE: observed zero — flat field ─────────────────────────────────────
//
// s→t→u, x=[5,5,5] — uniform node field.
// A = x[tgt] - x[src] = 0 for all edges.
// All A values are zero → ρ·χ = 0 → R = B = A → E = 0 antisymmetric.
// Expected: ObservedZero

#[test]
fn test_session_observed_zero_flat_field() {
    let mut session = Session::new();
    session.declare_topology(stu_topology());
    session.declare_field(stu_field_flat());
    let (rho_base, chi_0, prov) = illustrative_rho();
    session.declare_rho(rho_base, chi_0, prov);
    session.declare_cc(vec![], vec![]);

    let result = session.execute().expect("execution should succeed for admitted session");
    match result {
        ExecutionResult::ObservedZero { edge_field, parameter_provenance } => {
            // E values should be zero (uniform field → A = 0 → B = 0 → R = 0)
            let tol = 1e-12_f64;
            assert!(
                edge_field.spatial[0][0].abs() < tol,
                "E[e0] expected 0 for flat field, got {}", edge_field.spatial[0][0]
            );
            assert!(
                edge_field.spatial[0][1].abs() < tol,
                "E[e1] expected 0 for flat field, got {}", edge_field.spatial[0][1]
            );
            assert_eq!(parameter_provenance, ParameterProvenance::Illustrative);
        }
        ExecutionResult::RelationalOutput { detection_indicators, .. } => {
            panic!(
                "expected ObservedZero for flat field, got RelationalOutput with indicators {:?}",
                detection_indicators
            );
        }
    }
}
