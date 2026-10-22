/// Pre-build evaluator.
///
/// Takes a `Declaration` (loaded from a JSON file) and produces an
/// `EvaluationRecord` that documents what the evaluator found.
///
/// The evaluator runs before any `DeclarationPackage` is submitted to the gate.
/// It does not execute mappings. It does not require the gate to be constructed.
/// It does not contain application-specific logic.

use crate::declaration::{Declaration, InputClassification, ProvenanceStatus};
use crate::record::{
    EvaluationRecord, InventoryItem, InventoryKind,
    PrerequisiteReport, MappingChainEntry, EvaluationStatus,
};
use crate::provenance;
use crate::evidence;

pub struct Evaluator;

impl Evaluator {
    /// Evaluate a declaration and produce a record.
    ///
    /// Does not execute mappings. Does not submit values to the gate.
    /// Does not require the gate to be constructed or the application to be present.
    pub fn evaluate(decl: Declaration) -> EvaluationRecord {
        let record_id = format!("{}-{}-{}", decl.domain, decl.topology_id, decl.version);

        // ── Input inventory ───────────────────────────────────────────────────
        let input_inventory: Vec<InventoryItem> = decl.inputs
            .iter()
            .map(|i| {
                let kind = map_classification(&i.classification);
                let provenance_note = build_provenance_note(i);
                InventoryItem {
                    name:                    i.name.clone(),
                    classification:          kind,
                    source_identity:         i.source_identity.clone(),
                    acquisition_method:      i.acquisition_method.clone(),
                    relationship_to_problem: i.relationship_to_problem.clone(),
                    transformation_history:  i.transformation_history.clone(),
                    evidence_count:          i.supporting_evidence.len(),
                    known_limitations:       i.known_limitations.clone(),
                    provenance_status:       i.provenance_status.clone(),
                    provenance_note,
                    submitted_to_gate:       i.submitted_to_gate,
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
            interior_locus_verified:   p.interior_locus_verified,
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

        // ── Description in notes if provided ──────────────────────────────────
        if let Some(desc) = &decl.description {
            evaluation_notes.insert(0, format!("Problem space: {}", desc));
        }

        // ── Evaluation result ─────────────────────────────────────────────────
        let (status, failure_reason) = if !all_fields_declared {
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
            domain:               decl.domain,
            topology_id:          decl.topology_id,
            version:              decl.version,
            input_inventory,
            provenance_summary,
            regional_prerequisite,
            mapping_chain,
            status,
            failure_reason,
            authorization:        None,
            evaluation_notes,
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn map_classification(c: &InputClassification) -> InventoryKind {
    match c {
        InputClassification::PrimaryObservable => InventoryKind::PrimaryObservable,
        InputClassification::DeclaredParameter => InventoryKind::DeclaredParameter,
        InputClassification::DerivedValue      => InventoryKind::DerivedValue,
        InputClassification::SimulatedInput    => InventoryKind::SimulatedInput,
        InputClassification::AbstractInput     => InventoryKind::AbstractInput,
    }
}

fn build_provenance_note(input: &crate::declaration::InputDeclaration) -> String {
    let has_evidence = !input.supporting_evidence.is_empty();

    match &input.provenance_status {
        ProvenanceStatus::Declared => {
            if has_evidence {
                format!(
                    "Provenance declared by submitter. {} evidence reference(s) provided for Origin review.",
                    input.supporting_evidence.len()
                )
            } else {
                "Provenance declared by submitter. Not independently verified. No supporting evidence provided.".to_string()
            }
        }
        ProvenanceStatus::Verified => {
            if has_evidence {
                format!(
                    "Provenance claimed as verified with {} supporting evidence reference(s). \
                     Origin should confirm evidence is accessible and evaluated.",
                    input.supporting_evidence.len()
                )
            } else {
                "Provenance claimed as verified but no supporting evidence references provided. \
                 This is inconsistent — see evaluation_notes.".to_string()
            }
        }
        ProvenanceStatus::Unverified => {
            "Provenance explicitly marked as unverified by the submitter. \
             Origin should determine whether independent verification is required \
             before authorizing this topology.".to_string()
        }
    }
}
