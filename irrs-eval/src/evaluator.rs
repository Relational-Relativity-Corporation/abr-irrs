/// Pre-build evaluator.
///
/// Takes a `Declaration` (loaded from a JSON file) and the SHA-256 digest
/// of the raw declaration text, and produces an `EvaluationRecord` that
/// documents what the evaluator found.
///
/// The evaluator runs before any `DeclarationPackage` is submitted to the gate.
/// It does not execute mappings. It does not require the gate to be constructed.
/// It does not contain application-specific logic.

use crate::declaration::{Declaration, InputClassification, ProvenanceStatus, PriorEvidenceStatus};
use crate::record::{
    EvaluationRecord, InventoryItem, InventoryKind, ProvenanceAssessment,
    PrerequisiteReport, MappingChainEntry, EvaluationStatus,
    ContinuousProcessReport, PriorEvidenceReport,
};
use crate::provenance;
use crate::evidence;

pub struct Evaluator;

impl Evaluator {
    /// Evaluate a declaration and produce a record.
    ///
    /// `declaration_digest` is the SHA-256 hex digest of the raw declaration
    /// JSON text, computed before parsing. It is embedded in the record and
    /// must match the authorization's `declaration_digest` exactly.
    ///
    /// Does not execute mappings. Does not submit values to the gate.
    /// Does not require the gate to be constructed or the application to be present.
    pub fn evaluate(decl: Declaration, declaration_digest: String) -> EvaluationRecord {
        let record_id = format!("{}-{}-{}", decl.domain, decl.topology_id, decl.version);

        // ── Input inventory ───────────────────────────────────────────────────
        let input_inventory: Vec<InventoryItem> = decl.inputs
            .iter()
            .map(|i| {
                let kind       = map_classification(&i.classification);
                let assessment = assess_provenance_claim(i);
                InventoryItem {
                    name:                          i.name.clone(),
                    classification:                kind,
                    source_identity:               i.source_identity.clone(),
                    acquisition_method:            i.acquisition_method.clone(),
                    relationship_to_problem:       i.relationship_to_problem.clone(),
                    transformation_history:        i.transformation_history.clone(),
                    evidence_count:                i.supporting_evidence.len(),
                    known_limitations:             i.known_limitations.clone(),
                    provenance_status_claimed:     i.provenance_status.clone(),
                    evaluator_provenance_assessment: assessment,
                    submitted_to_gate:             i.submitted_to_gate,
                }
            })
            .collect();

        // ── Provenance summary ────────────────────────────────────────────────
        let provenance_summary = provenance::analyze(&decl.inputs);

        // ── Evidence coverage notes ───────────────────────────────────────────
        let mut evaluation_notes = evidence::evaluate_coverage(&decl.inputs);

        // ── Regional M prerequisite report ────────────────────────────────────
        let p = &decl.regional_prerequisite;
        let all_fields_declared =
            !p.region_name.is_empty()
            && !p.orientation_preserved.is_empty()
            && !p.orientation_discarded.is_empty()
            && !p.admission_rule_name.is_empty()
            && !p.admission_rule_description.is_empty();

        let regional_prerequisite = PrerequisiteReport {
            region_name:               p.region_name.clone(),
            extents:                   format!("({}, {}) {}",
                                           p.extents_lower,
                                           p.extents_upper,
                                           p.extents_label),
            orientation_preserved:     p.orientation_preserved.clone(),
            orientation_discarded:     p.orientation_discarded.clone(),
            admission_rule_name:       p.admission_rule_name.clone(),
            admission_rule_description: p.admission_rule_description.clone(),
            all_fields_declared,
            interior_locus_verified_claimed: p.interior_locus_verified,
        };

        // ── Mapping chain ─────────────────────────────────────────────────────
        let mapping_chain: Vec<MappingChainEntry> = decl.mappings
            .iter()
            .map(|m| MappingChainEntry {
                mapping_name:        m.name.clone(),
                input_description:   m.input_description.clone(),
                output_description:  m.output_description.clone(),
                preserves:           m.preserves.clone(),
                discards:            m.discards.clone(),
                executor:            "AdmissionGate (not Generator)".to_string(),
            })
            .collect();

        // ── Problem space description ─────────────────────────────────────────
        if let Some(desc) = &decl.description {
            evaluation_notes.insert(0, format!("Problem space: {}", desc));
        }

        // ── Continuous Relational Process Rule ────────────────────────────────
        // Every evaluation enters an ongoing relational process. Synthetic
        // initialization (fabricated zeros, cold-start states, non-observed
        // prior relational results) is prohibited. If prior evidence is
        // unavailable the evaluator reports it as OPEN, not as zero.
        let (continuous_process, synthetic_violation) =
            evaluate_continuous_process(&decl.persistence, &mut evaluation_notes);

        // ── Provenance inconsistency notes ────────────────────────────────────
        for item in &input_inventory {
            if let ProvenanceAssessment::InconsistentClaim { reason } =
                &item.evaluator_provenance_assessment
            {
                evaluation_notes.push(format!(
                    "PROVENANCE INCONSISTENCY — input '{}': {}",
                    item.name, reason
                ));
            }
        }

        // ── Evaluation result ─────────────────────────────────────────────────
        // Synthetic initialization is checked first: it is the most fundamental
        // violation of the Continuous Relational Process Rule. All other checks
        // follow in precedence order.
        let (status, failure_reason) = if synthetic_violation {
            (EvaluationStatus::Failed,
             Some("Continuous Relational Process Rule violation: synthetic initialization \
                   is prohibited. Fabricated zeros, cold-start states, and non-observed \
                   prior relational results are not admissible as prior relational evidence. \
                   PersistenceState::cold_start is not a valid entry point at Kernel V8. \
                   If prior relational evidence is genuinely unavailable at evaluation window \
                   open, declare prior_evidence as 'open' — the evaluator will report the \
                   dependency as OPEN without fabricating a value. Actual sensor readings of \
                   0.0 remain admissible; they are PrimaryObservable, not synthetic.".to_string()))
        } else if !all_fields_declared {
            (EvaluationStatus::Failed,
             Some("Regional M prerequisite: one or more required fields not declared (no defaults permitted)".to_string()))
        } else if !p.interior_locus_verified {
            (EvaluationStatus::Failed,
             Some("Interior-locus condition not verified: establish(locus) must be called with a verified interior point before topology instantiation".to_string()))
        } else if mapping_chain.is_empty() {
            (EvaluationStatus::Failed,
             Some("No authorized mappings declared: at least one mapping must be registered".to_string()))
        } else if input_inventory.is_empty() {
            (EvaluationStatus::Failed,
             Some("No inputs declared: at least one input must appear in the inventory".to_string()))
        } else {
            (EvaluationStatus::PassedAwaitingAuthorization, None)
        };

        EvaluationRecord {
            record_id,
            declaration_digest,
            domain:               decl.domain,
            topology_id:          decl.topology_id,
            version:              decl.version,
            input_inventory,
            provenance_summary,
            regional_prerequisite,
            mapping_chain,
            continuous_process,
            status,
            failure_reason,
            authorization:        None,
            evaluation_notes,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Evaluate the Continuous Relational Process Rule.
///
/// Returns `(report, synthetic_violation)`.
///
/// * `report` — the `ContinuousProcessReport` to embed in the record.
/// * `synthetic_violation` — `true` when a prohibited synthetic prior was
///   declared; the caller must fail the evaluation.
///
/// OPEN conditions are appended to `evaluation_notes` and collected in the
/// report's `open_conditions` field. They do NOT fail the evaluation.
///
/// Governing rules:
/// 1. Every evaluation enters an ongoing relational process.
/// 2. No synthetic zero, cold-start state, or fabricated prior is permitted.
/// 3. Actual observed zero values (PrimaryObservable) remain admissible.
/// 4. Where prior evidence is unavailable, report as OPEN; do not fabricate.
/// 5. Preserve provenance: O → M → D → operators → result → comparison.
fn evaluate_continuous_process(
    persistence: &Option<crate::declaration::PersistenceDeclaration>,
    evaluation_notes: &mut Vec<String>,
) -> (Option<ContinuousProcessReport>, bool) {
    match persistence {
        None => {
            // No persistence declaration. Note the gap; do not fail.
            evaluation_notes.push(
                "CONTINUOUS PROCESS: Persistence declaration absent. \
                 Continuous-process posture is undeclared. Submitters whose \
                 evaluation enters an ongoing relational process should declare \
                 a 'persistence' block including process_entry, prior_evidence \
                 status, and persistence_path. Origin should confirm the posture.".to_string()
            );
            (Some(ContinuousProcessReport {
                process_entry_declared:    false,
                process_entry_statement:   None,
                prior_evidence:            PriorEvidenceReport::Undeclared,
                persistence_path:          vec![],
                open_conditions:           vec![],
            }), false)
        }

        Some(p) => {
            let mut open_conditions: Vec<String> = vec![];
            let mut synthetic_violation = false;

            let prior_evidence_report = match &p.prior_evidence {
                PriorEvidenceStatus::Available => {
                    evaluation_notes.push(
                        "CONTINUOUS PROCESS: Prior relational evidence declared as available. \
                         Evaluation proceeds from existing relational prior. \
                         Process entry is onto an ongoing process; first observation \
                         begins the evaluation record, not the physical process.".to_string()
                    );
                    PriorEvidenceReport::Available
                }

                PriorEvidenceStatus::Open => {
                    // Prior evidence unavailable. This is an OPEN condition.
                    // Report it. Do not fabricate a value. Do not fail.
                    let cond = "OPEN — Prior relational evidence unavailable at evaluation \
                                window open. The evaluation record begins with the first \
                                admitted observation. The prior relational result does not \
                                yet exist; it is not zero. No value is fabricated. \
                                Otherwise admissible calculations proceed. This dependency \
                                resolves when the first complete evaluation cycle produces \
                                an actual relational result.".to_string();
                    evaluation_notes.push(format!("CONTINUOUS PROCESS: {}", cond));
                    open_conditions.push(cond);
                    PriorEvidenceReport::Open
                }

                PriorEvidenceStatus::Synthetic => {
                    // Prohibited synthetic initialization. Fail.
                    let violation =
                        "PROHIBITED SYNTHETIC INITIALIZATION declared in persistence block. \
                         Fabricated zeros, cold-start states, and non-observed prior \
                         relational results are inadmissible under the Continuous Relational \
                         Process Rule. PersistenceState::cold_start is not valid at \
                         Kernel V8 and must not be reintroduced. Actual observed sensor \
                         readings of 0.0 are admissible as PrimaryObservable inputs; \
                         only fabricated priors are prohibited. If prior relational evidence \
                         is genuinely absent, declare prior_evidence as 'open'.".to_string();
                    evaluation_notes.push(format!("CONTINUOUS PROCESS VIOLATION: {}", violation));
                    synthetic_violation = true;
                    PriorEvidenceReport::SyntheticViolation
                }
            };

            // Note persistence path if declared.
            if !p.persistence_path.is_empty() {
                evaluation_notes.push(format!(
                    "CONTINUOUS PROCESS: Declared persistence path: [{}]. \
                     Kernel V8 requires A_persistence → B_persistence → R_persistence.",
                    p.persistence_path.join(", ")
                ));
            } else {
                evaluation_notes.push(
                    "CONTINUOUS PROCESS: No persistence operator path declared. \
                     Kernel V8 requires persistence_path: \
                     [\"A_persistence\", \"B_persistence\", \"R_persistence\"].".to_string()
                );
            }

            let report = ContinuousProcessReport {
                process_entry_declared:    true,
                process_entry_statement:   Some(p.process_entry.clone()),
                prior_evidence:            prior_evidence_report,
                persistence_path:          p.persistence_path.clone(),
                open_conditions,
            };
            (Some(report), synthetic_violation)
        }
    }
}

fn map_classification(c: &InputClassification) -> InventoryKind {
    match c {
        InputClassification::PrimaryObservable => InventoryKind::PrimaryObservable,
        InputClassification::DeclaredParameter => InventoryKind::DeclaredParameter,
        InputClassification::DerivedValue      => InventoryKind::DerivedValue,
        InputClassification::SimulatedInput    => InventoryKind::SimulatedInput,
        InputClassification::AbstractInput     => InventoryKind::AbstractInput,
    }
}

/// Assess whether a provenance claim is internally consistent.
///
/// This is a structural check, not independent verification.
/// The evaluator cannot confirm that a physical acquisition actually occurred.
fn assess_provenance_claim(input: &crate::declaration::InputDeclaration) -> ProvenanceAssessment {
    match &input.provenance_status {
        // Declared: always structurally consistent (a bare assertion is still a valid claim).
        ProvenanceStatus::Declared => ProvenanceAssessment::ConsistentClaim,

        // Unverified: always structurally consistent (an honest admission of limitation).
        ProvenanceStatus::Unverified => ProvenanceAssessment::ConsistentClaim,

        // Verified: structurally inconsistent without supporting evidence.
        // A claim of "verified" requires at least one evidence reference to be coherent.
        ProvenanceStatus::Verified => {
            if input.supporting_evidence.is_empty() {
                ProvenanceAssessment::InconsistentClaim {
                    reason: "Status claimed as 'verified' but no supporting evidence \
                             references are provided. A verification claim without \
                             evidence cannot be assessed. Change status to 'declared' \
                             or provide evidence references.".to_string(),
                }
            } else {
                ProvenanceAssessment::ConsistentClaim
            }
        }
    }
}
