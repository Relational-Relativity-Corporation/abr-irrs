/// IRRS Admission — D/M-Driven R Admission Check
///
/// Implements the full-operator admissibility condition for Kernel V8:
/// A→B→R is admissible when M establishes at least one interior locus
/// t = target(e) = source(f) for independently declared edges e and f.
///
/// Three execution outcomes:
///   COMPLETE / RelationalOutput  — R executed; antisymmetric term ≠ 0
///   COMPLETE / CalculatedZero    — R executed; kernel calculated zero (not an observation)
///   PARTIAL                      — A and B computed; R prerequisites unmet
///   BLOCKED                      — prerequisites missing before A can execute
///
/// Three provenance states (distinct; not interchangeable):
///   ILLUSTRATIVE                       — mathematical demonstration; no observational claim
///   EVIDENCE DECLARED — VERIFICATION OPEN — evidence exists; not yet independently evaluated
///   OBSERVATIONALLY VERIFIED           — inspected, derived, compared; all three documented
///
/// "CalculatedZero" names what the kernel produced. It does not assert
/// that a physical observable is zero. That claim requires an independent
/// observational comparison not supplied by this layer.

use irrs_kernel::operators::EdgeField;

// ─── Detection tolerance ─────────────────────────────────────────────────────

/// Numerical tolerance for the detection indicator 𝟙[e].
///
/// 𝟙[e] = 1 iff |R[c][e] − B[c][e]| > DETECTION_TOLERANCE for any component c.
///
/// This is a numerical precision parameter of the session layer, not a physical
/// detection threshold. Physical detection requires independent observational
/// support (an experiment, a measurement) not provided here. The session layer
/// computes whether the antisymmetric term is numerically nonzero given declared
/// values; whether that nonzero term corresponds to a detectable physical
/// quantity is a separate question outside this layer.
pub const DETECTION_TOLERANCE: f64 = 1e-12;

// ─── EvidenceRecord ──────────────────────────────────────────────────────────

/// Record supporting a declaration of non-illustrative parameter provenance.
///
/// Required when declaring `ParameterProvenance::EvidenceDeclared` or as the
/// underlying record of `ParameterProvenance::ObservationallyVerified`.
///
/// The session layer cannot verify the evidence; it requires the record to
/// exist and be non-empty so that origins cannot silently claim non-illustrative
/// status. Existence of this record produces EVIDENCE DECLARED — VERIFICATION
/// OPEN; it does not produce OBSERVATIONALLY VERIFIED.
///
/// To reach OBSERVATIONALLY VERIFIED, supply a `VerificationRecord` that
/// carries declared observables, parameter derivation, comparison results,
/// and verification source. See `VerificationRecord`.
#[derive(Debug, Clone)]
pub struct EvidenceRecord {
    /// Description of the calibration or derivation procedure.
    /// Must be non-empty. E.g.: "ρ_base and χ₀ fitted to R[e0], R[e1] from
    /// measurement run 2026-10-15 at node field x = [3.1, 5.2, 1.9]."
    pub description: String,
    /// Optional reference: DOI, data file path, experiment ID, or similar.
    pub reference: Option<String>,
}

impl EvidenceRecord {
    pub fn new(description: impl Into<String>) -> Self {
        let description = description.into();
        assert!(!description.trim().is_empty(), "EvidenceRecord: description must not be empty");
        EvidenceRecord { description, reference: None }
    }

    pub fn with_reference(mut self, reference: impl Into<String>) -> Self {
        self.reference = Some(reference.into());
        self
    }
}

// ─── DeclaredObservable ──────────────────────────────────────────────────────

/// A declared observable quantity used in parameter verification.
///
/// Carries the quantity's label, numerical value, unit, and the basis on
/// which it was declared. Required in a `VerificationRecord` to make the
/// observation-to-calculation correspondence inspectable.
#[derive(Debug, Clone)]
pub struct DeclaredObservable {
    /// Label identifying the quantity (e.g. "R[e0]", "E[e0,c=0]").
    pub label: String,
    /// Numerical value of the declared observable.
    pub value: f64,
    /// Physical unit, if applicable (e.g. "m/s", "dimensionless").
    /// None when dimensionless or unit not applicable.
    pub unit: Option<String>,
    /// How this quantity was measured or derived.
    pub measurement_provenance: String,
}

// ─── ObservationalComparison ─────────────────────────────────────────────────

/// A declared comparison target: an observed quantity and its acceptable tolerance.
///
/// Supplied in a `VerificationRecord` before execution. The session looks up the
/// actual kernel-calculated value (by `component` and `edge_index`) after running
/// R and builds an `EvaluatedComparison`. The caller does not supply `calculated`.
///
/// Tolerance is a domain-declared measurement precision, distinct from
/// `DETECTION_TOLERANCE` (a numerical precision parameter for the session layer).
/// These must not be conflated.
#[derive(Debug, Clone)]
pub struct ObservationalComparison {
    /// Label identifying the compared quantity (e.g. "R[e0]").
    pub label: String,
    /// The independently declared observed value.
    pub observed: f64,
    /// Domain-declared measurement tolerance. Not `DETECTION_TOLERANCE`.
    pub tolerance: f64,
    /// Which edge in the kernel output to compare against.
    pub edge_index: usize,
    /// Which component in the kernel output to compare against. Usually 0.
    pub component: usize,
}

// ─── EvaluatedComparison ─────────────────────────────────────────────────────

/// Result of a session-executed comparison: observed (declared) vs calculated
/// (actual kernel output).
///
/// Produced by the session after running R. `calculated` is the kernel's actual
/// result at `ObservationalComparison.edge_index` / `component`; it is never
/// supplied by the caller.
#[derive(Debug, Clone)]
pub struct EvaluatedComparison {
    /// Label from the declared `ObservationalComparison`.
    pub label: String,
    /// The independently declared observed value.
    pub observed: f64,
    /// The kernel-calculated value at the declared edge/component index.
    pub calculated: f64,
    /// Domain-declared tolerance, carried from the declared comparison.
    pub tolerance: f64,
}

impl EvaluatedComparison {
    /// Returns true iff |observed − calculated| ≤ tolerance.
    pub fn passes(&self) -> bool {
        (self.observed - self.calculated).abs() <= self.tolerance
    }

    /// Signed residual: observed − calculated.
    pub fn residual(&self) -> f64 {
        self.observed - self.calculated
    }
}

// ─── VerificationRecord ──────────────────────────────────────────────────────

/// Record supporting a claim of observational verification.
///
/// Required for `ParameterProvenance::ObservationallyVerified`.
/// Must carry enough information for independent inspection of the
/// observation-to-calculation correspondence.
///
/// An `ObservationallyVerified` declaration that carries failing comparisons
/// or open items does not block operator execution — the mathematical result
/// stands as calculated. The trace report surfaces failures explicitly so
/// the discrepancy is visible without rewriting the kernel output.
///
/// Use `open_items` to record what could not be verified rather than
/// silently omitting it or fabricating entries.
///
/// # Note on synthetic fixtures
///
/// When writing tests or demonstrations, clearly label the record as a
/// synthetic fixture ("Synthetic fixture — not physical experimental evidence")
/// in the evidence description. Do not represent synthetic data as physical
/// measurement evidence.
#[derive(Debug, Clone)]
pub struct VerificationRecord {
    /// The underlying evidence record.
    pub evidence: EvidenceRecord,
    /// Declared observables used in parameter derivation.
    /// Each entry identifies the quantity, value, unit, and measurement basis.
    pub declared_observables: Vec<DeclaredObservable>,
    /// Description of how ρ_base and χ₀ were derived from declared_observables.
    pub parameter_derivation: String,
    /// Source of independent evaluation.
    /// Identifies who or what inspected the evidence and derivation.
    pub verification_source: String,
    /// Comparisons between declared observables and kernel-calculated results.
    /// An empty list means no comparisons have been recorded: OPEN.
    pub comparisons: Vec<ObservationalComparison>,
    /// Items that remain OPEN in this verification record.
    /// Use to record what could not be verified rather than silently omitting it.
    pub open_items: Vec<String>,
}

// Note: is_fully_verified() and failed_comparisons() are intentionally absent.
// Verification status depends on actual kernel output, not on declared content.
// The session computes EvaluatedComparison values after executing R and
// determines pass/fail from those. See ParameterProvenance::label_with_evaluation().

// ─── ParameterProvenance ─────────────────────────────────────────────────────

/// Provenance of declared domain-specific parameters (ρ_base, χ₀, cc).
///
/// Three distinct states. They are not interchangeable and must not be
/// conflated:
///
/// `Illustrative` — values chosen to demonstrate operator mechanics.
///   No observational correspondence is claimed or implied.
///   Explicitly labeled in all trace output.
///
/// `EvidenceDeclared` — an `EvidenceRecord` has been supplied, but the
///   underlying evidence has not yet been independently evaluated.
///   Status: EVIDENCE DECLARED — VERIFICATION OPEN.
///   The record's existence does not constitute verification.
///
/// `ObservationallyVerified` — a `VerificationRecord` has been supplied
///   carrying declared observables, parameter derivation, comparison results,
///   and verification source. Status: OBSERVATIONALLY VERIFIED.
///   Failed comparisons or open items in the record are surfaced in trace
///   output; they do not block mathematical execution.
///
/// Separately from these three states, ρ_base, χ₀, and cc are domain-specific
/// parameters: complexity of the declared domain determines applicable values.
/// This is a structural principle; it is independent of provenance state.
#[derive(Debug, Clone)]
pub enum ParameterProvenance {
    /// Values declared to enable illustration of the operator mechanics.
    /// Not derived from R-output calibration over a declared observation.
    /// Labels as ILLUSTRATIVE in all trace output.
    Illustrative,

    /// Values accompanied by an evidence record, but not yet independently
    /// evaluated. The record's existence records the declaration; it does
    /// not constitute verification.
    ///
    /// Labels as EVIDENCE DECLARED — VERIFICATION OPEN.
    EvidenceDeclared(EvidenceRecord),

    /// Values independently verified against declared observable quantities.
    /// The VerificationRecord must carry declared observables, parameter
    /// derivation, comparison results, and verification source.
    ///
    /// Failed comparisons or open items in the record are surfaced in trace
    /// output but do not alter the kernel-calculated result.
    ///
    /// Labels as OBSERVATIONALLY VERIFIED (or OBSERVATIONALLY VERIFIED —
    /// CORRESPONDENCE FAILED if comparisons fail).
    ObservationallyVerified(VerificationRecord),
}

impl ParameterProvenance {
    /// Returns true iff this is the Illustrative variant.
    pub fn is_illustrative(&self) -> bool {
        matches!(self, Self::Illustrative)
    }

    /// Returns true iff this is the EvidenceDeclared variant.
    pub fn is_evidence_declared(&self) -> bool {
        matches!(self, Self::EvidenceDeclared(_))
    }

    /// Returns true iff this is the ObservationallyVerified variant.
    pub fn is_verified(&self) -> bool {
        matches!(self, Self::ObservationallyVerified(_))
    }

    /// Returns true iff this is any non-Illustrative variant.
    pub fn is_observational(&self) -> bool {
        !self.is_illustrative()
    }

    /// Primary provenance label for trace output.
    ///
    /// For `ObservationallyVerified`, always returns "OBSERVATIONALLY VERIFIED".
    /// Use `label_with_evaluation` to include CORRESPONDENCE FAILED status,
    /// which requires actual kernel-computed values.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Illustrative => "ILLUSTRATIVE",
            Self::EvidenceDeclared(_) => "EVIDENCE DECLARED — VERIFICATION OPEN",
            Self::ObservationallyVerified(_) => "OBSERVATIONALLY VERIFIED",
        }
    }

    /// Provenance label that accounts for actual kernel output.
    ///
    /// For `ObservationallyVerified`: returns CORRESPONDENCE FAILED if any
    /// evaluated comparison fails or if the VerificationRecord has open items.
    /// `evaluated` must be the comparisons produced by the session after
    /// executing R — not caller-supplied calculated values.
    pub fn label_with_evaluation(&self, evaluated: &[EvaluatedComparison]) -> &'static str {
        match self {
            Self::ObservationallyVerified(rec) => {
                let comparisons_pass = !evaluated.is_empty()
                    && evaluated.iter().all(|c| c.passes());
                let fully_verified = comparisons_pass && rec.open_items.is_empty();
                if fully_verified {
                    "OBSERVATIONALLY VERIFIED"
                } else {
                    "OBSERVATIONALLY VERIFIED — CORRESPONDENCE FAILED"
                }
            }
            _ => self.label(),
        }
    }
}

impl PartialEq for ParameterProvenance {
    /// Two provenances are equal iff they are the same variant.
    /// Evidence and verification content are not compared; they are for
    /// traceability, not identity.
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Illustrative, Self::Illustrative)
            | (Self::EvidenceDeclared(_), Self::EvidenceDeclared(_))
            | (Self::ObservationallyVerified(_), Self::ObservationallyVerified(_))
        )
    }
}
impl Eq for ParameterProvenance {}

// ─── KernelOpenCondition ─────────────────────────────────────────────────────

/// Unresolved mathematical conditions within Kernel V8 that affect R evaluation.
///
/// These are not session-layer deficiencies — they are mathematical questions
/// that remain open in the kernel specification itself (Kernel V8 §11.2, §18).
/// They are surfaced here so that callers are informed of what is not yet
/// derived from first principles, even when the session is fully admitted.
///
/// Domain-specificity of ρ_base and χ₀ is a separate, structural principle
/// (resolved: complexity of the declared domain determines the values).
/// These are unresolved derivation questions above and beyond domain-specificity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelOpenCondition {
    /// Kernel V8 OPEN (§11.2, §18): when multiple edges are incident to a
    /// source node across multiple components, the rule for selecting which
    /// magnitude drives ρ has not been derived from relational first principles.
    /// Current implementation uses the maximum |A[c][e]| over all incident
    /// edges and all components; this is a working assumption, not a derivation.
    RhoMaxSelectionRule,

    /// Kernel V8 OPEN (§11.2, §18): the saturating functional form
    ///   ρ[e] = ρ_base · χ[src(e)] / (χ₀ + χ[src(e)])
    /// satisfies required boundary conditions (ρ→0 as χ→0, ρ→ρ_base as χ→∞)
    /// but has not been formally derived from relational first principles.
    /// It is used as a working form pending derivation.
    RhoSaturatingForm,
}

// ─── OpenCondition ───────────────────────────────────────────────────────────

/// An unresolved admission prerequisite or a pre-execution validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenCondition {
    TopologyNotDeclared,
    NoInteriorLocus,
    NodeFieldNotDeclared,
    RhoNotDeclared,
    DimensionMismatch {
        field_nodes:    usize,
        topology_nodes: usize,
    },
    InvalidRhoParameter(String),
}

// ─── AdmissionResult ─────────────────────────────────────────────────────────

pub enum AdmissionResult {
    Admitted {
        parameter_provenance: ParameterProvenance,
        kernel_open:          Vec<KernelOpenCondition>,
    },
    Open(Vec<OpenCondition>),
}

// ─── ExecutionResult ─────────────────────────────────────────────────────────

/// Note: `EdgeField` does not implement `Debug`; this enum is not `Debug`.
pub enum ExecutionResult {
    RelationalOutput {
        edge_field:             EdgeField,
        parameter_provenance:   ParameterProvenance,
        detection_indicators:   Vec<f64>,
        kernel_open:            Vec<KernelOpenCondition>,
        /// Comparisons between declared observables and actual kernel output.
        /// Non-empty only when provenance is ObservationallyVerified.
        /// `calculated` in each entry is the kernel's actual R value — not
        /// a caller-supplied value.
        evaluated_comparisons:  Vec<EvaluatedComparison>,
    },

    /// R operator executed; antisymmetric term ≤ DETECTION_TOLERANCE on every
    /// edge for every declared component.
    ///
    /// "CalculatedZero" names what the kernel computed — not what a physical
    /// observable is. Physical confirmation requires an independent
    /// observational comparison outside this layer.
    CalculatedZero {
        edge_field:             EdgeField,
        parameter_provenance:   ParameterProvenance,
        kernel_open:            Vec<KernelOpenCondition>,
        /// Same semantics as in RelationalOutput.
        evaluated_comparisons:  Vec<EvaluatedComparison>,
    },
}

// ─── SessionOutput ───────────────────────────────────────────────────────────

pub enum SessionOutput {
    Complete(ExecutionResult),
    Partial {
        a_field:           EdgeField,
        b_field:           EdgeField,
        open_r_conditions: Vec<OpenCondition>,
        kernel_open:       Vec<KernelOpenCondition>,
    },
    Blocked(Vec<OpenCondition>),
}
