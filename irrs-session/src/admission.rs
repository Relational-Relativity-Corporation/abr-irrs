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
/// Key distinction: "CalculatedZero" names what the kernel produced. It does
/// not assert that a physical observable is zero. That claim requires an
/// independent observational comparison not supplied by this layer.

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

/// Record supporting a claim of observationally established parameter provenance.
///
/// Required when declaring `ParameterProvenance::ObservationallyEstablished`.
/// The session layer cannot verify the evidence; it requires the record to exist
/// and be non-empty so that Origins cannot silently claim observational status.
///
/// Minimum: a human-readable description of how the parameter was calibrated.
/// Preferred: a reference to the measurement record, data file, or derivation.
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

// ─── ParameterProvenance ─────────────────────────────────────────────────────

/// Provenance of declared domain-specific parameters (ρ_base, χ₀, cc).
///
/// These parameters are domain-specific because complexity of the declared
/// domain determines the characteristic scales. They cannot be universally
/// assigned.
///
/// `ObservationallyEstablished` requires an `EvidenceRecord`. The provenance
/// enum alone is not verification; the record makes the basis of the claim
/// explicit and traceable.
///
/// See `irrs-kernel` documentation: "Domain-specificity principle."
#[derive(Debug, Clone)]
pub enum ParameterProvenance {
    /// Values declared from calibration against independently observable R
    /// output. The kernel was run over a declared observation; parameter values
    /// were adjusted until R output matched the observable. An `EvidenceRecord`
    /// describing the calibration is required.
    ObservationallyEstablished(EvidenceRecord),

    /// Values declared to enable illustration of the operator mechanics.
    /// Not derived from R-output calibration over a declared observation.
    /// The DECLARATION REPORT labels these clearly.
    ///
    /// An illustrative value does not falsify the operator; it faithfully
    /// reports that the parameter is not yet observationally constrained.
    Illustrative,
}

impl ParameterProvenance {
    /// Returns true iff this is the Illustrative variant.
    pub fn is_illustrative(&self) -> bool {
        matches!(self, ParameterProvenance::Illustrative)
    }

    /// Returns true iff this is the ObservationallyEstablished variant.
    pub fn is_observational(&self) -> bool {
        matches!(self, ParameterProvenance::ObservationallyEstablished(_))
    }

    /// Display label for trace reports.
    pub fn label(&self) -> &'static str {
        match self {
            ParameterProvenance::Illustrative => "ILLUSTRATIVE",
            ParameterProvenance::ObservationallyEstablished(_) => "OBSERVATIONALLY ESTABLISHED",
        }
    }
}

impl PartialEq for ParameterProvenance {
    /// Two provenances are equal iff they are the same variant.
    /// Evidence content is not compared; it is for traceability, not identity.
    fn eq(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Illustrative, Self::Illustrative)
            | (Self::ObservationallyEstablished(_), Self::ObservationallyEstablished(_))
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
///
/// When `AdmissionResult::Open` or `SessionOutput::Blocked` is returned,
/// each condition names one thing that prevents execution. These are not
/// errors — they are the honest report of what declaration is missing or
/// what parameter value is inadmissible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenCondition {
    /// The relational structure (topology) has not been declared.
    /// Without topology, no interior locus can be established.
    TopologyNotDeclared,

    /// Topology was declared but contains no interior locus.
    /// An interior locus requires a node t such that at least one edge
    /// terminates at t and at least one edge originates from t
    /// (independently declared: not the same edge in both roles).
    NoInteriorLocus,

    /// The node field (numerical projection through M) has not been declared.
    NodeFieldNotDeclared,

    /// Domain-specific parameters ρ_base and χ₀ have not been declared.
    /// These cannot be defaulted. The domain must establish them through M.
    RhoNotDeclared,

    /// The node field's node count does not match the declared topology.
    /// A cannot be evaluated: each edge references a source and target node
    /// by index, which must exist in the field.
    DimensionMismatch {
        field_nodes:    usize,
        topology_nodes: usize,
    },

    /// A declared ρ parameter value is inadmissible.
    /// χ₀ must be strictly positive (it appears as a denominator).
    /// ρ_base must be non-negative (it is a dimensionless ceiling).
    InvalidRhoParameter(String),
}

// ─── AdmissionResult ─────────────────────────────────────────────────────────

/// The outcome of the full-operator admissibility check (admit()).
///
/// `Admitted` confirms that all session prerequisites are met and A→B→R may
/// execute. It also carries `kernel_open` — the Kernel V8 mathematical
/// questions that remain unresolved regardless of session declarations.
pub enum AdmissionResult {
    /// All session prerequisites met. A→B→R may execute.
    Admitted {
        /// Provenance of ρ_base and χ₀ as declared by this session's M.
        parameter_provenance: ParameterProvenance,
        /// Unresolved mathematical conditions within Kernel V8.
        /// These are present whenever R is evaluated; they do not block
        /// execution but must be visible to the caller.
        kernel_open: Vec<KernelOpenCondition>,
    },

    /// Prerequisites not met. Lists every unresolved condition.
    /// Callers should address each listed condition and re-admit.
    Open(Vec<OpenCondition>),
}

// ─── ExecutionResult ─────────────────────────────────────────────────────────

/// The outcome of a complete A→B→R execution (within SessionOutput::Complete).
///
/// Note: `EdgeField` does not implement `Debug`; this enum is not `Debug`.
pub enum ExecutionResult {
    /// R operator executed; antisymmetric term ≠ 0 on at least one edge
    /// for at least one declared component.
    /// Detection indicators 𝟙[e] are read from R output, not parameter-set.
    RelationalOutput {
        /// The R-operator output (= E field) for all spatial edges.
        edge_field:           EdgeField,
        /// Provenance of ρ_base and χ₀ used in this execution.
        parameter_provenance: ParameterProvenance,
        /// 𝟙[e] — 1.0 if |R[c][e] − B[c][e]| > DETECTION_TOLERANCE for any c.
        /// One value per spatial edge. Read from R output; not a parameter.
        detection_indicators: Vec<f64>,
        /// Unresolved mathematical conditions from Kernel V8 that affected R.
        kernel_open:          Vec<KernelOpenCondition>,
    },

    /// R operator executed; antisymmetric term ≤ DETECTION_TOLERANCE on every
    /// edge for every declared component.
    ///
    /// "CalculatedZero" names what the kernel computed — not what a physical
    /// observable is. Physical confirmation that the relevant quantity is zero
    /// requires an independent observational comparison outside this layer.
    ///
    /// CalculatedZero ≠ audit A-3 (Kernel V8 OPEN): audit A-3 concerns
    /// detection events with undeclared topology. This variant is the
    /// correctly executed R operator returning numerical zero.
    CalculatedZero {
        /// The R-operator output (= B in this case; no antisymmetric term).
        edge_field:           EdgeField,
        /// Provenance of ρ_base and χ₀ used in this execution.
        parameter_provenance: ParameterProvenance,
        /// Unresolved mathematical conditions from Kernel V8.
        kernel_open:          Vec<KernelOpenCondition>,
    },
}

// ─── SessionOutput ───────────────────────────────────────────────────────────

/// The full output of Session::execute().
///
/// Three states:
///   Complete — all prerequisites met; A→B→R executed; result is RelationalOutput
///              or CalculatedZero.
///   Partial  — topology and field declared; R prerequisites unmet; A and B
///              computed and preserved; open R conditions listed.
///   Blocked  — prerequisites missing before A can execute, or a validation
///              failure was detected; nothing computed.
///
/// Partial execution exposes A and B even when R cannot run. This preserves
/// the calculable structural information (edge differences, successor
/// accumulations) for inspection without fabricating R values.
pub enum SessionOutput {
    /// Full A→B→R execution completed.
    Complete(ExecutionResult),

    /// A and B computed; R could not execute.
    ///
    /// Returned when topology and field are declared and an interior locus
    /// exists, but R-specific prerequisites (ρ) are unmet.
    /// A and B are provided for inspection; R is not fabricated.
    Partial {
        /// A operator output: edge differences from declared field and topology.
        a_field:            EdgeField,
        /// B operator output: successor accumulation over A.
        b_field:            EdgeField,
        /// The R admission prerequisites that remain unresolved.
        open_r_conditions:  Vec<OpenCondition>,
        /// Unresolved Kernel V8 mathematical conditions (present regardless).
        kernel_open:        Vec<KernelOpenCondition>,
    },

    /// Cannot begin computation. Missing declarations or validation failure.
    Blocked(Vec<OpenCondition>),
}
