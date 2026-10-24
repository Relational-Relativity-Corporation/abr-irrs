/// Kernel V8 — Derived Invariants, Part I (Confirmed)
///
/// The quantities in this file are produced by the operators acting on declared
/// directed relations. They do not exist independently of that process. Each
/// one is readable only after a complete operator chain has acted on a declared
/// relational structure — and its numerical existence alone does not establish
/// correspondence with the observable. Correspondence is declared through M.
///
/// The primitive that makes these quantities possible is defined in `lib.rs`.
///
/// ───────────────────────────────────────────────────────────────────────────
///
/// This file is `derived_invariants.rs`. What is produced is here.
/// Operators are in `operators.rs`. What acts is there.
/// Do not add operators to this file.
///
/// Part I quantities are confirmed invariants.
/// Part II statements (open derivation) are NOT in this file and may not be
/// cited as proven consequences of the operators.
/// Nothing in Part I depends on any statement in Part II.
///
/// Frozen at Kernel V8. No modification is authorized.
/// Section 13 of kernel_reference.md.

// ─── Declared measurement references ─────────────────────────────────────────
//
// Section 13.1 of kernel_reference.md.
// Observable provenance for each constant is stated below.
//
// H_PLANCK and C_DECLARED do not appear in any operator formula.
// They enter D only inside the M mapping hc/λ → ε[e] at Atomic Region photon edges.
// Do not introduce C_DECLARED into operator expressions or kernel output.

/// ℏ — reduced Planck constant.
/// Observable provenance: spectroscopic frequency-energy measurement.
pub const HBAR: f64 = 1.054_571_817e-34; // J·s

/// e — elementary charge unit.
/// Observable provenance: Millikan oil-drop and subsequent direct measurements.
pub const E_UNIT: f64 = 1.602_176_634e-19; // C

/// ι_e — relational inertia of the electron.
/// Observable provenance: Penning trap oscillation frequency under declared non-acceleration.
/// Numerical value identical to PDG electron mass entry. Primitive interpretation changes.
pub const I_UNIT_ELECTRON: f64 = 9.109_383_701_5e-31; // kg

/// h — Planck constant.
/// Observable provenance: spectroscopic frequency-energy measurement.
/// Enters D only inside M mapping hc/λ → ε[e] at Atomic Region photon edges.
pub const H_PLANCK: f64 = 6.626_070_15e-34; // J·s

/// c — declared speed of light.
/// Observable provenance: BIPM 2019 interferometric measurement.
/// Enters D only inside M mapping hc/λ → ε[e] at Atomic Region photon edges.
pub const C_DECLARED: f64 = 2.997_924_58e8; // m/s

// ─── ε[e] — Relational energy contrast ──────────────────────────────────────
//
// ε[e] = |Δ(ι)[e]| · 𝟙[e]
// Support: Derived. Confirmed: 22 transition edges across PDG and NIST.
// Section 13.3 of kernel_reference.md.

/// ε[e] — relational energy contrast at a spatial edge.
///
/// `delta_iota_e` is the Δ operator output at edge e over the ι (relational inertia) field.
/// `indicator` must be 0.0 or 1.0 (the 𝟙[e] detection indicator).
///
/// Kernel V8 HOLD: depends on the 𝟙[e] audit (A-3 open condition). No change.
pub fn epsilon_e(delta_iota_e: f64, indicator: f64) -> f64 {
    debug_assert!(
        indicator == 0.0 || indicator == 1.0,
        "indicator must be 0.0 or 1.0; got {}", indicator
    );
    delta_iota_e.abs() * indicator
}

// ─── 𝟙[e] — Detection indicator ─────────────────────────────────────────────
//
// 𝟙[e] = 1  iff  R(A(x))[e]_antisymmetric ≠ 0
// 𝟙[e] = 0  otherwise
// Support: Derived (OC-12 resolved).
// Section 13.4 of kernel_reference.md.

/// 𝟙[e] — detection indicator.
///
/// `r_antisymmetric_e` is the antisymmetric term of R at edge e.
/// `tol` is an IMPLEMENTATION-ONLY NUMERICAL TOLERANCE guarding finite-precision
/// arithmetic. It is not a property of the observed system and is never reported as one.
///
/// Returns 1.0 if |r_antisymmetric_e| > tol, otherwise 0.0.
///
/// Kernel V8 AUDIT PENDING: determine whether 0.0 always means an evaluated
/// R output whose antisymmetric term is 0.0, or is ever used where no relation
/// was declared through M. Follows Origin determination of empty-adjacency
/// semantics (audit A-3).
pub fn detection_indicator(r_antisymmetric_e: f64, tol: f64) -> f64 {
    if r_antisymmetric_e.abs() > tol { 1.0 } else { 0.0 }
}

// ─── ε[e] at photon edges ─────────────────────────────────────────────────────
//
// ε[e] = hc / λ
// M mapping: spectrometer-reported wavelength λ (m) → ε[e] (J).
// hc/λ is part of M at the Atomic Region.
// ι at the photon locus: not declared through M (A-11 CLOSED).
// Under the declared-information rule, ι is not evaluated at photon edges.
// Confirmed: 14 transitions (NIST ASD v5.12).
// Section 13.5 of kernel_reference.md.

/// ε[e] at a photon edge.
///
/// `lambda_m` is the spectrometer-reported wavelength in meters,
/// declared from instrument through M.
pub fn epsilon_photon_edge(lambda_m: f64) -> f64 {
    H_PLANCK * C_DECLARED / lambda_m
}

/// τ[v] at a photon-emitting locus.
///
/// τ[v] = ℏ / ε[e]
pub fn tau_photon_edge(lambda_m: f64) -> f64 {
    HBAR / epsilon_photon_edge(lambda_m)
}

// ─── N[v] at photon-emitting loci ────────────────────────────────────────────
//
// N[v] = τ_stability[v] · ε[e] / ℏ
// Count of declared relational intervals within the measured stability interval.
// Dimensionless. Confirmed: H n=3 (Hα) → N ≈ 4.58e6.
// Section 13.7 of kernel_reference.md.

/// N[v] — count of relational cycles within the measured stability interval.
///
/// `tau_stability_s` is the measured stability interval in seconds.
/// `lambda_m` is the photon wavelength in meters.
pub fn n_relational_cycles(tau_stability_s: f64, lambda_m: f64) -> f64 {
    tau_stability_s * epsilon_photon_edge(lambda_m) / HBAR
}

// ─── τ[v] — Relational progression interval (unified) ───────────────────────
//
// τ[v] = ℏ / |A_persistence[e_coupling] · cos θ[e_v, e_coupling]|
// Support: Derived (OC-13 resolved).
// Section 13.6 of kernel_reference.md.

/// τ[v] — unified relational progression interval.
///
/// `a_persistence_coupling` is A_persistence at the coupling edge.
/// `cos_theta` is the cosine of the declared angle between e_v and e_coupling.
pub fn tau_v(a_persistence_coupling: f64, cos_theta: f64) -> f64 {
    HBAR / (a_persistence_coupling * cos_theta).abs()
}

/// τ[v] — isolated locus case (no coupling edge declared).
///
/// `e_v` is the relational energy contrast at the locus's edge.
pub fn tau_v_isolated(e_v: f64) -> f64 {
    HBAR / e_v
}

// ─── θ[e] — Equilibrium bond angle ──────────────────────────────────────────
//
// v[e] = B(g)[e] · θ̂[e]
// Equilibrium condition: Σᵢ v[eᵢ] = 0 at each declared locus.
// Confirmed exact: 2 bonds → 180°, 3 bonds → 120°, 4 bonds → 109.47°.
// Section 13.8 of kernel_reference.md.

/// θ[e] — equilibrium bond angle in radians for n_bonds declared edges at a locus.
///
/// Returns `None` for n_bonds outside {2, 3, 4}.
pub fn equilibrium_bond_angle_rad(n_bonds: usize) -> Option<f64> {
    match n_bonds {
        2 => Some(std::f64::consts::PI),                       // 180°
        3 => Some(2.0 * std::f64::consts::PI / 3.0),          // 120°
        4 => Some(f64::acos(-1.0 / 3.0)),                     // 109.47...°
        _ => None,
    }
}

/// θ[e] — equilibrium bond angle in degrees for n_bonds declared edges at a locus.
///
/// Returns `None` for n_bonds outside {2, 3, 4}.
pub fn equilibrium_bond_angle_deg(n_bonds: usize) -> Option<f64> {
    equilibrium_bond_angle_rad(n_bonds).map(|r| r.to_degrees())
}

// ─── Φ[v] — Coherence potential (node) ───────────────────────────────────────
//
// Φ[v] = ι[v] · τ[v]
// Support: Observed (formal derivation OC-Φ-1 pending).
// Confirmed across 29 declared loci spanning 78 decades in Φ.
// Section 13.9 of kernel_reference.md.

/// Φ[v] — coherence potential at a declared locus.
///
/// `iota_v` is the relational inertia at v.
/// `tau_v_val` is the relational progression interval at v.
pub fn phi_v(iota_v: f64, tau_v_val: f64) -> f64 {
    iota_v * tau_v_val
}

/// log₁₀ Φ[v].
pub fn phi_v_log10(iota_v: f64, tau_v_val: f64) -> f64 {
    phi_v(iota_v, tau_v_val).log10()
}

// ─── Φ[S] — Coherence potential (collective) ─────────────────────────────────
//
// Φ[S] = ι[S] · τ[S]  where  ι[S] = Σᵢ ι[vᵢ]
// Accumulation condition: Φ[S] > Σᵢ Φ[vᵢ]
// Support: Observed. Confirmed across six accumulation transitions, four scale transitions.
// Zero violations.
// Section 13.10 of kernel_reference.md.

/// Φ[S] — collective coherence potential.
///
/// `iota_s` is the sum of relational inertia values over the declared collective.
/// `tau_s_val` is the relational progression interval of the collective.
pub fn phi_s(iota_s: f64, tau_s_val: f64) -> f64 {
    iota_s * tau_s_val
}

/// Accumulation condition: Φ[S] > Σᵢ Φ[vᵢ].
///
/// Returns true when the collective coherence potential exceeds the sum of
/// its constituents' individual potentials.
pub fn accumulation_condition(phi_s_val: f64, phi_components: &[f64]) -> bool {
    phi_s_val > phi_components.iter().sum::<f64>()
}

// ─── J[v] — R output multiplicity ────────────────────────────────────────────
//
// J[v] = (declared edges at v) / 2
// R output states = 2J[v] + 1
// What the established literature calls spin is R output multiplicity.
// What the established literature calls Zeeman components is what the framework
// computes as 2J[v] + 1 distinct R output states.
// Support: Derived. Confirmed 29/52 (VR-J-01 — completion in progress).
// Section 13.11 of kernel_reference.md.

/// J[v] — R output multiplicity at a declared locus.
///
/// `n_declared_edges_at_v` is the number of declared spatial edges at node v.
pub fn j_v(n_declared_edges_at_v: usize) -> f64 {
    n_declared_edges_at_v as f64 / 2.0
}

/// Count of distinct R output states at a declared locus.
///
/// Returns 2J[v] + 1.
pub fn r_output_states(n_declared_edges_at_v: usize) -> f64 {
    2.0 * j_v(n_declared_edges_at_v) + 1.0
}
