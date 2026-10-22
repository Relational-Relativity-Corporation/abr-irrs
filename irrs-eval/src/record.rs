/// Serializable evaluation record.
///
/// Produced by `Evaluator::evaluate()` before any runtime gate submission.
/// Written to `eval/record.json` by CI. Reviewed by Origin.
/// Origin commits `eval/authorization.json` to authorize the build.

use serde::{Serialize, Deserialize};
use crate::provenance::ProvenanceSummary;

// ── Top-level record ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
    /// Stable human-readable identifier: "{domain}-{topology_id}-{version}".
    pub record_id: String,

    /// SHA-256 hex digest of the raw declaration JSON text.
    ///
    /// Authorization is only valid when `authorization.declaration_digest`
    /// matches this field exactly. A change to any byte of the declaration
    /// invalidates the existing authorization.
    pub declaration_digest: String,

    pub domain: String,
    pub topology_id: String,
    pub version: String,

    /// Classification, provenance claims, and evaluator assessments for every
    /// input value in the declaration. Includes values submitted to the gate
    /// AND values the gate produces (DerivedValue).
    pub input_inventory: Vec<InventoryItem>,

    /// Aggregated view of provenance status across all inputs.
    /// The note explicitly states what the evaluator can and cannot determine.
    pub provenance_summary: ProvenanceSummary,

    /// Verification of the Regional M prerequisite.
    pub regional_prerequisite: PrerequisiteReport,

    /// Trace of every authorized mapping: inputs, outputs, what is preserved,
    /// what is discarded, and who executes it.
    pub mapping_chain: Vec<MappingChainEntry>,

    /// Result of the evaluation.
    pub status: EvaluationStatus,

    /// Present only if status is `Failed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,

    /// Present only after Origin commits eval/authorization.json and CI re-runs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization: Option<OriginAuthorization>,

    /// Continuous-process rule evaluation report.
    ///
    /// Documents the evaluator's findings on the Continuous Relational Process Rule:
    /// process entry posture, prior evidence status, persistence operator path,
    /// and any OPEN conditions arising from unavailable prior relational evidence.
    ///
    /// Present whenever a `persistence` declaration is provided or the evaluator
    /// needs to note that the posture is undeclared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuous_process: Option<ContinuousProcessReport>,

    /// Evaluator observations — inconsistencies, provenance gaps, or informational
    /// notes not severe enough to fail evaluation. Reviewed by Origin.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evaluation_notes: Vec<String>,
}

// ── Input inventory ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub name: String,

    /// Classification from the declaration (submitter's claim).
    pub classification: InventoryKind,

    /// Source identity from the declaration.
    pub source_identity: String,

    /// Acquisition or generation method.
    pub acquisition_method: String,

    /// Relationship to the declared problem space.
    pub relationship_to_problem: String,

    /// Transformation history, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transformation_history: Option<String>,

    /// Number of supporting evidence references supplied.
    pub evidence_count: usize,

    /// Known limitations from the declaration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub known_limitations: Option<String>,

    /// Provenance status claimed by the submitter.
    pub provenance_status_claimed: crate::declaration::ProvenanceStatus,

    /// The evaluator's structural assessment of the provenance claim.
    ///
    /// This is NOT independent verification of physical acquisition.
    /// It is a consistency check: does the claim make internal sense?
    /// (e.g., `Verified` with no evidence references is inconsistent.)
    /// Independent verification is Origin's determination to make.
    pub evaluator_provenance_assessment: ProvenanceAssessment,

    /// Whether this value is submitted by the Generator in its DeclarationPackage.
    pub submitted_to_gate: bool,
}

/// The evaluator's structural assessment of a provenance claim.
///
/// This assesses the internal consistency of the claim, not whether
/// the underlying acquisition actually occurred as described.
/// The evaluator cannot independently verify physical acquisition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceAssessment {
    /// The claim is internally consistent.
    /// Declared with or without evidence: consistent.
    /// Verified with at least one evidence reference: consistent.
    ConsistentClaim,

    /// The claim is internally inconsistent.
    /// Example: status is `Verified` but no evidence references are provided.
    InconsistentClaim {
        /// Reason the evaluator determined this claim to be inconsistent.
        reason: String,
    },
}

/// Five classifications describing the observational basis of an input.
///
/// These describe provenance. They do not independently determine whether
/// an operator can execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InventoryKind {
    /// An input supported by an identified observational acquisition process.
    PrimaryObservable,

    /// A value supplied as a declared configuration or reference.
    DeclaredParameter,

    /// A value produced through an identified transformation of other inputs.
    /// The gate produces this; the Generator does not submit it.
    DerivedValue,

    /// A value generated within a declared simulation or test fixture.
    SimulatedInput,

    /// A conceptual or stipulated value without a demonstrated observational basis.
    AbstractInput,
}

// ── Regional M prerequisite report ───────────────────────────────────────────

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

    /// True if the submitter declared that `establish(locus)` was called
    /// with a verified interior point.
    ///
    /// This is a SUBMITTER CLAIM, not an independent evaluator check.
    /// The evaluator records what was declared; it does not re-execute
    /// the gate's locus verification at evaluation time.
    pub interior_locus_verified_claimed: bool,
}

// ── Mapping chain ─────────────────────────────────────────────────────────────

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

// ── Continuous-process rule report ───────────────────────────────────────────

/// Report of the Continuous Relational Process Rule evaluation.
///
/// Every evaluation enters an ongoing relational process. This report documents
/// what the evaluator found about the process entry posture and prior relational
/// evidence status declared by the submitter.
///
/// An OPEN condition in `open_conditions` does not fail the evaluation.
/// It is a declared dependency reported for Origin review and carried forward
/// until prior relational evidence is available from an actual evaluation cycle.
///
/// A `SyntheticViolation` in `prior_evidence` fails the evaluation. The failure
/// reason is also written to the top-level `failure_reason` field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousProcessReport {
    /// True when a `persistence` block was present in the declaration.
    pub process_entry_declared: bool,

    /// The submitter's plain-language statement of how the evaluation window
    /// opens onto the ongoing process, if declared.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_entry_statement: Option<String>,

    /// Evaluator finding on prior relational evidence status.
    pub prior_evidence: PriorEvidenceReport,

    /// The declared Kernel V8 persistence operator path, if any.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub persistence_path: Vec<String>,

    /// Whether the declared persistence operator path is valid.
    ///
    /// `true` when `persistence_path` is exactly
    /// `["A_persistence", "B_persistence", "R_persistence"]` in that order.
    /// `false` when the path is absent, incomplete, or in the wrong order.
    pub persistence_path_valid: bool,

    /// If `persistence_path_valid` is false, the reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistence_path_validation_note: Option<String>,

    /// Count of traceable evidence references supplied for the prior evidence claim.
    ///
    /// Zero when `prior_evidence: open` or when `available` was claimed without refs.
    pub prior_evidence_refs_count: usize,

    /// OPEN conditions arising from the continuous-process evaluation.
    ///
    /// An OPEN condition means a relational dependency exists but cannot be
    /// resolved from available evidence. It does NOT mean the value is zero.
    /// The evaluator does not fabricate evidence; it reports the gap.
    /// Otherwise admissible calculations proceed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_conditions: Vec<String>,
}

/// The evaluator's finding on prior relational evidence status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorEvidenceReport {
    /// Prior relational evidence is declared as available.
    /// Evaluation proceeds with the available prior.
    Available,

    /// Prior relational evidence is unavailable at evaluation window open.
    ///
    /// Reported as an OPEN condition. Otherwise admissible calculations proceed.
    /// The dependency is declared, not fabricated as zero.
    Open,

    /// A synthetic, fabricated, or cold-start prior was declared. PROHIBITED.
    /// The evaluator fails the declaration. See `failure_reason` for details.
    SyntheticViolation,

    /// No persistence declaration was present in the declaration.
    /// The continuous-process posture is undeclared. Origin should confirm.
    Undeclared,
}

// ── Evaluation status ─────────────────────────────────────────────────────────

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
/// The evaluator applies this on its next run only when both fields match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginAuthorization {
    /// Must match `EvaluationRecord.record_id` exactly.
    pub record_id: String,

    /// Must match `EvaluationRecord.declaration_digest` exactly.
    ///
    /// Copy from the evaluation record produced for the commit you reviewed.
    /// A change to any byte of the declaration produces a different digest
    /// and invalidates this authorization.
    pub declaration_digest: String,

    pub authorized_by: String,
    pub authorized_at: String,
    /// Origin's explicit statement about what was reviewed and authorized.
    pub statement: String,
}
