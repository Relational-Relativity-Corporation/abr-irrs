/// Serializable evaluation record.
///
/// Produced by `Evaluator::evaluate()` before any runtime gate submission.
/// Written to `eval/record.json` by CI. Reviewed by Origin.
/// Origin commits `eval/authorization.json` to authorize the build.

use serde::{Serialize, Deserialize};

// ── Top-level record ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
    /// Stable identifier: "{domain}-{topology_id}-{version}".
    /// Referenced in authorization.json to bind the two together.
    pub record_id: String,

    pub domain: String,
    pub topology_id: String,
    pub version: String,

    /// Classification of every value in the declaration.
    pub observable_inventory: Vec<InventoryItem>,

    /// Verification of the Regional M prerequisite.
    pub regional_prerequisite: PrerequisiteReport,

    /// Trace of every authorized mapping: what goes in, what comes out,
    /// what is preserved, what is discarded, and who executes it.
    pub mapping_chain: Vec<MappingChainEntry>,

    /// Result of the evaluation.
    pub status: EvaluationStatus,

    /// Present only if status is `Failed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,

    /// Present only after Origin commits eval/authorization.json and CI re-runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<OriginAuthorization>,
}

// ── Observable inventory ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub name: String,

    /// Classification of this value.
    pub kind: InventoryKind,

    /// Where the value originates: sensor ID, config key, derived expression.
    pub source: String,

    /// Plain-language statement of physical basis (or lack thereof).
    pub physical_basis: String,

    /// Whether this value appears in the DeclarationPackage submitted to the gate.
    /// DerivedValues are produced by the gate; the Generator does not submit them.
    pub submitted_to_gate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryKind {
    /// Comes directly from a physical sensor or measurement instrument.
    PrimaryObservable,
    /// A declared constant or configuration value; not sensed from the environment.
    DeclaredParameter,
    /// Computed from observables and parameters by an authorized mapping.
    /// The gate produces this value; the Generator does not.
    DerivedValue,
}

// ── Regional M prerequisite report ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrerequisiteReport {
    pub region_name: String,
    pub extents: String,
    pub orientation_preserved: String,
    pub orientation_discarded: String,
    pub admission_rule_name: String,
    pub admission_rule_description: String,

    /// True if all four prerequisite fields were declared (no defaults).
    pub all_fields_declared: bool,

    /// True if `establish(locus)` was called and the locus was interior.
    /// The gate enforces this at build time, not at runtime submission.
    pub interior_locus_verified: bool,
}

// ── Mapping chain ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingChainEntry {
    pub mapping_name: String,
    pub input_description: String,
    pub output_description: String,
    pub preserves: String,
    pub discards: String,
    /// Always "AdmissionGate (not Generator)".
    pub executor: String,
}

// ── Evaluation status ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationStatus {
    /// All prerequisites satisfied; awaiting Origin review and authorization.
    PassedAwaitingAuthorization,
    /// Origin has reviewed and committed eval/authorization.json.
    Authorized,
    /// A prerequisite was not satisfied; build must not proceed.
    Failed,
}

// ── Origin authorization ──────────────────────────────────────────────────────

/// Written by Origin to eval/authorization.json after reviewing the record.
/// The evaluator merges this into the record on its next run if record_id matches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginAuthorization {
    /// Must match EvaluationRecord.record_id exactly.
    pub record_id: String,
    pub authorized_by: String,
    pub authorized_at: String,
    /// Origin's explicit statement about what was reviewed and authorized.
    pub statement: String,
}
