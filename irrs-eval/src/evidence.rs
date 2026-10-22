/// Evidence analysis helpers.
///
/// Examines `EvidenceRef` entries across the declaration and produces
/// observations the evaluator includes in the record's evaluation notes.

use crate::declaration::InputDeclaration;

/// Produce evaluation notes about evidence coverage across all declared inputs.
///
/// Returns a list of plain-language observations for Origin review.
/// An empty note list means there is nothing unusual to flag.
pub fn evaluate_coverage(inputs: &[InputDeclaration]) -> Vec<String> {
    let mut notes = Vec::new();

    // Inputs with `Verified` provenance status but no evidence references
    // are an inconsistency: "verified" requires something to have been evaluated.
    let verified_without_evidence: Vec<&str> = inputs
        .iter()
        .filter(|i| {
            matches!(i.provenance_status, crate::declaration::ProvenanceStatus::Verified)
                && i.supporting_evidence.is_empty()
        })
        .map(|i| i.name.as_str())
        .collect();

    if !verified_without_evidence.is_empty() {
        notes.push(format!(
            "INCONSISTENCY: input(s) claim 'verified' provenance status but provide \
             no supporting evidence references: {}. \
             Either add evidence references or change status to 'declared'.",
            verified_without_evidence.join(", ")
        ));
    }

    // Inputs with evidence references
    let evidence_count: usize = inputs.iter().map(|i| i.supporting_evidence.len()).sum();
    if evidence_count > 0 {
        notes.push(format!(
            "{} evidence reference(s) declared across {} input(s). \
             Origin should confirm that referenced evidence is accessible and \
             supports the classification claimed.",
            evidence_count,
            inputs.iter().filter(|i| !i.supporting_evidence.is_empty()).count()
        ));
    }

    notes
}
