/// ABR Simple — Minimal Canonical ABR Calculation
///
/// This example executes one complete, traceable relational calculation
/// using the canonical Kernel V8 operators (A→B→R).
///
/// It is NOT a DataFeel application. It demonstrates the mathematical process
/// as defined in `kernel_reference.md`, section 11.
///
/// ─── DECLARATION (Origin) ───────────────────────────────────────────────────
///
/// Domain: minimal relational graph, single observable family.
///
/// System: three declared loci (nodes 0, 1, 2) connected by two declared edges.
///   Node 0 (s) → Node 1 (t) → Node 2 (u)
///   Edge 0: s→t, provenance Continuation
///   Edge 1: t→u, provenance Continuation
///
/// Interior locus: node 1 (t) is the target of edge 0 and the source of edge 1.
///   adj⁺[0] = {1}  (edge 1 starts where edge 0 ends)
///   This establishes the full-operator admissibility condition: A→B→R is admissible.
///
/// Observable: a single declared scalar field over three loci.
///   x[0][0] = 3.0  (locus s, component 0)
///   x[0][1] = 5.0  (locus t, component 0)
///   x[0][2] = 2.0  (locus u, component 0)
///   These values are declared through M from observable evidence.
///
/// M-declared operator parameters:
///   ρ_base = 0.8   (dimensionless ceiling; declared by M)
///   χ₀ = 1.0       (reference for saturating map; declared by M)
///   No component pairs declared (minimal single-component system).
///   cc = []        (no cross-topology coupling declared)
///
/// Expected mathematical trace is printed below each computation step.
///
/// ─── WORKFLOW ───────────────────────────────────────────────────────────────
///
/// Step 1 (Declare): Origin declares D, M, loci, edges, provenance.
///                   Done above and encoded below.
/// Step 2 (Execute): Generator executes the applicable kernel unchanged.
///                   Full-operator admissibility condition holds: A→B→R.
/// Step 3 (Verify):  Verifier traces every operation back to declared observables.
///                   Mathematical trace is preserved below for independent inspection.

use irrs_kernel::operators::{
    DeclaredRelations, EdgeProvenance, NodeField,
    operator_a, operator_b, operator_r, compute_rho,
    require_rho_declaration, require_cc_declaration,
};

fn main() {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  ABR Simple — Kernel V8 Canonical ABR Calculation");
    println!("  Metatron Dynamics. Bounded over D. No claim beyond D.");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();

    // ── DECLARATION ─────────────────────────────────────────────────────────

    println!("── DECLARATION ─────────────────────────────────────────────────────");
    println!("  n_nodes : 3  (loci s=0, t=1, u=2)");
    println!("  n_edges : 2  (e0: s→t, e1: t→u)");
    println!("  edge 0  : (0, 1)  Continuation  (s → t)");
    println!("  edge 1  : (1, 2)  Continuation  (t → u)");
    println!("  Interior locus: t (node 1) — target of e0, source of e1");
    println!("  Full-operator admissibility condition: SATISFIED");
    println!("  Kernel: A → B → R");
    println!();

    let rel = DeclaredRelations::from_edges_with_provenance(
        3,
        vec![(0, 1), (1, 2)],
        vec![EdgeProvenance::Continuation, EdgeProvenance::Continuation],
    );

    // adj_plus[0] = edges starting where edge 0 ends (target=1, which is source of edge 1)
    // adj_plus[1] = edges starting where edge 1 ends (target=2, none)
    println!("  adj⁺[0] = {:?}  (successors of e0)", rel.adj_plus[0]);
    println!("  adj⁺[1] = {:?}  (successors of e1, terminal)", rel.adj_plus[1]);
    println!("  adj⁻[0] = {:?}  (predecessors of e0, source=0, no edges end at 0)", rel.adj_minus[0]);
    println!("  adj⁻[1] = {:?}  (predecessors of e1, source=1, e0 ends at 1)", rel.adj_minus[1]);
    println!();

    // ── OBSERVED VALUES THROUGH M ────────────────────────────────────────────

    println!("── NUMERICAL PROJECTION THROUGH M ──────────────────────────────────");
    println!("  x[0][0] = 3.0  (locus s, component 0, declared from observable)");
    println!("  x[0][1] = 5.0  (locus t, component 0, declared from observable)");
    println!("  x[0][2] = 2.0  (locus u, component 0, declared from observable)");
    println!();

    let x = NodeField::new(vec![vec![3.0, 5.0, 2.0]]);

    // ── M-DECLARED PARAMETERS ────────────────────────────────────────────────

    let rho_base = 0.8_f64;
    let chi_0    = 1.0_f64;
    let pairs: Vec<(usize, usize)> = vec![];
    let cc:    Vec<f64>            = vec![];

    require_rho_declaration(rho_base, chi_0);
    require_cc_declaration(&cc, &pairs);

    println!("── M-DECLARED OPERATOR PARAMETERS ──────────────────────────────────");
    println!("  ρ_base  = {rho_base}  (M-supplied; dimensionless ceiling)");
    println!("  χ₀      = {chi_0}  (M-supplied; reference for saturating map)");
    println!("  pairs   = []  (no cross-topology coupling declared)");
    println!("  cc      = []  (no cross-topology coefficients)");
    println!();

    // ── A OPERATOR ──────────────────────────────────────────────────────────
    //
    // A(x)[e] = x[src(e)] − x[tgt(e)]
    // e0: A[0][0] = x[0][0] − x[0][1] = 3.0 − 5.0 = −2.0
    // e1: A[0][1] = x[0][1] − x[0][2] = 5.0 − 2.0 =  3.0

    let a = operator_a(&x, &rel, &pairs);

    println!("── A OPERATOR: A(x)[e] = x[src(e)] − x[tgt(e)] ────────────────────");
    println!("  A[0][0] (e0, s→t) = x[0][0] − x[0][1] = 3.0 − 5.0 = {}",
        a.spatial[0][0]);
    println!("  A[0][1] (e1, t→u) = x[0][1] − x[0][2] = 5.0 − 2.0 = {}",
        a.spatial[0][1]);
    println!();

    // ── ρ COMPUTATION ────────────────────────────────────────────────────────
    //
    // χ[i] = max |A[c][e]| over all components c and edges incident to node i
    // Node 0 (s): incident to e0 only → χ[0] = |A[0][0]| = 2.0
    // Node 1 (t): incident to e0 and e1 → χ[1] = max(2.0, 3.0) = 3.0
    // Node 2 (u): incident to e1 only → χ[2] = |A[0][1]| = 3.0
    //
    // ρ[i] = ρ_base · χ[i] / (χ₀ + χ[i])
    // ρ[0] = 0.8 · 2.0 / (1.0 + 2.0) = 1.6 / 3.0 ≈ 0.5333
    // ρ[1] = 0.8 · 3.0 / (1.0 + 3.0) = 2.4 / 4.0 = 0.6
    //
    // ρ[e] = ρ[source(e)]
    // ρ[e0] = ρ[src=0] ≈ 0.5333
    // ρ[e1] = ρ[src=1] = 0.6

    let rho = compute_rho(&a, &rel, rho_base, chi_0);

    println!("── ρ COMPUTATION: ρ[e] = ρ_base · χ[src(e)] / (χ₀ + χ[src(e)]) ───");
    println!("  χ[0] (node s, incident to e0 only)        = |A[0][0]| = {}",
        a.spatial[0][0].abs());
    println!("  χ[1] (node t, incident to e0 and e1)      = max({}, {}) = {}",
        a.spatial[0][0].abs(), a.spatial[0][1].abs(),
        a.spatial[0][0].abs().max(a.spatial[0][1].abs()));
    println!("  χ[2] (node u, incident to e1 only)        = |A[0][1]| = {}",
        a.spatial[0][1].abs());
    println!("  ρ[e0] = ρ[src=0] = {:.4}  (0.8 · 2.0 / 3.0)", rho[0]);
    println!("  ρ[e1] = ρ[src=1] = {:.4}  (0.8 · 3.0 / 4.0)", rho[1]);
    println!();

    // ── B OPERATOR ──────────────────────────────────────────────────────────
    //
    // B(g)[e] = g[e] + Σ_{f ∈ succ(e)} g[f]
    // adj⁺[0] = {1}  → B[0][0] = A[0][0] + A[0][1] = −2.0 + 3.0 = 1.0
    // adj⁺[1] = {}   → B[0][1] = A[0][1]             = 3.0 (terminal)

    let b = operator_b(&a, &rel);

    println!("── B OPERATOR: B(g)[e] = g[e] + Σ_{{succ(e)}} g[f] ─────────────────");
    println!("  B[0][0] (e0, succ={{e1}}) = A[0][0] + A[0][1] = −2.0 + 3.0 = {}",
        b.spatial[0][0]);
    println!("  B[0][1] (e1, terminal)   = A[0][1]            = {}",
        b.spatial[0][1]);
    println!();

    // ── R OPERATOR ──────────────────────────────────────────────────────────
    //
    // R(g)[e] = g[e] + ρ[src(e)] · (Σ_{succ(e)} g − Σ_{pred(e)} g)
    //
    // e0: succ = {e1}, pred = {} (nothing ends at src=0)
    //   sum_succ = B[0][1] = 3.0
    //   sum_pred = 0.0
    //   R[0][0] = B[0][0] + ρ[e0] · (3.0 − 0.0)
    //           = 1.0 + 0.5333 · 3.0 ≈ 2.6
    //
    // e1: succ = {} (nothing starts at tgt=2), pred = {e0} (e0 ends at src(e1)=1)
    //   sum_succ = 0.0
    //   sum_pred = B[0][0] = 1.0
    //   R[0][1] = B[0][1] + ρ[e1] · (0.0 − 1.0)
    //           = 3.0 + 0.6 · (−1.0) = 2.4

    let r = operator_r(&b, &rel, &rho, &cc);

    println!("── R OPERATOR: R(g)[e] = g[e] + ρ[src(e)]·(Σ_succ g − Σ_pred g) ──");
    println!("  e0: succ={{e1}}, pred={{}}");
    println!("    R[0][0] = B[0][0] + ρ[e0] · (B[0][1] − 0)");
    println!("            = {:.4} + {:.4} · {:.4} = {:.6}",
        b.spatial[0][0], rho[0], b.spatial[0][1], r.spatial[0][0]);
    println!("  e1: succ={{}}, pred={{e0}}");
    println!("    R[0][1] = B[0][1] + ρ[e1] · (0 − B[0][0])");
    println!("            = {:.4} + {:.4} · ({:.4}) = {:.6}",
        b.spatial[0][1], rho[1], -b.spatial[0][0], r.spatial[0][1]);
    println!();

    // ── RESULT ──────────────────────────────────────────────────────────────

    println!("── RESULT: E = R(B(A(x))) ──────────────────────────────────────────");
    println!("  E[0][0] (edge e0, s→t) = {:.6}", r.spatial[0][0]);
    println!("  E[0][1] (edge e1, t→u) = {:.6}", r.spatial[0][1]);
    println!();
    println!("── MATHEMATICAL TRACE ──────────────────────────────────────────────");
    println!("  Origin declaration → M → D");
    println!("    x: s=3.0, t=5.0, u=2.0");
    println!("  → A (directed differences)");
    println!("    A[e0] = −2.0, A[e1] = 3.0");
    println!("  → ρ (M-supplied parameters ρ_base={}, χ₀={})", rho_base, chi_0);
    println!("    ρ[e0] ≈ {:.6}, ρ[e1] ≈ {:.6}", rho[0], rho[1]);
    println!("  → B (successor accumulation, interior locus t absorbs e1)");
    println!("    B[e0] = {:.4}, B[e1] = {:.4}", b.spatial[0][0], b.spatial[0][1]);
    println!("  → R (antisymmetric neighbor term, cross-topology: none declared)");
    println!("    R[e0] ≈ {:.6}, R[e1] ≈ {:.6}", r.spatial[0][0], r.spatial[0][1]);
    println!();
    println!("── CORRESPONDENCE ──────────────────────────────────────────────────");
    println!("  Calculated result preserved exactly. No suppression applied.");
    println!("  Comparison with observables: not declared in this example.");
    println!("  Return to Origin for observable comparison declaration.");
    println!();
    println!("── OPEN CONDITIONS ─────────────────────────────────────────────────");
    println!("  ρ max-selection rule: OPEN (Origin declaration required)");
    println!("  ρ saturating form: OPEN (Origin declaration required)");
    println!("  No persistence evaluation (first declared observation only).");
    println!("  To evaluate persistence: call PersistenceState::from_prior(E)");
    println!("  at the next declared observation, then operator_e_v5.");
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Operators hold no agency. Responsibility stays with Origin.");
    println!("  Metatron Dynamics Kernel V8.");
    println!("═══════════════════════════════════════════════════════════════════");
}
