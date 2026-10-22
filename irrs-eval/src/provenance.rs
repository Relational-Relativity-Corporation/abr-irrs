/// Provenance analysis helpers.
///
/// Analyzes the provenance claims in a set of `InputDeclaration`s and
/// produces a `ProvenanceSummary` that the evaluator includes in the record.
///
/// # What the evaluator can and cannot do
///
/// The evaluator can check whether provenance claims are internally consistent
/// (e.g., a claim of `Verified` status with no evidence references is inconsistent).
///
/// The evaluator CANNOT independently verify physical acquisition. It cannot
/// confirm that a sensor was calibrated, that a clock was accurate, or that
/// a declared constant was actually measured. These are Origin's determinations.

use crate::declaration::{InputDeclaration, ProvenanceStatus};

// ── Summary ───────────────────────────────────────────────────────────────────

/// Aggregated provenance status across all declared inputs.
///
/// Included in `EvaluationRecord` so Origin sees at a glance what was
/// claimed and what the evaluator assessed structurally.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceSummary {
    /// Inputs whose provenance status is claimed as `Declared`.
    pub claimed_declared: usize,

    /// Inputs whose provenance status is claimed as `Verified`.
    pub claimed_verified: usize,

    /// Inputs explicitly marked as `Unverified` by the submitter.
    pub claimed_unverified: usize,

    /// Inputs with at least one evidence reference.
    pub inputs_with_evidence: usize,

    /// Names of inputs that carry no supporting evidence references.
    pub inputs_without_evidence: Vec<String>,

    /// Number of inputs whose provenance claim is internally inconsistent
    /// (e.g., `Verified` claimed without any evidence references).
    pub inconsistent_claims: usize,

    /// The evaluator's note about provenance posture and its own limits.
    ///
    /// Always includes an explicit statement that the evaluator records
    /// submitter claims only. Independent verification is Origin's determination.
    pub evaluator_note: String,
}

/// Analyze all declared inputs and produce a provenance summary.
pub fn analyze(inputs: &[InputDeclaration]) -> ProvenanceSummary {
    let mut claimed_declared    = 0usize;
    let mut claimed_verified    = 0usize;
    let mut claimed_unverified  = 0usize;
    let mut with_evidence       = 0usize;
    let mut without_evidence    = Vec::new();
    let mut inconsistent_claims = 0usize;

    for input in inputs {
        match input.provenance_status {
            ProvenanceStatus::Declared    => claimed_declared    += 1,
            ProvenanceStatus::Verified    => claimed_verified    += 1,
            ProvenanceStatus::Unverified  => claimed_unverified  += 1,
        }

        if input.supporting_evidence.is_empty() {
            without_evidence.push(input.name.clone());
        } else {
            with_evidence += 1;
        }

        // Structural inconsistency: Verified claimed with no evidence.
        if matches!(input.provenance_status, ProvenanceStatus::Verified)
            && input.supporting_evidence.is_empty()
        {
            inconsistent_claims += 1;
        }
    }

    let note = build_evaluator_note(
        claimed_declared, claimed_verified, claimed_unverified,
        inconsistent_claims, &without_evidence,
    );

    ProvenanceSummary {
        claimed_declared,
        claimed_verified,
        claimed_unverified,
        inputs_with_evidence: with_evidence,
        inputs_without_evidence: without_evidence,
        inconsistent_claims,
        evaluator_note: note,
    }
}

fn build_evaluator_note(
    declared:    usize,
    verified:    usize,
    unverified:  usize,
    inconsistent: usize,
    without_evidence: &[String],
) -> String {
    // The evaluator note always opens with an explicit statement of its limits.
    let mut parts = vec![
        "The evaluator records submitter claims. \
         It does not independently verify physical acquisition. \
         Independent verification is Origin's determination to make.".to_string()
    ];

    if inconsistent > 0 {
        parts.push(format!(
            "INCONSISTENCY: {} input(s) claim 'verified' provenance status \
             but provide no supporting evidence references. \
             A claim of 'verified' without evidence is not self-supporting. \
             See evaluation_notes for details.",
            inconsistent
        ));
    }

    if unverified > 0 {
        parts.push(format!(
            "{} input(s) are explicitly marked 'unverified' by the submitter. \
             Origin should determine whether independent verification is required \
             before authorizing this topology.",
            unverified
        ));
    }

    if !without_evidence.is_empty() && inconsistent == 0 {
        // Only note this if we haven't already flagged inconsistent claims.
        parts.push(format!(
            "{} input(s) carry no supporting evidence ({}). \
             This is acceptable for a declared-only submission; \
             Origin notes which inputs would benefit from independent evidence.",
            without_evidence.len(),
            without_evidence.join(", ")
        ));
    }

    if verified > 0 && inconsistent == 0 {
        parts.push(format!(
            "{} input(s) claim 'verified' status with at least one evidence reference. \
             The evaluator has confirmed structural consistency of this claim. \
             Origin must confirm that referenced evidence is accessible and supports \
             the classification claimed.",
            verified
        ));
    }

    let _ = (declared, verified, unverified); // suppress unused warnings

    parts.join(" ")
}
