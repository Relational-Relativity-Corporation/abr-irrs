/// IRRS Admission — D/M-Driven R Admission Check
///
/// Implements the full-operator admissibility condition for Kernel V8:
/// A→B→R is admissible when M establishes at least one interior locus
/// t = target(e) = source(f) for independently declared edges e and f.
///
/// Three outcomes:
///   ADMITTED              — all prerequisites met; R operator can execute
///   OBSERVED ZERO         — admitted but antisymmetric circulation = 0
///   OPEN(conditions)      — prerequisites not met; identifies which
///
/// The admission check does not fabricate values. When prerequisites are
/// unmet, it names them. When they are met but the result is zero, it
/// reports that observation faithfully.

use irrs_kernel::operators::EdgeField;

// ─── ParameterProvenance ─────────────────────────────────────────────────────

/// Provenance of declared domain-specific parameters (ρ_base, χ₀, cc).
///
/// These parameters are domain-specific because complexity of the declared
/// domain determines the characteristic scales. They cannot be universally
/// assigned; an Origin that supplies them from measurement has established
/// them; one that supplies them as starting values has not.
///
/// See `irrs-kernel` documentation: "Domain-specificity principle."
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterProvenance {
    /// Values declared from calibration against independently observable R
    /// output. The kernel was run over a declared observation; parameter
    /// values were adjusted until R output matched the observable.
    ObservationallyEstablished,

    /// Values declared to enable illustration of the operator mechanics.
    /// Not derived from R-output calibration over a declared observation.
    /// The DECLARATION REPORT labels these clearly.
    ///
    /// An illustrative value does not falsify the operator; it faithfully
    /// reports that the parameter is not yet observationally constrained.
    Illustrative,
}

// ─── OpenCondition ───────────────────────────────────────────────────────────

/// An unresolved R admission prerequisite.
///
/// When `AdmissionResult::Open` is returned, each condition names one
/// prerequisite that M has not satisfied. These are not errors — they are
/// the honest report of what declaration is missing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenCondition {
    /// The relational structure (topology) has not been declared.
    /// Without topology, no interior locus can be established.
    TopologyNotDeclared,

    /// Topology was declared but contains no interior locus.
    /// An interior locus requires a node t such that at least one edge
    /// terminates at t and at least one edge originates from t
    /// (independently declared: not the same edge in both roles).
    /// A topology with only one edge (s→t) has no interior node that is
    /// simultaneously a target and a source.
    NoInteriorLocus,

    /// The node field (numerical projection through M) has not been declared.
    NodeFieldNotDeclared,

    /// Domain-specific parameters ρ_base and χ₀ have not been declared.
    /// These cannot be defaulted. The domain must establish them through M.
    RhoNotDeclared,
}

// ─── AdmissionResult ─────────────────────────────────────────────────────────

/// The outcome of the full-operator admissibility check.
pub enum AdmissionResult {
    /// All prerequisites met. A→B→R may execute.
    Admitted {
        /// Provenance of ρ_base and χ₀ as declared by this session's M.
        parameter_provenance: ParameterProvenance,
    },

    /// Prerequisites not met. Lists every unresolved condition.
    /// Callers should address each listed condition and re-admit.
    Open(Vec<OpenCondition>),
}

// ─── ExecutionResult ─────────────────────────────────────────────────────────

/// The outcome of executing A→B→R over an admitted declaration.
///
/// Note: `EdgeField` does not implement `Debug`; this enum is not `Debug`.
pub enum ExecutionResult {
    /// R operator executed; antisymmetric term ≠ 0 on at least one edge.
    /// Detection indicators 𝟙[e] are read from the R output, not parameter-set.
    RelationalOutput {
        /// The EdgeField output of operator_e (A, B, R, and E assembled).
        edge_field:            EdgeField,
        /// Provenance of ρ_base and χ₀ used in this execution.
        parameter_provenance:  ParameterProvenance,
        /// 𝟙[e] — 1.0 if R antisymmetric term ≠ 0, else 0.0.
        /// One value per spatial edge. Read from R output; not a parameter.
        detection_indicators:  Vec<f64>,
    },

    /// R operator executed; antisymmetric term = 0 on every edge.
    ///
    /// This is an observable outcome, not a prerequisite failure.
    /// A topology with a declared interior locus and a flat node field
    /// (e.g., x = [5, 5, 5]) produces this outcome.
    ///
    /// OBSERVED ZERO ≠ audit A-3 (Kernel V8 OPEN): the latter concerns
    /// detection events with undeclared topology. This outcome is the
    /// correctly executed R operator returning zero antisymmetric circulation.
    ObservedZero {
        /// The EdgeField output of operator_e (A, B, R identical here).
        edge_field:           EdgeField,
        /// Provenance of ρ_base and χ₀ used in this execution.
        parameter_provenance: ParameterProvenance,
    },
}
