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
