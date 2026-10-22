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
        //
        // The persistence declaration is now required (UT-05). Absence of a
        // persistence block fails the evaluation before regional checks.
        let (continuous_process, cp_failure) =
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
        // Continuous-process failures (missing persistence, invalid path, synthetic
        // initialization) are checked first — they are the most fundamental
        // violations of the Continuous Relational Process Rule. All other checks
        // follow in precedence order.
        let (status, failure_reason) = if let Some(cp_fail_msg) = cp_failure {
            (EvaluationStatus::Failed, Some(cp_fail_msg))
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

/// Required Kernel V8 persistence operator path (exact order required).
const REQUIRED_PATH: &[&str] = &["A_persistence", "B_persistence", "R_persistence"];

/// Validate that the declared persistence operator path is exactly the
/// required Kernel V8 sequence: [A_persistence, B_persistence, R_persistence].
///
/// Returns `Ok(())` when the path is valid, or `Err(reason)` when it is not.
/// Three distinct failure modes:
///   - Empty path (UT-06)
///   - Incomplete path — some operators present but not all (UT-07)
///   - Wrong order or extra entries (UT-08)
fn validate_persistence_path(path: &[String]) -> Result<(), String> {
    if path.is_empty() {
        return Err(
            "Persistence path is empty. Kernel V8 requires exactly: \
             [A_persistence, B_persistence, R_persistence] in that order. \
             An empty path is not a valid persistence declaration.".to_string()
        );
    }

    let missing: Vec<&str> = REQUIRED_PATH
        .iter()
        .filter(|&&op| !path.iter().any(|p| p == op))
        .copied()
        .collect();

    if !missing.is_empty() {
        return Err(format!(
            "Persistence path incomplete. Missing operator(s): [{}]. \
             Required sequence: [A_persistence, B_persistence, R_persistence]. \
             A partial path is not a valid persistence path under Kernel V8. \
             A correct operator name does not establish correct operator execution; \
             all three stages must be declared.",
            missing.join(", ")
        ));
    }

    let required: Vec<String> = REQUIRED_PATH.iter().map(|s| s.to_string()).collect();
    if path != required.as_slice() {
        return Err(format!(
            "Persistence path operators present but in incorrect order or with extra entries. \
             Declared: [{}]. Required exactly: [A_persistence, B_persistence, R_persistence]. \
             Operator names in an invalid sequence do not constitute a valid persistence path.",
            path.join(", ")
        ));
    }

    Ok(())
}

/// Evaluate the Continuous Relational Process Rule.
///
/// Returns `(report, failure_reason)`.
///
/// * `report` — the `ContinuousProcessReport` to embed in the record.
/// * `failure_reason` — `Some(msg)` when the evaluator must fail the evaluation.
///   Covers: missing persistence declaration (UT-05), invalid path (UT-06/07/08),
///   and synthetic initialization (UT-04).
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
/// 6. Persistence declaration is required (UT-05).
/// 7. Persistence path must be exactly [A_persistence, B_persistence, R_persistence] (UT-06/07/08).
/// 8. Available prior without traceable refs is downgraded to OPEN (UT-10).
fn evaluate_continuous_process(
    persistence: &Option<crate::declaration::PersistenceDeclaration>,
    evaluation_notes: &mut Vec<String>,
) -> (Option<ContinuousProcessReport>, Option<String>) {
    match persistence {
        None => {
            // UT-05: Persistence declaration is required.
            // A declaration that omits the persistence block cannot be assessed
            // for compliance with the Continuous Relational Process Rule and is
            // not eligible for Origin authorization until the required declaration
            // is supplied.
            let failure_msg =
                "Continuous Relational Process Rule: persistence declaration is required. \
                 Every evaluation enters an ongoing relational process. The declaration \
                 must include a 'persistence' block specifying process_entry, \
                 prior_evidence status, and the Kernel V8 persistence_path \
                 [A_persistence, B_persistence, R_persistence]. This declaration is not \
                 eligible for authorization until the required persistence block is supplied.".to_string();
            evaluation_notes.push(format!("CONTINUOUS PROCESS VIOLATION: {}", failure_msg));
            (Some(ContinuousProcessReport {
                process_entry_declared:           false,
                process_entry_statement:          None,
                prior_evidence:                   PriorEvidenceReport::Undeclared,
                persistence_path:                 vec![],
                persistence_path_valid:           false,
                persistence_path_validation_note: Some(
                    "No persistence block declared. Path cannot be validated.".to_string()
                ),
                prior_evidence_refs_count:        0,
                open_conditions:                  vec![],
            }), Some(failure_msg))
        }

        Some(p) => {
            let mut open_conditions: Vec<String> = vec![];
            let mut failure_reason: Option<String> = None;

            // ── Path validation (UT-06, UT-07, UT-08) ────────────────────────
            let (path_valid, path_validation_note) = match validate_persistence_path(&p.persistence_path) {
                Ok(()) => {
                    evaluation_notes.push(format!(
                        "CONTINUOUS PROCESS: Declared persistence path [{}] is valid. \
                         Matches Kernel V8 requirement: \
                         A_persistence → B_persistence → R_persistence.",
                        p.persistence_path.join(", ")
                    ));
                    (true, None)
                }
                Err(reason) => {
                    let note = format!("CONTINUOUS PROCESS VIOLATION: Persistence path invalid — {}", reason);
                    evaluation_notes.push(note.clone());
                    if failure_reason.is_none() {
                        failure_reason = Some(format!("Persistence path invalid: {}", reason));
                    }
                    (false, Some(reason))
                }
            };

            // ── Prior evidence status (UT-04, UT-10, UT-11) ──────────────────
            let prior_evidence_refs_count = p.prior_evidence_refs.len();

            let prior_evidence_report = match &p.prior_evidence {
                PriorEvidenceStatus::Available => {
                    if p.prior_evidence_refs.is_empty() {
                        // UT-10: Available claimed but no traceable evidence refs supplied.
                        // Downgrade to OPEN. A declaration that merely asserts availability
                        // without evidence cannot be treated as evidence of availability.
                        let cond =
                            "OPEN — Prior evidence claimed available but no traceable evidence \
                             references supplied. Availability is not independently established. \
                             A declaration that merely asserts availability without structured \
                             evidence references does not constitute evidence of availability. \
                             Reporting as OPEN. Supply prior_evidence_refs with at least one \
                             EvidenceRef identifying the prior evaluation record.".to_string();
                        evaluation_notes.push(format!("CONTINUOUS PROCESS: {}", cond));
                        open_conditions.push(cond);
                        PriorEvidenceReport::Open
                    } else {
                        // UT-11: Available with evidence refs. Accept the claim.
                        evaluation_notes.push(format!(
                            "CONTINUOUS PROCESS: Prior relational evidence declared as available \
                             with {} traceable reference(s). Evaluation proceeds from existing \
                             relational prior. Process entry is onto an ongoing process; \
                             first observation begins the evaluation record, not the physical process.",
                            prior_evidence_refs_count
                        ));
                        PriorEvidenceReport::Available
                    }
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
                    if failure_reason.is_none() {
                        failure_reason = Some(
                            "Continuous Relational Process Rule violation: synthetic initialization \
                             is prohibited. Fabricated zeros, cold-start states, and non-observed \
                             prior relational results are not admissible as prior relational evidence. \
                             PersistenceState::cold_start is not a valid entry point at Kernel V8. \
                             If prior relational evidence is genuinely unavailable at evaluation window \
                             open, declare prior_evidence as 'open' — the evaluator will report the \
                             dependency as OPEN without fabricating a value. Actual sensor readings of \
                             0.0 remain admissible; they are PrimaryObservable, not synthetic.".to_string()
                        );
                    }
                    PriorEvidenceReport::SyntheticViolation
                }
            };

            let report = ContinuousProcessReport {
                process_entry_declared:           true,
                process_entry_statement:          Some(p.process_entry.clone()),
                prior_evidence:                   prior_evidence_report,
                persistence_path:                 p.persistence_path.clone(),
                persistence_path_valid:           path_valid,
                persistence_path_validation_note: path_validation_note,
                prior_evidence_refs_count,
                open_conditions,
            };
            (Some(report), failure_reason)
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
