/// Kernel V8 — Canonical Relational Operators
///
/// The operators in this file exist because of one primitive: a directed
/// relation declared from observable information. They cannot be applied to
/// anything that was not declared through M, and they produce no output that
/// carries meaning beyond the declared boundary.
///
/// The primitive is defined in `lib.rs`. Read it first.
///
/// ───────────────────────────────────────────────────────────────────────────
///
/// This file is `operators.rs`. What acts is here.
/// Derived invariants are in `derived_invariants.rs`. What is produced is there.
/// Do not add derived invariants to this file.
///
/// The operator chain is A → B → R (ABR kernel) or Δ → Σ (primary kernel).
/// Neither chain is independent of the declared directed relations that precede
/// it. `DeclaredRelations` is not an input container — it is the primitive
/// itself, encoded as a structure the operators can act on.
///
/// Frozen at Kernel V8. No modification is authorized.
/// Where this file and `kernel_reference.md` differ, this file governs.
/// A difference is a finding to be returned to Origin.
///
/// Source authority: Metatron Dynamics Kernel V8 (2026-09-27; discipline 2026-09-29).

// ─── EdgeProvenance ──────────────────────────────────────────────────────────

/// Every declared edge carries one of three provenances.
/// A relation with no declared provenance is fabricated within D.
/// Section 12.7 of kernel_reference.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeProvenance {
    /// Within-family spatial continuation — declared from observable or device geometry.
    /// Open boundary: no wraparound.
    Continuation,
    /// Cross-family spatial coupling — declared from physical coupling mechanism.
    Coupling,
    /// Relational-evolutionary — connects E_prior[e] to E_current[e] across one
    /// declared process step. Not declared in the spatial edge list. Use PersistenceState.
    Persistence,
}

// ─── DeclaredRelations ───────────────────────────────────────────────────────

/// The foundational primitive encoded as a structure.
///
/// `DeclaredRelations` is the directed observable relation made available for
/// operator action. It is not a convenience wrapper or an input container.
/// It is the primitive itself — the thing without which no operator in this
/// file has meaning or can be applied.
///
/// Every edge in `DeclaredRelations` must carry a declared provenance: the
/// observable or physical basis from which the relation was declared through M.
/// An edge without provenance is a fabrication within D and is rejected at
/// construction.
///
/// Production constructor — requires explicit provenance for every edge.
/// `Persistence` edges are not declared here; use `PersistenceState`.
/// Section 12.1 of kernel_reference.md.
pub struct DeclaredRelations {
    pub n_nodes: usize,
    n_edges: usize,
    edges: Vec<(usize, usize)>,       // edges[e] = (src, tgt)
    provenance: Vec<EdgeProvenance>,
    /// out[i] — edge indices leaving node i (ABR B and R)
    pub out: Vec<Vec<usize>>,
    /// inc[i] — edge indices entering node i (ABR B and R)
    pub inc: Vec<Vec<usize>>,
    /// adj_plus[e] = succ(e) — edges starting where e ends (Primary Σ, ABR B/R)
    pub adj_plus: Vec<Vec<usize>>,
    /// adj_minus[e] = pred(e) — edges ending where e starts (Primary Σ, ABR B/R)
    pub adj_minus: Vec<Vec<usize>>,
}

impl DeclaredRelations {
    pub fn from_edges_with_provenance(
        n_nodes: usize,
        edges: Vec<(usize, usize)>,
        provenance: Vec<EdgeProvenance>,
    ) -> Self {
        assert_eq!(
            edges.len(), provenance.len(),
            "every declared edge must have a declared provenance"
        );
        let n_edges = edges.len();

        let mut out = vec![vec![]; n_nodes];
        let mut inc = vec![vec![]; n_nodes];
        for (e, &(src, tgt)) in edges.iter().enumerate() {
            assert!(
                src < n_nodes && tgt < n_nodes,
                "edge {} ({}, {}) references node outside declared range [0, {})",
                e, src, tgt, n_nodes
            );
            out[src].push(e);
            inc[tgt].push(e);
        }

        // adj_plus[e] = edges f such that source(f) == target(e)
        // adj_minus[e] = edges f such that target(f) == source(e)
        let mut adj_plus  = vec![vec![]; n_edges];
        let mut adj_minus = vec![vec![]; n_edges];
        for e in 0..n_edges {
            let tgt_e = edges[e].1;
            let src_e = edges[e].0;
            adj_plus[e]  = out[tgt_e].clone();
            adj_minus[e] = inc[src_e].clone();
        }

        DeclaredRelations {
            n_nodes, n_edges, edges, provenance, out, inc, adj_plus, adj_minus,
        }
    }

    pub fn n_edges(&self)            -> usize         { self.n_edges }
    pub fn src(&self, e: usize)      -> usize         { self.edges[e].0 }
    pub fn tgt(&self, e: usize)      -> usize         { self.edges[e].1 }
    pub fn provenance(&self, e: usize) -> EdgeProvenance { self.provenance[e] }
}

// ─── NodeField ───────────────────────────────────────────────────────────────

/// Numerical projection through M over declared nodes.
/// Input to both kernels.
///
/// data[c][i] — component c, node i.
/// All values must be finite (∈ D).
/// Section 12.2 of kernel_reference.md.
pub struct NodeField {
    pub data: Vec<Vec<f64>>,
    /// n_components
    pub k: usize,
    /// n_nodes
    pub n: usize,
}

impl NodeField {
    pub fn new(data: Vec<Vec<f64>>) -> Self {
        assert!(!data.is_empty(), "NodeField must have at least one component");
        let n = data[0].len();
        for (c, col) in data.iter().enumerate() {
            assert_eq!(col.len(), n,
                "component {} has {} values but component 0 has {}", c, col.len(), n);
            for (i, &v) in col.iter().enumerate() {
                assert!(v.is_finite(),
                    "NodeField component {} node {}: value {} is not finite (∈ D)", c, i, v);
            }
        }
        let k = data.len();
        NodeField { data, k, n }
    }
}

// ─── PrimaryEdgeField ────────────────────────────────────────────────────────

/// Output of primary kernel operators.
/// field[c][e] — component c, edge e.
/// Section 12.3 of kernel_reference.md.
pub struct PrimaryEdgeField {
    pub field: Vec<Vec<f64>>,
    pub k: usize,
    pub n_edges: usize,
}

// ─── EdgeField ───────────────────────────────────────────────────────────────

/// Output of ABR kernel operators.
/// Section 12.4 of kernel_reference.md.
pub struct EdgeField {
    /// spatial[c][e] — component c, spatial edge e
    pub spatial: Vec<Vec<f64>>,
    /// comp[p][i] — component pair p, node i
    pub comp: Vec<Vec<f64>>,
    /// declared component relations
    pub comp_pairs: Vec<(usize, usize)>,
    pub k: usize,
}

// ─── PersistenceState ────────────────────────────────────────────────────────

/// Persistence state — the prior EdgeField required to evaluate persistence operators.
///
/// Can only be constructed from the kernel output of a completed prior declared
/// observation. Not from a model, parameter value, or static configuration.
///
/// `EdgeField::zero`, `PersistenceState::cold_start`, and `is_cold_start`
/// were removed at Kernel V8. There is no cold-start constructor.
/// Section 12.5 of kernel_reference.md.
pub struct PersistenceState {
    pub e_prior: EdgeField,
}

impl PersistenceState {
    /// Construct from the kernel output of a completed declared observation.
    /// `e_prior` must be the `EdgeField` returned by a prior call to
    /// `operator_e` or `operator_e_v5` over a declared observation.
    pub fn from_prior(e_prior: EdgeField) -> Self {
        PersistenceState { e_prior }
    }
}

// ─── PersistenceOutput ───────────────────────────────────────────────────────

/// Output of the persistence operator path A_p → B_p → R_p.
/// Section 12.6 of kernel_reference.md.
pub struct PersistenceOutput {
    pub a_persistence: Vec<Vec<f64>>,   // [c][e]
    pub b_persistence: Vec<Vec<f64>>,   // [c][e]
    pub r_persistence: Vec<Vec<f64>>,   // [c][e]
}

// ─── M-declaration enforcement ────────────────────────────────────────────────

/// Verify that ρ_base and χ₀ were declared by M.
///
/// The kernel holds no value for either and supplies no default.
/// This is not a missing implementation — it is the correct structure.
///
/// ρ_base (dimensionless ceiling) and χ₀ (reference scale for saturation)
/// are domain-specific. The complexity and characteristic scales of the
/// declared domain determine their applicable values. A hydrogen emission edge,
/// a geomagnetic current layer, and a neural coupling site each inhabit a
/// different relational scale and will have incommensurable values.
///
/// An application layer that assigns the same ρ_base and χ₀ across
/// different physical domains without independent M-declaration for each
/// is not applying the kernel correctly.
///
/// Section 11.3 of kernel_reference.md.
pub fn require_rho_declaration(rho_base: f64, chi_0: f64) {
    assert!(
        rho_base.is_finite(),
        "ρ_base must be declared by M as a finite value"
    );
    assert!(
        chi_0.is_finite() && chi_0 > 0.0,
        "χ₀ must be declared by M as a finite positive value in the dimension of χ"
    );
}

/// Verify that cc was declared by M for every declared component pair.
///
/// Cross-topology coupling coefficients (cc) are domain-specific for the
/// same reason ρ_base and χ₀ are: the coupling between two observable
/// families (e.g., spatial and angular momentum, electric and magnetic,
/// position and momentum) is determined by the relational structure of
/// the declared domain, not by any universal value. cc = 0.5 as a fixed
/// default was retired at Kernel V7.1 precisely because it imposed
/// cross-domain uniformity where none is warranted.
///
/// An empty cc with empty pairs is valid: it declares that no cross-topology
/// coupling is applicable in this domain at this declaration.
///
/// Section 11.3 of kernel_reference.md.
pub fn require_cc_declaration(cc: &[f64], pairs: &[(usize, usize)]) {
    assert_eq!(
        cc.len(), pairs.len(),
        "cc must be declared by M for every declared component pair"
    );
    assert!(
        cc.iter().all(|v| v.is_finite()),
        "cc values declared by M must be finite"
    );
}

// ─── ρ computation ────────────────────────────────────────────────────────────

/// Compute ρ for the primary kernel (Δ→Σ).
///
/// χ[i] = max |Δ(x)[c][e]| over all components c and edges e incident to i.
/// ρ[i] = ρ_base · χ[i] / (χ₀ + χ[i])
/// ρ[e] = ρ[source(e)]
///
/// Domain-specificity: χ[i] is calculable from declared observables alone.
/// ρ_base and χ₀ are not — they are domain-specific parameters that M must
/// supply. See `require_rho_declaration` for the structural reason.
///
/// OPEN: max-selection rule requires Origin declaration (section 11.2, 18).
///   This is a separate question from domain-specificity: even after M declares
///   ρ_base and χ₀ for this domain, the rule for combining χ over multiple
///   components and incident edges remains unconfirmed in the kernel reference.
/// OPEN: saturating form requires Origin declaration (section 11.2, 18).
///   The functional form ρ_base · χ/(χ₀+χ) is stated but its derivation is
///   not closed. A different form would still require domain-specific parameters.
pub fn compute_rho_primary(
    delta: &PrimaryEdgeField,
    rel: &DeclaredRelations,
    rho_base: f64,
    chi_0: f64,
) -> Vec<f64> {
    require_rho_declaration(rho_base, chi_0);
    let n_edges = rel.n_edges();

    let mut chi = vec![0.0_f64; rel.n_nodes];
    for e in 0..n_edges {
        let src = rel.src(e);
        let tgt = rel.tgt(e);
        for c in 0..delta.k {
            let v = delta.field[c][e].abs();
            if v > chi[src] { chi[src] = v; }
            if v > chi[tgt] { chi[tgt] = v; }
        }
    }

    let mut rho = vec![0.0_f64; n_edges];
    for e in 0..n_edges {
        let src = rel.src(e);
        rho[e] = rho_base * chi[src] / (chi_0 + chi[src]);
    }
    rho
}

/// Compute ρ for the ABR kernel (A→B→R).
///
/// χ[i] = max |A(x)[c][e]| over all components c and edges e incident to i.
/// ρ[i] = ρ_base · χ[i] / (χ₀ + χ[i])
/// ρ[e] = ρ[source(e)]
///
/// Domain-specificity: χ[i] is calculable from declared x alone.
/// ρ_base and χ₀ are domain-specific; see `require_rho_declaration`.
/// The same OPEN conditions as `compute_rho_primary` apply.
pub fn compute_rho(
    a_field: &EdgeField,
    rel: &DeclaredRelations,
    rho_base: f64,
    chi_0: f64,
) -> Vec<f64> {
    require_rho_declaration(rho_base, chi_0);
    let n_edges = rel.n_edges();

    let mut chi = vec![0.0_f64; rel.n_nodes];
    for e in 0..n_edges {
        let src = rel.src(e);
        let tgt = rel.tgt(e);
        for c in 0..a_field.k {
            let v = a_field.spatial[c][e].abs();
            if v > chi[src] { chi[src] = v; }
            if v > chi[tgt] { chi[tgt] = v; }
        }
    }

    let mut rho = vec![0.0_f64; n_edges];
    for e in 0..n_edges {
        let src = rel.src(e);
        rho[e] = rho_base * chi[src] / (chi_0 + chi[src]);
    }
    rho
}

/// Compute ρ for persistence operators.
/// Driven by A_persistence magnitudes.
///
/// Domain-specificity applies identically: ρ_base and χ₀ are domain-specific
/// regardless of whether the input field is a spatial A or a persistence A_p.
/// The same declared values from M apply across both kernel paths within a
/// single declared domain.
pub fn compute_rho_persistence(
    a_persistence: &[Vec<f64>],
    rel: &DeclaredRelations,
    rho_base: f64,
    chi_0: f64,
) -> Vec<f64> {
    require_rho_declaration(rho_base, chi_0);
    let n_edges = rel.n_edges();
    let k = a_persistence.len();

    let mut chi = vec![0.0_f64; rel.n_nodes];
    for e in 0..n_edges {
        let src = rel.src(e);
        let tgt = rel.tgt(e);
        for c in 0..k {
            let v = a_persistence[c][e].abs();
            if v > chi[src] { chi[src] = v; }
            if v > chi[tgt] { chi[tgt] = v; }
        }
    }

    let mut rho = vec![0.0_f64; n_edges];
    for e in 0..n_edges {
        let src = rel.src(e);
        rho[e] = rho_base * chi[src] / (chi_0 + chi[src]);
    }
    rho
}

// ─── Primary kernel: Δ → Σ ───────────────────────────────────────────────────

/// Δ operator.
///
/// Δ(x)[e] = x[s] − x[t], for each declared edge e = (s, t).
/// Section 11.1 of kernel_reference.md.
pub fn operator_delta(x: &NodeField, rel: &DeclaredRelations) -> PrimaryEdgeField {
    let n_edges = rel.n_edges();
    let mut field = vec![vec![0.0_f64; n_edges]; x.k];
    for e in 0..n_edges {
        let s = rel.src(e);
        let t = rel.tgt(e);
        for c in 0..x.k {
            field[c][e] = x.data[c][s] - x.data[c][t];
        }
    }
    PrimaryEdgeField { field, k: x.k, n_edges }
}

/// Σ operator.
///
/// Σ(g)[e] = g[e] + ρ[e] · (Σ_{adj⁺(e)} g − Σ_{adj⁻(e)} g)
/// g = Δ(x); immediate neighbours only.
/// Section 11.1 of kernel_reference.md.
pub fn operator_sigma(
    delta: &PrimaryEdgeField,
    rel: &DeclaredRelations,
    rho_base: f64,
    chi_0: f64,
) -> PrimaryEdgeField {
    let rho = compute_rho_primary(delta, rel, rho_base, chi_0);
    let n_edges = rel.n_edges();
    let mut field = vec![vec![0.0_f64; n_edges]; delta.k];
    for e in 0..n_edges {
        for c in 0..delta.k {
            let sum_plus:  f64 = rel.adj_plus[e].iter().map(|&f| delta.field[c][f]).sum();
            let sum_minus: f64 = rel.adj_minus[e].iter().map(|&f| delta.field[c][f]).sum();
            field[c][e] = delta.field[c][e] + rho[e] * (sum_plus - sum_minus);
        }
    }
    PrimaryEdgeField { field, k: delta.k, n_edges }
}

/// E_primary = Σ(Δ(x)).
/// Returns (Δ output, Σ output).
/// Section 11.1 of kernel_reference.md.
pub fn operator_e_primary(
    x: &NodeField,
    rel: &DeclaredRelations,
    rho_base: f64,
    chi_0: f64,
) -> (PrimaryEdgeField, PrimaryEdgeField) {
    let delta = operator_delta(x, rel);
    let sigma = operator_sigma(&delta, rel, rho_base, chi_0);
    (delta, sigma)
}

// ─── ABR kernel: A → B → R ───────────────────────────────────────────────────

/// A operator.
///
/// A(x)[e] = x[s] − x[t] (formula identical to Δ); spatial edges.
/// Component pairs (a,b): comp[p][i] = x[a][i] − x[b][i] at each node.
/// Section 11.1 of kernel_reference.md.
pub fn operator_a(
    f: &NodeField,
    rel: &DeclaredRelations,
    pairs: &[(usize, usize)],
) -> EdgeField {
    let n_edges = rel.n_edges();
    let k = f.k;

    let mut spatial = vec![vec![0.0_f64; n_edges]; k];
    for e in 0..n_edges {
        let s = rel.src(e);
        let t = rel.tgt(e);
        for c in 0..k {
            spatial[c][e] = f.data[c][s] - f.data[c][t];
        }
    }

    let mut comp = vec![vec![0.0_f64; f.n]; pairs.len()];
    for (p, &(a, b)) in pairs.iter().enumerate() {
        assert!(a < k && b < k,
            "component pair ({}, {}) out of range for NodeField with {} components", a, b, k);
        for i in 0..f.n {
            comp[p][i] = f.data[a][i] - f.data[b][i];
        }
    }

    EdgeField { spatial, comp, comp_pairs: pairs.to_vec(), k }
}

/// B operator.
///
/// B(g)[e] = g[e] + Σ_{f ∈ succ(e)} g[f]; immediate successors only.
/// Terminal edges accumulate nothing (empty succ → no contribution).
/// Section 11.1 of kernel_reference.md.
pub fn operator_b(g: &EdgeField, rel: &DeclaredRelations) -> EdgeField {
    let n_edges = rel.n_edges();
    let mut spatial = vec![vec![0.0_f64; n_edges]; g.k];
    for e in 0..n_edges {
        for c in 0..g.k {
            let sum_succ: f64 = rel.adj_plus[e].iter().map(|&f| g.spatial[c][f]).sum();
            spatial[c][e] = g.spatial[c][e] + sum_succ;
        }
    }
    EdgeField {
        spatial,
        comp: g.comp.clone(),
        comp_pairs: g.comp_pairs.clone(),
        k: g.k,
    }
}

/// R operator.
///
/// Spatial term:
///   R(g)[c][e] = g[c][e] + ρ[src(e)] · (Σ_{succ(e)} g[c] − Σ_{pred(e)} g[c])
///
/// Cross-topology term: for each declared component pair p = (a, b),
///   the component difference at src(e) is added to spatial channels a and b,
///   scaled by cc[p]. Direction preserved from the declared pair ordering.
///
/// Section 11.1 of kernel_reference.md.
pub fn operator_r(
    bg: &EdgeField,
    rel: &DeclaredRelations,
    rho: &[f64],
    cc: &[f64],
) -> EdgeField {
    require_cc_declaration(cc, &bg.comp_pairs);
    let n_edges = rel.n_edges();
    let mut spatial = vec![vec![0.0_f64; n_edges]; bg.k];

    for e in 0..n_edges {
        let src = rel.src(e);

        // Spatial antisymmetric term
        for c in 0..bg.k {
            let sum_succ: f64 = rel.adj_plus[e].iter().map(|&f| bg.spatial[c][f]).sum();
            let sum_pred: f64 = rel.adj_minus[e].iter().map(|&f| bg.spatial[c][f]).sum();
            spatial[c][e] = bg.spatial[c][e] + rho[e] * (sum_succ - sum_pred);
        }

        // Cross-topology coupling for each declared component pair
        for (p, &(a, b)) in bg.comp_pairs.iter().enumerate() {
            let cross = bg.comp[p][src] * cc[p];
            spatial[a][e] += cross;
            spatial[b][e] -= cross;
        }
    }

    EdgeField {
        spatial,
        comp: bg.comp.clone(),
        comp_pairs: bg.comp_pairs.clone(),
        k: bg.k,
    }
}

/// E (Phase 1): R(B(A(x)), ρ(A(x)))
///
/// Full ABR path. Requires at least one interior locus (full-operator
/// admissibility condition, section 10 of kernel_reference.md).
/// Section 11.1 of kernel_reference.md.
pub fn operator_e(
    f: &NodeField,
    rel: &DeclaredRelations,
    pairs: &[(usize, usize)],
    cc: &[f64],
    rho_base: f64,
    chi_0: f64,
) -> EdgeField {
    let a = operator_a(f, rel, pairs);
    let rho = compute_rho(&a, rel, rho_base, chi_0);
    let b = operator_b(&a, rel);
    operator_r(&b, rel, &rho, cc)
}

// ─── Persistence operators ────────────────────────────────────────────────────

/// A_persistence operator.
///
/// A_persistence[e] = E_current[e] − E_prior[e]; direction fixed.
/// Section 11.1 of kernel_reference.md.
pub fn operator_a_persistence(
    e_current: &EdgeField,
    e_prior: &EdgeField,
) -> Vec<Vec<f64>> {
    assert_eq!(e_current.k, e_prior.k, "A_persistence: component count mismatch");
    let n_edges = e_current.spatial[0].len();
    assert_eq!(e_prior.spatial[0].len(), n_edges, "A_persistence: edge count mismatch");
    let mut out = vec![vec![0.0_f64; n_edges]; e_current.k];
    for c in 0..e_current.k {
        for e in 0..n_edges {
            out[c][e] = e_current.spatial[c][e] - e_prior.spatial[c][e];
        }
    }
    out
}

/// B_persistence operator.
///
/// B form over A_persistence: immediate successor accumulation over relational steps.
/// Section 11.1 of kernel_reference.md.
pub fn operator_b_persistence(
    a_persistence: &[Vec<f64>],
    rel: &DeclaredRelations,
) -> Vec<Vec<f64>> {
    let n_edges = rel.n_edges();
    let k = a_persistence.len();
    let mut out = vec![vec![0.0_f64; n_edges]; k];
    for e in 0..n_edges {
        for c in 0..k {
            let sum_succ: f64 = rel.adj_plus[e].iter().map(|&f| a_persistence[c][f]).sum();
            out[c][e] = a_persistence[c][e] + sum_succ;
        }
    }
    out
}

/// R_persistence operator.
///
/// R form over B_persistence; spatial component only.
/// Section 11.1 of kernel_reference.md.
pub fn operator_r_persistence(
    b_persistence: &[Vec<f64>],
    rel: &DeclaredRelations,
    rho_persistence: &[f64],
) -> Vec<Vec<f64>> {
    let n_edges = rel.n_edges();
    let k = b_persistence.len();
    let mut out = vec![vec![0.0_f64; n_edges]; k];
    for e in 0..n_edges {
        for c in 0..k {
            let sum_succ: f64 = rel.adj_plus[e].iter().map(|&f| b_persistence[c][f]).sum();
            let sum_pred: f64 = rel.adj_minus[e].iter().map(|&f| b_persistence[c][f]).sum();
            out[c][e] = b_persistence[c][e] + rho_persistence[e] * (sum_succ - sum_pred);
        }
    }
    out
}

/// E (Phases 1+2): Phase 1 (A→B→R), then A_p → B_p → R_p.
///
/// Usage pattern:
///   At the first declared observation, call `operator_e` (Phase 1 only).
///   At the second and subsequent observations, call this function.
///   `persistence_state` must be constructed from `PersistenceState::from_prior(e1)`.
///
/// Returns (Phase-1 EdgeField, PersistenceOutput).
/// Section 11.6 of kernel_reference.md.
pub fn operator_e_v5(
    f: &NodeField,
    rel: &DeclaredRelations,
    pairs: &[(usize, usize)],
    cc: &[f64],
    persistence_state: &PersistenceState,
    rho_base: f64,
    chi_0: f64,
) -> (EdgeField, PersistenceOutput) {
    // Phase 1: A→B→R
    let e_current = operator_e(f, rel, pairs, cc, rho_base, chi_0);

    // Phase 2: A_p → B_p → R_p
    let a_p = operator_a_persistence(&e_current, &persistence_state.e_prior);
    let b_p = operator_b_persistence(&a_p, rel);
    let rho_p = compute_rho_persistence(&a_p, rel, rho_base, chi_0);
    let r_p = operator_r_persistence(&b_p, rel, &rho_p);

    let persistence = PersistenceOutput {
        a_persistence: a_p,
        b_persistence: b_p,
        r_persistence: r_p,
    };
    (e_current, persistence)
}

// ─── Structural checks ────────────────────────────────────────────────────────

/// Relational isolation check.
///
/// Returns true when every edge has empty adj⁺ and adj⁻ (no successors
/// or predecessors). In this case Σ's and R's antisymmetric terms are
/// structurally zero.
/// Section 11.5 of kernel_reference.md.
pub fn relational_isolation(rel: &DeclaredRelations) -> bool {
    let n_edges = rel.n_edges();
    (0..n_edges).all(|e| rel.adj_plus[e].is_empty() && rel.adj_minus[e].is_empty())
}

/// Declared edge-image admissibility check.
///
/// Returns true when no edge vector is the negation of another declared edge vector.
/// `tol` is an IMPLEMENTATION-ONLY NUMERICAL TOLERANCE guarding finite-precision
/// arithmetic. It is not a property of the observed system.
/// Section 11.5 of kernel_reference.md.
pub fn declared_edge_image_admissibility_check(delta: &PrimaryEdgeField, tol: f64) -> bool {
    let n_edges = delta.n_edges;
    for e in 0..n_edges {
        for f in (e + 1)..n_edges {
            // Check if delta[·][e] ≈ -delta[·][f] for all components
            let negation = (0..delta.k).all(|c| {
                (delta.field[c][e] + delta.field[c][f]).abs() <= tol
            });
            if negation {
                return false;
            }
        }
    }
    true
}

/// Extract the antisymmetric term of Σ separately.
///
/// antisymmetric_term[c][e] = ρ[e] · (Σ_{adj⁺(e)} Δ[c] − Σ_{adj⁻(e)} Δ[c])
///
/// `tol` is an IMPLEMENTATION-ONLY NUMERICAL TOLERANCE. Not a system property.
/// Section 11.5 of kernel_reference.md.
pub fn antisymmetric_term(
    delta: &PrimaryEdgeField,
    rel: &DeclaredRelations,
    rho_base: f64,
    chi_0: f64,
) -> PrimaryEdgeField {
    let rho = compute_rho_primary(delta, rel, rho_base, chi_0);
    let n_edges = rel.n_edges();
    let mut field = vec![vec![0.0_f64; n_edges]; delta.k];
    for e in 0..n_edges {
        for c in 0..delta.k {
            let sum_plus:  f64 = rel.adj_plus[e].iter().map(|&f| delta.field[c][f]).sum();
            let sum_minus: f64 = rel.adj_minus[e].iter().map(|&f| delta.field[c][f]).sum();
            field[c][e] = rho[e] * (sum_plus - sum_minus);
        }
    }
    PrimaryEdgeField { field, k: delta.k, n_edges }
}
