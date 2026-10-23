/// IRRS Session — D/M Declaration and Kernel Execution
///
/// Accepts a bounded D and M, checks admission, executes Kernel V8 operators,
/// and reports the mathematical trace with parameter provenance.
///
/// Session is stateful: declarations accumulate until execute() is called.
/// No operator is applied before admit() returns Admitted.
///
/// Continuous Relational Process Rule: every evaluation enters an already-
/// existing process. This session layer does not construct synthetic cold
/// starts. PersistenceState is not available here; the session executes the
/// spatial A→B→R path only.

use irrs_kernel::operators::{
    DeclaredRelations, NodeField,
    operator_a, operator_b, operator_r,
    compute_rho,
    require_rho_declaration, require_cc_declaration,
};

use crate::admission::{
    AdmissionResult, ExecutionResult, OpenCondition, ParameterProvenance,
};

// ─── Session ─────────────────────────────────────────────────────────────────

/// Interactive session for D/M-driven relational computation.
///
/// Workflow:
///   1. `declare_topology` — supply DeclaredRelations
///   2. `declare_field`    — supply NodeField
///   3. `declare_rho`      — supply ρ_base, χ₀, and provenance label
///   4. `declare_cc`       — supply cross-topology coupling (may be empty)
///   5. `admit`            — check prerequisites; inspect OpenConditions if any
///   6. `execute`          — run frozen Kernel V8 operators; get ExecutionResult
///   7. `trace_report`     — print mathematical trace and provenance
pub struct Session {
    rel:   Option<DeclaredRelations>,
    field: Option<NodeField>,
    /// (ρ_base, χ₀, provenance)
    rho:   Option<(f64, f64, ParameterProvenance)>,
    cc:    Vec<f64>,
    pairs: Vec<(usize, usize)>,
}

impl Session {
    /// Create an empty session. No declarations are present.
    pub fn new() -> Self {
        Session {
            rel:   None,
            field: None,
            rho:   None,
            cc:    vec![],
            pairs: vec![],
        }
    }

    /// Declare the relational topology (D).
    ///
    /// Every edge must carry an explicit provenance. Persistence edges are not
    /// declared here; they belong to PersistenceState in a separate evaluation.
    pub fn declare_topology(&mut self, rel: DeclaredRelations) {
        self.rel = Some(rel);
    }

    /// Declare the node field (M: O → D).
    ///
    /// data[c][i] is the numerical projection through M at component c, node i.
    /// All values must be finite (∈ D).
    pub fn declare_field(&mut self, field: NodeField) {
        self.field = Some(field);
    }

    /// Declare domain-specific parameters ρ_base and χ₀ with their provenance.
    ///
    /// These are not defaults, universal constants, or placeholders.
    /// Complexity of the declared domain determines the applicable values.
    ///
    /// `provenance` must honestly describe how the values were established:
    ///   - `Illustrative` — chosen to demonstrate operator mechanics
    ///   - `ObservationallyEstablished` — calibrated against observable R output
    pub fn declare_rho(
        &mut self,
        rho_base: f64,
        chi_0:    f64,
        provenance: ParameterProvenance,
    ) {
        self.rho = Some((rho_base, chi_0, provenance));
    }

    /// Declare cross-topology coupling values cc with corresponding edge pairs.
    ///
    /// The fixed cc = 0.5 default was retired at Kernel V7.1. Coupling
    /// between families must be M-declared per domain. If the declared
    /// topology has no cross-family edges, pass empty vecs.
    pub fn declare_cc(&mut self, cc: Vec<f64>, pairs: Vec<(usize, usize)>) {
        self.cc    = cc;
        self.pairs = pairs;
    }

    // ─── Admission check ─────────────────────────────────────────────────────

    /// Check whether A→B→R is admissible given current declarations.
    ///
    /// Returns `Admitted` if all prerequisites are met.
    /// Returns `Open(conditions)` listing every unmet prerequisite.
    ///
    /// Does not execute any operator. Does not fabricate values.
    pub fn admit(&self) -> AdmissionResult {
        let mut open: Vec<OpenCondition> = vec![];

        // Prerequisite 1: topology declared
        let rel = match &self.rel {
            Some(r) => r,
            None => {
                open.push(OpenCondition::TopologyNotDeclared);
                // Without topology we cannot check the interior locus.
                // Report all other missing declarations before returning.
                if self.field.is_none() {
                    open.push(OpenCondition::NodeFieldNotDeclared);
                }
                if self.rho.is_none() {
                    open.push(OpenCondition::RhoNotDeclared);
                }
                return AdmissionResult::Open(open);
            }
        };

        // Prerequisite 2: interior locus exists
        //
        // An interior locus is a node t such that:
        //   ∃ edge e with tgt(e) = t  AND  ∃ edge f with src(f) = t
        //   where e ≠ f (independently declared).
        //
        // This is the admissibility condition from Kernel V8 §11.
        let has_interior = (0..rel.n_nodes).any(|t| {
            let has_incoming = (0..rel.n_edges()).any(|e| rel.tgt(e) == t);
            let has_outgoing = (0..rel.n_edges()).any(|f| rel.src(f) == t);
            has_incoming && has_outgoing
        });
        if !has_interior {
            open.push(OpenCondition::NoInteriorLocus);
        }

        // Prerequisite 3: node field declared
        if self.field.is_none() {
            open.push(OpenCondition::NodeFieldNotDeclared);
        }

        // Prerequisite 4: ρ_base and χ₀ declared
        if self.rho.is_none() {
            open.push(OpenCondition::RhoNotDeclared);
        }

        if open.is_empty() {
            let provenance = self.rho.as_ref().unwrap().2;
            AdmissionResult::Admitted { parameter_provenance: provenance }
        } else {
            AdmissionResult::Open(open)
        }
    }

    // ─── Execution ───────────────────────────────────────────────────────────

    /// Execute frozen Kernel V8 operators A→B→R.
    ///
    /// Returns `Ok(ExecutionResult)` if admitted.
    /// Returns `Err(conditions)` listing unmet prerequisites otherwise.
    ///
    /// The kernel operators are called unchanged. No operator mathematics
    /// are implemented in this crate.
    pub fn execute(&self) -> Result<ExecutionResult, Vec<OpenCondition>> {
        // Re-check admission; surface conditions if any.
        match self.admit() {
            AdmissionResult::Open(conditions) => return Err(conditions),
            AdmissionResult::Admitted { .. } => {}
        }

        let rel   = self.rel.as_ref().unwrap();
        let field = self.field.as_ref().unwrap();
        let (rho_base, chi_0, provenance) = self.rho.as_ref().unwrap();

        // Validate M-declared parameters through kernel enforcement functions.
        require_rho_declaration(*rho_base, *chi_0);
        require_cc_declaration(&self.cc, &self.pairs);

        // Frozen Kernel V8 operators — called unchanged, in canonical A→B→R order.
        // A: edge differences from declared node field and topology
        let a = operator_a(field, rel, &self.pairs);
        // ρ[e] per edge — computed from A magnitudes and M-declared ρ_base, χ₀
        let rho = compute_rho(&a, rel, *rho_base, *chi_0);
        // B: accumulates successor contributions onto A
        let b = operator_b(&a, rel);
        // R: adds antisymmetric circulation term; R = E (final assembled field)
        let r = operator_r(&b, rel, &rho, &self.cc);

        // Detection indicators: 𝟙[e] = 1 if R antisymmetric term ≠ 0.
        // Read from R output. Not a parameter.
        let tol: f64 = 1e-12;
        let detection_indicators: Vec<f64> = (0..rel.n_edges())
            .map(|edge_idx| {
                // Antisymmetric term = R[e] − B[e] for each component; use c=0.
                let antis = r.spatial[0][edge_idx] - b.spatial[0][edge_idx];
                if antis.abs() > tol { 1.0 } else { 0.0 }
            })
            .collect();

        let all_zero = detection_indicators.iter().all(|&v| v == 0.0);

        if all_zero {
            Ok(ExecutionResult::ObservedZero {
                edge_field:           r,
                parameter_provenance: *provenance,
            })
        } else {
            Ok(ExecutionResult::RelationalOutput {
                edge_field:           r,
                parameter_provenance: *provenance,
                detection_indicators,
            })
        }
    }

    // ─── Trace report ────────────────────────────────────────────────────────

    /// Print the mathematical trace and parameter provenance for an execution result.
    ///
    /// This report distinguishes:
    ///   - CALCULATED quantities (from declared topology and node field alone)
    ///   - M-SUPPLIED quantities (ρ_base, χ₀) with their provenance label
    ///   - DETECTION INDICATORS read from R output
    ///   - ADMISSION RESULT
    pub fn trace_report(&self, result: &ExecutionResult) {
        let rel   = self.rel.as_ref().expect("trace_report requires declared topology");
        let field = self.field.as_ref().expect("trace_report requires declared field");
        let (rho_base, chi_0, _) = self.rho.as_ref().expect("trace_report requires declared rho");
        // Reproduce A to compute ρ (as in execute; ρ is derived from A magnitudes)
        let a_for_rho = operator_a(field, rel, &self.pairs);
        let rho = compute_rho(&a_for_rho, rel, *rho_base, *chi_0);

        println!();
        println!("═══════════════════════════════════════════════════════");
        println!("  IRRS SESSION — EXECUTION TRACE");
        println!("═══════════════════════════════════════════════════════");

        // Topology
        println!();
        println!("  DECLARED TOPOLOGY  ({} nodes, {} edges)",
            rel.n_nodes, rel.n_edges());
        for e in 0..rel.n_edges() {
            println!("    e{}: node {} → node {}  [{:?}]",
                e, rel.src(e), rel.tgt(e), rel.provenance(e));
        }

        // Node field
        println!();
        println!("  DECLARED NODE FIELD  (component 0)");
        for i in 0..field.n {
            println!("    x[{}] = {}", i, field.data[0][i]);
        }

        // ρ values (calculated from declared field and M-supplied parameters)
        println!();
        println!("  CALCULATED ρ (from declared field + M-supplied ρ_base, χ₀)");
        for i in 0..rel.n_nodes {
            println!("    ρ[{}] = {:.6}", i, rho[i]);
        }

        // M-supplied parameters with provenance
        let provenance_label = match result {
            ExecutionResult::RelationalOutput { parameter_provenance, .. } => *parameter_provenance,
            ExecutionResult::ObservedZero     { parameter_provenance, .. } => *parameter_provenance,
        };
        let provenance_str = match provenance_label {
            ParameterProvenance::Illustrative             => "ILLUSTRATIVE",
            ParameterProvenance::ObservationallyEstablished => "OBSERVATIONALLY ESTABLISHED",
        };
        println!();
        println!("  M-SUPPLIED PARAMETERS  [{}]", provenance_str);
        println!("    ρ_base = {}   (domain-specific ceiling; complexity-determined)", rho_base);
        println!("    χ₀     = {}   (domain-specific saturation scale; complexity-determined)", chi_0);
        if provenance_label == ParameterProvenance::Illustrative {
            println!();
            println!("  NOTE: Illustrative parameters enable operator demonstration.");
            println!("  To establish these observationally, M must independently observe");
            println!("  R[e] for two or more edges, then solve:");
            println!("    R[e] = B[e] + ρ[src(e)] · (Σ_succ B − Σ_pred B)");
            println!("    ρ[i] = ρ_base · χ[i] / (χ₀ + χ[i])");
            println!("  for ρ_base and χ₀ using declared topology and node field.");
        }

        // Execution results
        println!();
        match result {
            ExecutionResult::RelationalOutput { edge_field, detection_indicators, .. } => {
                println!("  ADMISSION RESULT: ADMITTED → RELATIONAL OUTPUT");
                println!();
                println!("  E[e] (assembled edge field, component 0):");
                for e in 0..rel.n_edges() {
                    println!("    E[e{}] = {:.6}  𝟙[e{}] = {:.0}",
                        e, edge_field.spatial[0][e], e, detection_indicators[e]);
                }
            }
            ExecutionResult::ObservedZero { edge_field, .. } => {
                println!("  ADMISSION RESULT: ADMITTED → OBSERVED ZERO");
                println!("  Antisymmetric circulation = 0 on all edges.");
                println!("  This is an observable outcome (flat field or zero χ).");
                println!("  Not a prerequisite failure. Not audit A-3.");
                println!();
                println!("  E[e] (assembled edge field, component 0):");
                for e in 0..rel.n_edges() {
                    println!("    E[e{}] = {:.6}  𝟙[e{}] = 0", e, edge_field.spatial[0][e], e);
                }
            }
        }
        println!();
        println!("═══════════════════════════════════════════════════════");
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
