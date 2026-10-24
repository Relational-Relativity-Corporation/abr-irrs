/// IRRS Session — D/M Declaration and Kernel Execution
///
/// Accepts a bounded D and M, validates declarations, checks admission,
/// executes Kernel V8 operators, and reports the mathematical trace with
/// parameter provenance.
///
/// Session is stateful: declarations accumulate until execute() is called.
/// No operator is applied before prerequisites are satisfied.
///
/// Execution order:
///   1. Pre-execution validation (dimensions, parameter admissibility)
///   2. Topology + interior locus check → Blocked if insufficient
///   3. A and B computed (topology + field sufficient)
///   4. R prerequisites check → Partial if ρ unmet, Complete otherwise
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
use irrs_eval::record::{EvaluationRecord, EvaluationStatus};

use crate::admission::{
    AdmissionResult, EvaluatedComparison, ExecutionResult, KernelOpenCondition,
    OpenCondition, ParameterProvenance, SessionOutput,
    DETECTION_TOLERANCE,
};

// ─── Session ─────────────────────────────────────────────────────────────────

/// Interactive session for D/M-driven relational computation.
///
/// Workflow:
///   1. `declare_topology`          — supply DeclaredRelations
///   2. `declare_field`             — supply NodeField
///   3. `declare_rho`               — supply ρ_base, χ₀, and ParameterProvenance
///                                    (ObservationallyEstablished requires EvidenceRecord)
///   4. `declare_cc`                — supply cross-topology coupling (may be empty)
///   5. `declare_observable_basis`  — supply Origin-authorized EvaluationRecord
///                                    (omit to proceed with OPEN observable provenance)
///   6. `admit`                     — check prerequisites; inspect OpenConditions if any
///   7. `execute`                   — run frozen Kernel V8 operators; get SessionOutput
///   8. `trace_report`              — print mathematical trace and provenance
///
/// Observable relational provenance (step 5):
///   Supply the `EvaluationRecord` produced by `irrs_eval::Evaluator::evaluate`
///   after Origin has reviewed and committed `eval/authorization.json`.
///
///   - `EvaluationStatus::Authorized`                → observable provenance ADMITTED
///   - `EvaluationStatus::PassedAwaitingAuthorization` → OPEN; Origin review pending
///   - `EvaluationStatus::Failed`                    → execution BLOCKED; derivation
///     inadmissible. A failed evaluation means the provenance chain is broken; no
///     part of the operator chain is admitted.
///   - (not called)                                  → OPEN; basis undeclared
pub struct Session {
    rel:              Option<DeclaredRelations>,
    field:            Option<NodeField>,
    /// (ρ_base, χ₀, provenance)
    rho:              Option<(f64, f64, ParameterProvenance)>,
    cc:               Vec<f64>,
    pairs:            Vec<(usize, usize)>,
    /// Origin-authorized observable relational provenance record.
    /// None → basis not declared (OPEN).
    observable_basis: Option<EvaluationRecord>,
}

impl Session {
    /// Create an empty session. No declarations are present.
    pub fn new() -> Self {
        Session {
            rel:              None,
            field:            None,
            rho:              None,
            cc:               vec![],
            pairs:            vec![],
            observable_basis: None,
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
    /// All values must be finite (∈ D). field.n must equal the number of nodes
    /// in the declared topology; mismatches are caught at execute() time.
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
    ///   - `ObservationallyEstablished(evidence)` — calibrated against
    ///     independently observable R output; an `EvidenceRecord` is required
    ///
    /// Parameter admissibility (χ₀ > 0, ρ_base ≥ 0) is checked at execute().
    pub fn declare_rho(
        &mut self,
        rho_base:   f64,
        chi_0:      f64,
        provenance: ParameterProvenance,
    ) {
        self.rho = Some((rho_base, chi_0, provenance));
    }

    /// Declare cross-topology coupling values cc with corresponding component pairs.
    ///
    /// The fixed cc = 0.5 default was retired at Kernel V7.1. Coupling
    /// between families must be M-declared per domain. If the declared
    /// topology has no cross-family edges, pass empty vecs (the default).
    pub fn declare_cc(&mut self, cc: Vec<f64>, pairs: Vec<(usize, usize)>) {
        self.cc    = cc;
        self.pairs = pairs;
    }

    /// Declare the observable relational provenance basis for this session.
    ///
    /// Supply the `EvaluationRecord` produced by `irrs_eval::Evaluator::evaluate`
    /// after Origin has reviewed and committed `eval/authorization.json`.
    ///
    /// This is step 5 of the session workflow. It is optional: omitting it
    /// allows execution to proceed with `OpenCondition::ObservableBasisNotDeclared`.
    /// A record with `EvaluationStatus::Failed` blocks execution entirely —
    /// see `SessionOutput::Blocked(OpenCondition::ObservableProvenanceRejected)`.
    ///
    /// The `EvaluationRecord` is threaded into the `ExecutionResult` so every
    /// admitted result carries a verifiable link to its declared observable basis.
    pub fn declare_observable_basis(&mut self, record: EvaluationRecord) {
        self.observable_basis = Some(record);
    }

    // ─── Kernel OPEN conditions ───────────────────────────────────────────────

    /// Return the Kernel V8 mathematical conditions that remain unresolved
    /// regardless of session declarations. These are always present when R
    /// is evaluated; they do not block execution but must be surfaced.
    pub fn kernel_open_conditions() -> Vec<KernelOpenCondition> {
        vec![
            KernelOpenCondition::RhoMaxSelectionRule,
            KernelOpenCondition::RhoSaturatingForm,
        ]
    }

    // ─── Pre-execution validation ─────────────────────────────────────────────

    /// Validate declared values before any operator is called.
    ///
    /// Checks:
    ///   - field.n == topology.n_nodes (dimension compatibility)
    ///   - χ₀ > 0 (strictly positive; appears as denominator in ρ formula)
    ///   - ρ_base ≥ 0 (dimensionless ceiling; must be non-negative)
    ///
    /// Returns a list of `OpenCondition::DimensionMismatch` or
    /// `OpenCondition::InvalidRhoParameter` items. Empty means valid.
    fn validate(&self) -> Vec<OpenCondition> {
        let mut errors: Vec<OpenCondition> = vec![];

        // Dimension check: field nodes must equal topology nodes.
        if let (Some(rel), Some(field)) = (&self.rel, &self.field) {
            if field.n != rel.n_nodes {
                errors.push(OpenCondition::DimensionMismatch {
                    field_nodes:    field.n,
                    topology_nodes: rel.n_nodes,
                });
            }
        }

        // ρ parameter admissibility.
        if let Some((rho_base, chi_0, _)) = &self.rho {
            if *chi_0 <= 0.0 {
                errors.push(OpenCondition::InvalidRhoParameter(format!(
                    "χ₀ must be strictly positive (got {}); it appears as a denominator \
                     in ρ[e] = ρ_base·χ[src(e)] / (χ₀ + χ[src(e)])",
                    chi_0
                )));
            }
            if *rho_base < 0.0 {
                errors.push(OpenCondition::InvalidRhoParameter(format!(
                    "ρ_base must be non-negative (got {}); it is a dimensionless ceiling",
                    rho_base
                )));
            }
        }

        errors
    }

    // ─── Interior locus check ─────────────────────────────────────────────────

    fn has_interior_locus(rel: &DeclaredRelations) -> bool {
        (0..rel.n_nodes).any(|t| {
            let has_incoming = (0..rel.n_edges()).any(|e| rel.tgt(e) == t);
            let has_outgoing = (0..rel.n_edges()).any(|f| rel.src(f) == t);
            has_incoming && has_outgoing
        })
    }

    // ─── Admission check ─────────────────────────────────────────────────────

    /// Check whether A→B→R is admissible given current declarations.
    ///
    /// Returns `Admitted` if all prerequisites are met (including dimension
    /// validation and ρ parameter admissibility).
    /// Returns `Open(conditions)` listing every unmet prerequisite.
    ///
    /// `Admitted` also carries `kernel_open` — the Kernel V8 mathematical
    /// questions that remain unresolved regardless of session declarations.
    ///
    /// Does not execute any operator. Does not fabricate values.
    pub fn admit(&self) -> AdmissionResult {
        // Validation errors block admission.
        let validation_errors = self.validate();
        if !validation_errors.is_empty() {
            return AdmissionResult::Open(validation_errors);
        }

        let mut open: Vec<OpenCondition> = vec![];

        // Prerequisite 1: topology declared.
        let rel = match &self.rel {
            Some(r) => r,
            None => {
                open.push(OpenCondition::TopologyNotDeclared);
                if self.field.is_none() { open.push(OpenCondition::NodeFieldNotDeclared); }
                if self.rho.is_none()   { open.push(OpenCondition::RhoNotDeclared); }
                return AdmissionResult::Open(open);
            }
        };

        // Prerequisite 2: interior locus.
        if !Self::has_interior_locus(rel) {
            open.push(OpenCondition::NoInteriorLocus);
        }

        // Prerequisite 3: node field.
        if self.field.is_none() {
            open.push(OpenCondition::NodeFieldNotDeclared);
        }

        // Prerequisite 4: ρ_base and χ₀.
        if self.rho.is_none() {
            open.push(OpenCondition::RhoNotDeclared);
        }

        // Check observable relational provenance.
        // A Failed evaluation blocks execution; surface the reason here.
        // An absent basis is OPEN (not blocking) — noted as ObservableBasisNotDeclared.
        match &self.observable_basis {
            None => {
                open.push(OpenCondition::ObservableBasisNotDeclared);
            }
            Some(record) if record.status == EvaluationStatus::Failed => {
                open.push(OpenCondition::ObservableProvenanceRejected(
                    record.failure_reason.clone().unwrap_or_else(|| {
                        "EvaluationRecord status: Failed (no failure_reason provided)".to_string()
                    })
                ));
            }
            _ => {}
        }

        if open.iter().any(|c| matches!(c, OpenCondition::ObservableProvenanceRejected(_))) {
            // A failed evaluation is a hard block — like SyntheticViolation in irrs-eval.
            return AdmissionResult::Open(open);
        }

        // ObservableBasisNotDeclared is present in open but does NOT block Admitted.
        // The session can proceed experimentally with OPEN provenance.
        let structural_open: Vec<OpenCondition> = open.iter()
            .filter(|c| !matches!(c, OpenCondition::ObservableBasisNotDeclared))
            .cloned()
            .collect();

        if structural_open.is_empty() {
            AdmissionResult::Admitted {
                parameter_provenance:  self.rho.as_ref().unwrap().2.clone(),
                kernel_open:           Self::kernel_open_conditions(),
                observable_provenance: self.observable_basis.clone(),
            }
        } else {
            AdmissionResult::Open(open)
        }
    }

    // ─── Execution ───────────────────────────────────────────────────────────

    /// Execute frozen Kernel V8 operators A→B→R.
    ///
    /// Returns `SessionOutput`:
    ///   `Complete`  — all prerequisites met; full A→B→R executed
    ///   `Partial`   — topology and field declared, ρ unmet; A and B computed
    ///   `Blocked`   — missing declarations, validation failure, or rejected
    ///                 observable provenance; nothing computed
    ///
    /// When ρ is not declared but topology and field are present, A and B are
    /// computed and returned in `Partial`. R is not fabricated.
    ///
    /// When `declare_observable_basis` was called with a record whose status is
    /// `EvaluationStatus::Failed`, execution is blocked entirely — A, B, and R are
    /// all withheld. A failed provenance evaluation means the derivation's observable
    /// basis is inadmissible; no part of the operator chain is admitted.
    ///
    /// Detection indicators 𝟙[e] are evaluated across ALL declared components
    /// and ALL edges. DETECTION_TOLERANCE is explicit; see its documentation
    /// for the distinction between numerical zero and physical detection.
    ///
    /// The kernel operators are called unchanged. No operator mathematics
    /// are implemented in this crate.
    pub fn execute(&self) -> SessionOutput {
        // Step 0: check observable provenance rejection before any operator executes.
        //
        // A Failed EvaluationRecord means the declared basis is inadmissible. The
        // operator chain does not run, regardless of whether structural prerequisites
        // are met. This is the enforcement boundary for the Relational Primitive
        // Provenance Invariant: a correct numerical result is not sufficient evidence
        // that the derivation is traceable to declared observable relations.
        if let Some(record) = &self.observable_basis {
            if record.status == EvaluationStatus::Failed {
                return SessionOutput::Blocked(vec![
                    OpenCondition::ObservableProvenanceRejected(
                        record.failure_reason.clone().unwrap_or_else(|| {
                            "EvaluationRecord status: Failed (no failure_reason provided)"
                                .to_string()
                        })
                    )
                ]);
            }
        }

        // Step 1: pre-execution validation (dimensions, ρ admissibility).
        let validation_errors = self.validate();
        if !validation_errors.is_empty() {
            return SessionOutput::Blocked(validation_errors);
        }

        // Step 2: require topology and field to compute A and B.
        let (rel, field) = match (&self.rel, &self.field) {
            (Some(r), Some(f)) => (r, f),
            _ => {
                let mut conditions = vec![];
                if self.rel.is_none()   { conditions.push(OpenCondition::TopologyNotDeclared); }
                if self.field.is_none() { conditions.push(OpenCondition::NodeFieldNotDeclared); }
                if self.rho.is_none()   { conditions.push(OpenCondition::RhoNotDeclared); }
                return SessionOutput::Blocked(conditions);
            }
        };

        // Step 3: interior locus required before any operator acts.
        if !Self::has_interior_locus(rel) {
            return SessionOutput::Blocked(vec![OpenCondition::NoInteriorLocus]);
        }

        // A and B are computable. Compute them using frozen kernel operators.
        // A: edge differences — x[src(e)] − x[tgt(e)] for each edge e.
        let a = operator_a(field, rel, &self.pairs);
        // B: accumulates immediate successor A values — B[e] = A[e] + Σ_{succ(e)} A.
        let b = operator_b(&a, rel);

        // Step 4: require ρ to compute R.
        let (rho_base, chi_0, provenance) = match &self.rho {
            Some(triple) => (&triple.0, &triple.1, &triple.2),
            None => {
                return SessionOutput::Partial {
                    a_field:           a,
                    b_field:           b,
                    open_r_conditions: vec![OpenCondition::RhoNotDeclared],
                    kernel_open:       Self::kernel_open_conditions(),
                };
            }
        };

        // Validate M-declared parameters through kernel enforcement functions.
        // (These assert on violation; validation above ensures we reach here safely.)
        require_rho_declaration(*rho_base, *chi_0);
        require_cc_declaration(&self.cc, &self.pairs);

        // ρ[e] per edge — computed from A magnitudes and M-declared ρ_base, χ₀.
        let rho = compute_rho(&a, rel, *rho_base, *chi_0);

        // R: adds antisymmetric circulation term — R[c][e] = B[c][e] + ρ[e]·(Σ_succ B − Σ_pred B).
        let r = operator_r(&b, rel, &rho, &self.cc);

        // Detection indicators: 𝟙[e] = 1 iff antisymmetric term is numerically
        // nonzero for ANY declared component c at edge e.
        // Checked across ALL components, not only c=0.
        let detection_indicators: Vec<f64> = (0..rel.n_edges())
            .map(|e| {
                let any_nonzero = (0..r.k).any(|c| {
                    (r.spatial[c][e] - b.spatial[c][e]).abs() > DETECTION_TOLERANCE
                });
                if any_nonzero { 1.0 } else { 0.0 }
            })
            .collect();

        let all_zero = detection_indicators.iter().all(|&v| v == 0.0);
        let prov        = provenance.clone();
        let kernel_open = Self::kernel_open_conditions();

        // Evaluate declared comparisons against actual kernel output.
        // Only populated when provenance is ObservationallyVerified.
        // `calculated` comes from the kernel result, never from the caller.
        let evaluated_comparisons: Vec<EvaluatedComparison> = match provenance {
            ParameterProvenance::ObservationallyVerified(rec) => {
                rec.comparisons.iter().map(|c| {
                    let calculated =
                        if c.component < r.k && c.edge_index < r.spatial[c.component].len() {
                            r.spatial[c.component][c.edge_index]
                        } else {
                            f64::NAN  // out-of-bounds declaration → fails comparison
                        };
                    EvaluatedComparison {
                        label:      c.label.clone(),
                        observed:   c.observed,
                        calculated,
                        tolerance:  c.tolerance,
                    }
                }).collect()
            }
            _ => vec![],
        };

        // Clone the observable_provenance once for use in both branches.
        let observable_provenance = self.observable_basis.clone();

        if all_zero {
            SessionOutput::Complete(ExecutionResult::CalculatedZero {
                edge_field:            r,
                parameter_provenance:  prov,
                kernel_open,
                evaluated_comparisons,
                observable_provenance,
            })
        } else {
            SessionOutput::Complete(ExecutionResult::RelationalOutput {
                edge_field:            r,
                parameter_provenance:  prov,
                detection_indicators,
                kernel_open,
                evaluated_comparisons,
                observable_provenance,
            })
        }
    }

    // ─── Trace report ────────────────────────────────────────────────────────

    /// Print the mathematical trace and provenance for any SessionOutput.
    ///
    /// This report distinguishes:
    ///   CALCULATED  — quantities from declared topology and node field alone
    ///   M-SUPPLIED  — ρ_base, χ₀ with their provenance label
    ///   KERNEL OPEN — unresolved Kernel V8 mathematical conditions
    ///   RESULT      — Complete (RelationalOutput / CalculatedZero) or Partial
    pub fn trace_report(&self, output: &SessionOutput) {
        // Extract session-evaluated comparisons (actual kernel output vs declared).
        // These are used both in the M-SUPPLIED PARAMETERS section (for the
        // comparison table) and for the result label.
        let evaluated: &[EvaluatedComparison] = match output {
            SessionOutput::Complete(ExecutionResult::RelationalOutput {
                evaluated_comparisons, ..
            })
            | SessionOutput::Complete(ExecutionResult::CalculatedZero {
                evaluated_comparisons, ..
            }) => evaluated_comparisons.as_slice(),
            _ => &[],
        };

        println!();
        println!("═══════════════════════════════════════════════════════");
        println!("  IRRS SESSION — EXECUTION TRACE");
        println!("═══════════════════════════════════════════════════════");

        // Topology
        if let Some(rel) = &self.rel {
            println!();
            println!("  DECLARED TOPOLOGY  ({} nodes, {} edges)",
                rel.n_nodes, rel.n_edges());
            for e in 0..rel.n_edges() {
                println!("    e{}: node {} → node {}  [{:?}]",
                    e, rel.src(e), rel.tgt(e), rel.provenance(e));
            }
        } else {
            println!();
            println!("  DECLARED TOPOLOGY  [not declared]");
        }

        // Node field
        if let Some(field) = &self.field {
            println!();
            let comp_label = if field.k == 1 { "1 component".to_string() }
                             else { format!("{} components", field.k) };
            println!("  DECLARED NODE FIELD  ({})", comp_label);
            for c in 0..field.k {
                for i in 0..field.n {
                    println!("    x[c={},i={}] = {}", c, i, field.data[c][i]);
                }
            }
        } else {
            println!();
            println!("  DECLARED NODE FIELD  [not declared]");
        }

        // M-supplied parameters
        println!();
        if let Some((rho_base, chi_0, provenance)) = &self.rho {
            println!("  M-SUPPLIED PARAMETERS  [{}]", provenance.label());
            println!("    ρ_base = {}   (domain-specific ceiling; complexity-determined)", rho_base);
            println!("    χ₀     = {}   (domain-specific saturation scale; complexity-determined)", chi_0);
            match provenance {
                ParameterProvenance::Illustrative => {
                    println!();
                    println!("  NOTE: Illustrative parameters enable operator demonstration.");
                    println!("  No observational correspondence is claimed.");
                    println!("  To reach EVIDENCE DECLARED status, supply an EvidenceRecord.");
                    println!("  To reach OBSERVATIONALLY VERIFIED status, supply a VerificationRecord");
                    println!("  with declared observables, parameter derivation, comparison results,");
                    println!("  and verification source.");
                }
                ParameterProvenance::EvidenceDeclared(ev) => {
                    println!();
                    println!("  EVIDENCE RECORD:");
                    println!("    {}", ev.description);
                    if let Some(r) = &ev.reference {
                        println!("    Reference: {}", r);
                    }
                    println!();
                    println!("  VERIFICATION STATUS: OPEN");
                    println!("  The evidence record declares the basis for these parameters.");
                    println!("  It has not been independently evaluated. To reach OBSERVATIONALLY");
                    println!("  VERIFIED, supply a VerificationRecord with declared observables,");
                    println!("  parameter derivation, comparison results, and verification source.");
                }
                ParameterProvenance::ObservationallyVerified(rec) => {
                    println!();
                    println!("  EVIDENCE RECORD:");
                    println!("    {}", rec.evidence.description);
                    if let Some(r) = &rec.evidence.reference {
                        println!("    Reference: {}", r);
                    }
                    println!();
                    println!("  PARAMETER DERIVATION:");
                    println!("    {}", rec.parameter_derivation);
                    println!();
                    println!("  VERIFICATION SOURCE: {}", rec.verification_source);
                    if !rec.declared_observables.is_empty() {
                        println!();
                        println!("  DECLARED OBSERVABLES:");
                        for obs in &rec.declared_observables {
                            let unit = obs.unit.as_deref().unwrap_or("dimensionless");
                            println!("    {} = {}  [{}]  ({})",
                                obs.label, obs.value, unit, obs.measurement_provenance);
                        }
                    }
                    // Show session-evaluated comparisons (actual kernel output).
                    if !evaluated.is_empty() {
                        println!();
                        println!("  OBSERVATIONAL COMPARISONS (kernel output vs declared):");
                        let mut n_failed = 0usize;
                        for cmp in evaluated {
                            let status = if cmp.passes() { "PASS" } else { n_failed += 1; "FAIL" };
                            println!("    {}  observed={:.6}  calculated={:.6}  \
                                     residual={:+.6}  tol={:.2e}  [{}]",
                                cmp.label, cmp.observed, cmp.calculated,
                                cmp.residual(), cmp.tolerance, status);
                        }
                        if n_failed > 0 {
                            println!();
                            println!("  ⚠ FAILED COMPARISONS: {}", n_failed);
                            println!("  The kernel-calculated result stands as computed.");
                            println!("  Declared observables and actual kernel output disagree");
                            println!("  beyond declared tolerance. The mathematical result has");
                            println!("  not been rewritten to match the declared observable.");
                        }
                    } else if rec.comparisons.is_empty() {
                        println!();
                        println!("  COMPARISONS: none declared — correspondence OPEN");
                    }
                    if !rec.open_items.is_empty() {
                        println!();
                        println!("  OPEN VERIFICATION ITEMS:");
                        for item in &rec.open_items {
                            println!("    ✗ {}", item);
                        }
                    }
                }
            }
        } else {
            println!("  M-SUPPLIED PARAMETERS  [not declared]");
            println!("    ρ_base and χ₀ are domain-specific and must be declared.");
        }

        // Observable relational provenance
        println!();
        match &self.observable_basis {
            None => {
                println!("  OBSERVABLE RELATIONAL PROVENANCE  [OPEN — basis not declared]");
                println!("    No EvaluationRecord supplied. The derivation is not traceable to a");
                println!("    declared observable basis. To establish provenance:");
                println!("      1. Run irrs_eval::Evaluator::evaluate on a Declaration.");
                println!("      2. Have Origin review and commit eval/authorization.json.");
                println!("      3. Call Session::declare_observable_basis with the EvaluationRecord.");
            }
            Some(record) => {
                use irrs_eval::record::EvaluationStatus;
                let status_label = match &record.status {
                    EvaluationStatus::Authorized => "ADMITTED — Origin authorized",
                    EvaluationStatus::PassedAwaitingAuthorization =>
                        "OPEN — evaluated; awaiting Origin authorization",
                    EvaluationStatus::Failed => "REJECTED — evaluation failed (should have blocked execution)",
                };
                println!("  OBSERVABLE RELATIONAL PROVENANCE  [{}]", status_label);
                println!("    Record ID:          {}", record.record_id);
                println!("    Domain:             {}", record.domain);
                println!("    Topology ID:        {}", record.topology_id);
                println!("    Declaration digest: {}", record.declaration_digest);
                if let Some(auth) = &record.authorization {
                    println!("    Authorized by:      {}", auth.authorized_by);
                    println!("    Authorized at:      {}", auth.authorized_at);
                    println!("    Statement:          {}", auth.statement);
                }
                if let Some(reason) = &record.failure_reason {
                    println!("    Failure reason:     {}", reason);
                }
                let obs_count = record.input_inventory.iter()
                    .filter(|i| matches!(i.classification,
                        irrs_eval::record::InventoryKind::PrimaryObservable))
                    .count();
                println!("    Observable inputs:  {}", obs_count);
                println!("    Total inputs:       {}", record.input_inventory.len());
                if !record.evaluation_notes.is_empty() {
                    println!("    Evaluation notes:");
                    for note in &record.evaluation_notes {
                        println!("      — {}", note);
                    }
                }
            }
        }

        // Kernel OPEN conditions — always surfaced
        println!();
        println!("  KERNEL V8 OPEN CONDITIONS (unresolved mathematical questions):");
        println!("    ① RhoMaxSelectionRule  — χ magnitude selection across components");
        println!("      and incident edges unconfirmed (§11.2, §18).");
        println!("    ② RhoSaturatingForm    — saturating form ρ=ρ_base·χ/(χ₀+χ) not");
        println!("      derived from relational first principles (§11.2, §18).");

        // Execution result
        println!();
        match output {
            SessionOutput::Blocked(conditions) => {
                println!("  RESULT: BLOCKED");
                for c in conditions {
                    println!("    ✗ {:?}", c);
                }
            }

            SessionOutput::Partial { a_field, b_field, open_r_conditions, .. } => {
                let rel = self.rel.as_ref().unwrap();
                println!("  RESULT: PARTIAL EXECUTION — A and B computed; R blocked");
                println!();
                println!("  A[e] (edge differences, all components):");
                for c in 0..a_field.k {
                    for e in 0..rel.n_edges() {
                        println!("    A[c={},e{}] = {:.6}", c, e, a_field.spatial[c][e]);
                    }
                }
                println!();
                println!("  B[e] (successor accumulation, all components):");
                for c in 0..b_field.k {
                    for e in 0..rel.n_edges() {
                        println!("    B[c={},e{}] = {:.6}", c, e, b_field.spatial[c][e]);
                    }
                }
                println!();
                println!("  R OPEN conditions:");
                for c in open_r_conditions {
                    println!("    ✗ {:?}", c);
                }
            }

            SessionOutput::Complete(result) => {
                let rel = self.rel.as_ref().unwrap();
                match result {
                    ExecutionResult::RelationalOutput {
                        edge_field, detection_indicators, parameter_provenance,
                        evaluated_comparisons, ..
                    } => {
                        println!("  RESULT: ADMITTED → RELATIONAL OUTPUT [{}]",
                            parameter_provenance.label_with_evaluation(evaluated_comparisons));
                        println!();
                        println!("  R[e] = E[e] (assembled edge field, all components):");
                        for c in 0..edge_field.k {
                            for e in 0..rel.n_edges() {
                                println!("    E[c={},e{}] = {:.6}  𝟙[e{}] = {:.0}",
                                    c, e, edge_field.spatial[c][e], e, detection_indicators[e]);
                            }
                        }
                        println!();
                        println!("  Detection tolerance: {}", DETECTION_TOLERANCE);
                        println!("  (Numerical precision threshold — not a physical detection limit.)");
                    }
                    ExecutionResult::CalculatedZero {
                        edge_field, parameter_provenance, evaluated_comparisons, ..
                    } => {
                        println!("  RESULT: ADMITTED → CALCULATED ZERO [{}]",
                            parameter_provenance.label_with_evaluation(evaluated_comparisons));
                        println!("  The kernel computed zero antisymmetric term on all edges.");
                        println!("  This is a kernel output, not a physical observation.");
                        println!("  Independent observational comparison is required to assert");
                        println!("  that the physical observable is zero.");
                        println!();
                        println!("  R[e] = B[e] (no antisymmetric contribution, all components):");
                        for c in 0..edge_field.k {
                            for e in 0..rel.n_edges() {
                                println!("    E[c={},e{}] = {:.6}  𝟙[e{}] = 0", c, e,
                                    edge_field.spatial[c][e], e);
                            }
                        }
                        println!();
                        println!("  Detection tolerance: {}", DETECTION_TOLERANCE);
                    }
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
