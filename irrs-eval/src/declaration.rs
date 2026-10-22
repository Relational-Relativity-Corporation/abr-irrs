/// JSON-deserializable declaration format.
///
/// A `Declaration` is supplied by a domain author as an external JSON file.
/// `irrs-eval` reads it at runtime — no recompilation required to accept a new problem space.
///
/// Load a declaration:
///   let text = std::fs::read_to_string(path)?;
///   let decl: Declaration = serde_json::from_str(&text)?;

use serde::{Serialize, Deserialize};

// ── Top-level declaration document ───────────────────────────────────────────

/// Complete declaration supplied via `--declaration <path>.json`.
///
/// Contains all information required for the evaluator to produce an
/// `EvaluationRecord` without any domain-specific Rust code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Declaration {
    /// Domain identifier, e.g. "datafeel".
    pub domain: String,

    /// Topology identifier declared in the relational gate, e.g. "circadian_v1".
    pub topology_id: String,

    /// Semantic version of the application declaring this topology.
    pub version: String,

    /// Optional human-readable description of the problem space.
    #[serde(default)]
    pub description: Option<String>,

    /// Continuous-process persistence declaration.
    ///
    /// When present, the evaluator enforces the Continuous Relational Process Rule:
    /// no synthetic initialization, OPEN reporting for unavailable prior evidence,
    /// and Kernel V8 persistence operator path verification.
    ///
    /// When absent, the evaluator records that the continuous-process posture is
    /// undeclared and adds a note to the evaluation record for Origin review.
    #[serde(default)]
    pub persistence: Option<PersistenceDeclaration>,

    /// Every input value involved in the declared problem space.
    /// Includes values submitted to the gate AND values the gate produces.
    pub inputs: Vec<InputDeclaration>,

    /// The four required Regional M prerequisite fields.
    /// No defaults. If any field is absent the declaration fails.
    pub regional_prerequisite: RegionalPrerequisiteDecl,

    /// Every authorized mapping that may be applied by the gate.
    pub mappings: Vec<MappingDeclaration>,
}

// ── Input declarations ────────────────────────────────────────────────────────

/// Declaration of a single input value in the problem space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputDeclaration {
    /// Short identifier matching the name used in mapping descriptions.
    pub name: String,

    /// One of five classifications. See `InputClassification`.
    pub classification: InputClassification,

    /// Identity of the system, sensor, configuration file, or process
    /// that provides or generates this value.
    pub source_identity: String,

    /// How this value is acquired or generated: sensor read, config load,
    /// formula application, simulation step, mathematical stipulation.
    pub acquisition_method: String,

    /// Sequence of transformations applied before this value reaches the gate,
    /// if any. Absent when the value is submitted raw.
    #[serde(default)]
    pub transformation_history: Option<String>,

    /// Plain-language statement of how this value relates to the declared problem space.
    pub relationship_to_problem: String,

    /// References to calibration records, specifications, or test results
    /// that support the provenance claim. May be empty.
    #[serde(default)]
    pub supporting_evidence: Vec<EvidenceRef>,

    /// Known limitations, gaps, or unresolved conditions in the provenance.
    #[serde(default)]
    pub known_limitations: Option<String>,

    /// Claim about the strength of provenance for this input.
    /// `Declared` is the default; `Verified` requires supporting evidence.
    pub provenance_status: ProvenanceStatus,

    /// True when this value is submitted by the Generator in its `DeclarationPackage`.
    /// False for values the gate produces (e.g., `DerivedValue` outputs).
    pub submitted_to_gate: bool,
}

/// Five classifications describing the observational basis of an input.
///
/// These describe provenance. They do not independently determine whether
/// an operator can execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InputClassification {
    /// An input supported by an identified observational acquisition process.
    /// Example: a sensor reading, a measured time elapsed.
    PrimaryObservable,

    /// A value supplied as a declared configuration or reference.
    /// Example: a declared period, a calibration constant.
    DeclaredParameter,

    /// A value produced through an identified transformation of other inputs.
    /// The gate produces this; the Generator does not submit it.
    DerivedValue,

    /// A value generated within a declared simulation or test fixture.
    /// The simulation boundary must be identified.
    SimulatedInput,

    /// A conceptual or stipulated value without a demonstrated observational basis.
    /// Example: a unit-circle phase for algebraic verification.
    AbstractInput,
}

/// Claim about the degree of independent support for a provenance assertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceStatus {
    /// The source and history asserted by the submitting party.
    /// Not independently verified. The evaluator records this explicitly.
    Declared,

    /// The portion of the history supported by evidence actually evaluated.
    /// Requires non-empty `supporting_evidence`.
    Verified,

    /// The claim could not be independently verified.
    /// The record marks this input as unverified and notes it for Origin review.
    Unverified,
}

/// A reference to external evidence supporting a provenance claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// Short identifier for this piece of evidence, e.g. "cal-2024-03".
    pub id: String,

    /// Human-readable description of what this evidence is and how it supports the claim.
    pub description: String,

    /// Kind of evidence: "calibration", "specification", "test_result",
    /// "measurement_log", "design_document", or similar.
    pub kind: String,
}

// ── Continuous-process persistence declaration ────────────────────────────────

/// Declaration of the continuous-process persistence posture.
///
/// Every evaluation enters an ongoing relational process. The evaluation window
/// opens onto a process that is already running; the first admitted observation
/// begins the evaluation record, not the physical process.
///
/// This declaration captures:
///   * How the evaluation window opens (process_entry statement).
///   * Whether prior relational evidence is available at window open.
///   * Which Kernel V8 persistence operators are declared for this process.
///
/// Synthetic initialization is PROHIBITED. Fabricated zeros, cold-start states,
/// and non-observed prior relational results are inadmissible. Actual observed
/// zero values remain admissible.
///
/// If prior evidence is unavailable, declare `prior_evidence: Open`. The
/// evaluator reports this as an OPEN condition and does not fabricate prior
/// evidence or prevent otherwise admissible calculations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceDeclaration {
    /// Plain-language statement of how the evaluation window opens onto the
    /// ongoing process. Must assert that the process is continuous and that
    /// the first admitted observation begins the evaluation record, not the
    /// physical process.
    pub process_entry: String,

    /// Status of prior relational evidence at evaluation window open.
    ///
    /// * `Available` — prior relational evidence from the running process is
    ///   present. Evaluation proceeds normally.
    /// * `Open` — prior evidence is not yet available. The evaluator reports
    ///   this as an OPEN condition. Otherwise admissible calculations proceed.
    ///   An unavailable prior is NOT equivalent to a zero prior.
    /// * `Synthetic` — a fabricated, cold-start, or non-observed prior has
    ///   been declared. PROHIBITED. The evaluator fails the declaration.
    pub prior_evidence: PriorEvidenceStatus,

    /// Traceable references to the prior relational evidence.
    ///
    /// Required when `prior_evidence: Available`. An availability claim without
    /// traceable references cannot be independently assessed; the evaluator
    /// downgrades it to OPEN.
    ///
    /// A descriptive assertion in `process_entry` does NOT substitute for
    /// structured evidence references. Only this field is checked.
    #[serde(default)]
    pub prior_evidence_refs: Vec<EvidenceRef>,

    /// The Kernel V8 persistence operator path declared by the submitter.
    ///
    /// Must list the operators that will be applied in sequence. For Kernel V8
    /// compliance the path must be exactly:
    /// `["A_persistence", "B_persistence", "R_persistence"]` in that order.
    pub persistence_path: Vec<String>,
}

/// Status of prior relational evidence at evaluation window open.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorEvidenceStatus {
    /// Prior relational evidence is available from the running process.
    /// Evaluation proceeds with the available prior.
    Available,

    /// Prior relational evidence is not available at evaluation window open.
    ///
    /// This is a declared OPEN condition. The evaluator reports the dependency
    /// as OPEN and does not fabricate prior evidence or prevent otherwise
    /// admissible calculations. The OPEN condition carries forward until prior
    /// relational evidence is available from an actual completed evaluation cycle.
    ///
    /// Note: this is not the same as a prior value of zero. "Unavailable" means
    /// the evidence does not yet exist, not that it is numerically zero.
    Open,

    /// A synthetic, fabricated, or cold-start prior relational result has been
    /// declared as prior relational evidence. PROHIBITED.
    ///
    /// `PersistenceState::cold_start` is not admissible. Fabricated zeros and
    /// non-observed initialization values are not admissible. If the prior is
    /// genuinely absent, declare `Open` instead.
    ///
    /// Actual observed sensor values of 0.0 remain admissible; they are
    /// `PrimaryObservable` inputs acquired from a real sensor, not synthetic.
    Synthetic,
}

// ── Regional M prerequisite declaration ──────────────────────────────────────

/// The four required fields of the Regional M prerequisite.
///
/// Mirrors the four fields required by `RegionBuilder` in `relational-gate`.
/// No defaults: every field must be explicitly supplied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalPrerequisiteDecl {
    /// Region name matching the topology ID declared in the gate.
    pub region_name: String,

    /// Lower bound of the observable extent (exclusive).
    pub extents_lower: f64,

    /// Upper bound of the observable extent (exclusive).
    pub extents_upper: f64,

    /// Label for the extent unit, e.g. "phase_fraction".
    pub extents_label: String,

    /// What the authorized mapping preserves across the family boundary.
    pub orientation_preserved: String,

    /// What the authorized mapping discards.
    pub orientation_discarded: String,

    /// Short identifier for the regional admission rule.
    pub admission_rule_name: String,

    /// Plain-language description of the admission rule.
    pub admission_rule_description: String,

    /// True when the domain builder has called `establish(locus)` with a
    /// verified interior point. The evaluator records this; it does not
    /// re-verify at evaluation time.
    pub interior_locus_verified: bool,
}

// ── Mapping declarations ──────────────────────────────────────────────────────

/// Declaration of one authorized mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingDeclaration {
    /// Name matching the `AuthorizedMapping` registered in the gate.
    pub name: String,

    /// Description of the inputs this mapping receives.
    pub input_description: String,

    /// Description of the value this mapping produces.
    pub output_description: String,

    /// What relational structure is preserved across the mapping.
    pub preserves: String,

    /// What is intentionally discarded.
    pub discards: String,
}
